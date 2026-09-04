//! Adapter from the machine-readable `OpenSpec` CLI contract to Sigillum core.

#![forbid(unsafe_code)]

mod json;

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;
use std::fs;
use std::io;
use std::path::{Component, Path, PathBuf};
use std::process::Command;

use json::Value;
use sigillum_core::contract::{ArtifactInput, Snapshot, SnapshotError};

const MAX_ARTIFACT_COUNT: usize = 1_024;
const MAX_ARTIFACT_BYTES: u64 = 8 * 1024 * 1024;
const MAX_CLOSURE_BYTES: u64 = 32 * 1024 * 1024;

/// Successful result of resolving an `OpenSpec` change into a Sigillum contract.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LoadedContract {
    openspec_version: String,
    snapshot: Snapshot,
}

impl LoadedContract {
    /// Returns the detected `OpenSpec` CLI version string.
    #[must_use]
    pub fn openspec_version(&self) -> &str {
        &self.openspec_version
    }

    /// Returns the canonical Sigillum contract snapshot.
    #[must_use]
    pub const fn snapshot(&self) -> &Snapshot {
        &self.snapshot
    }

    /// Consumes the wrapper and returns the canonical snapshot.
    #[must_use]
    pub fn into_snapshot(self) -> Snapshot {
        self.snapshot
    }
}

/// Process-backed client for the machine-readable `OpenSpec` CLI.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Client {
    executable: PathBuf,
}

impl Default for Client {
    fn default() -> Self {
        let executable = if cfg!(windows) {
            PathBuf::from("openspec.cmd")
        } else {
            PathBuf::from("openspec")
        };
        Self { executable }
    }
}

impl Client {
    /// Creates a client using a specific `OpenSpec` executable.
    #[must_use]
    pub fn new(executable: impl Into<PathBuf>) -> Self {
        Self {
            executable: executable.into(),
        }
    }

    /// Resolves an `OpenSpec` change and builds its canonical contract snapshot.
    ///
    /// # Errors
    ///
    /// Returns [`AdapterError`] when the CLI cannot run, emits an incompatible
    /// payload, reports incomplete planning, exposes unsafe file paths, or when
    /// the core rejects the resulting artifact closure.
    pub fn load_contract(
        &self,
        project_root: &Path,
        change_id: &str,
    ) -> Result<LoadedContract, AdapterError> {
        let openspec_version = self.detect_version(project_root)?;
        let status_output = self.run_json(
            project_root,
            "status",
            &["status", "--change", change_id, "--json"],
        )?;
        let status = parse_status(&status_output)?;
        if !status.planning_complete {
            return Err(AdapterError::PlanningIncomplete(change_id.to_owned()));
        }

        let apply_output = self.run_json(
            project_root,
            "instructions apply",
            &["instructions", "apply", "--change", change_id, "--json"],
        )?;
        let apply = parse_apply(&apply_output)?;
        let protocol = reconcile(status, apply, change_id)?;
        let snapshot = load_snapshot(protocol)?;

        Ok(LoadedContract {
            openspec_version,
            snapshot,
        })
    }

    fn detect_version(&self, project_root: &Path) -> Result<String, AdapterError> {
        let output = Command::new(&self.executable)
            .arg("--version")
            .current_dir(project_root)
            .output()
            .map_err(|source| AdapterError::Spawn {
                executable: self.executable.clone(),
                source,
            })?;
        if !output.status.success() {
            return Err(AdapterError::CommandFailed {
                command: "--version",
                exit_code: output.status.code(),
                detail: output_detail(&output.stdout, &output.stderr),
            });
        }

        let version = String::from_utf8(output.stdout)
            .map_err(|_| AdapterError::Protocol("version output is not UTF-8".to_owned()))?;
        let version = version.trim();
        if version.is_empty() || !version.bytes().any(|byte| byte.is_ascii_digit()) {
            return Err(AdapterError::Protocol(
                "version output does not contain a version number".to_owned(),
            ));
        }
        Ok(version.to_owned())
    }

    fn run_json(
        &self,
        project_root: &Path,
        command_name: &'static str,
        arguments: &[&str],
    ) -> Result<Vec<u8>, AdapterError> {
        let output = Command::new(&self.executable)
            .args(arguments)
            .current_dir(project_root)
            .output()
            .map_err(|source| AdapterError::Spawn {
                executable: self.executable.clone(),
                source,
            })?;
        if output.status.success() {
            Ok(output.stdout)
        } else {
            Err(AdapterError::CommandFailed {
                command: command_name,
                exit_code: output.status.code(),
                detail: output_detail(&output.stdout, &output.stderr),
            })
        }
    }
}

/// Failure while converting `OpenSpec` planning state into a Sigillum contract.
#[derive(Debug)]
pub enum AdapterError {
    /// The configured executable could not be started.
    Spawn {
        /// Configured executable path.
        executable: PathBuf,
        /// Operating-system error.
        source: io::Error,
    },
    /// An `OpenSpec` command returned a non-zero status.
    CommandFailed {
        /// Stable command label.
        command: &'static str,
        /// Process exit code, if the operating system supplied one.
        exit_code: Option<i32>,
        /// Bounded diagnostic detail from stdout or stderr.
        detail: String,
    },
    /// A JSON document was malformed.
    InvalidJson(String),
    /// A JSON document was valid but violated the documented protocol.
    Protocol(String),
    /// Planning artifacts are not ready for execution.
    PlanningIncomplete(String),
    /// An artifact path escaped the selected planning root or change directory.
    UnsafeArtifactPath(PathBuf),
    /// Reading or canonicalizing a planning artifact failed.
    ArtifactIo {
        /// Artifact or planning-root path.
        path: PathBuf,
        /// Filesystem error.
        source: io::Error,
    },
    /// The artifact closure exceeded a deterministic resource limit.
    ArtifactLimit(String),
    /// Sigillum core rejected the canonical closure.
    Snapshot(SnapshotError),
}

impl fmt::Display for AdapterError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Spawn { executable, source } => {
                write!(formatter, "failed to start {}: {source}", executable.display())
            }
            Self::CommandFailed {
                command,
                exit_code,
                detail,
            } => write!(
                formatter,
                "openspec {command} failed with exit code {exit_code:?}: {detail}"
            ),
            Self::InvalidJson(message) => write!(formatter, "invalid OpenSpec JSON: {message}"),
            Self::Protocol(message) => write!(formatter, "incompatible OpenSpec payload: {message}"),
            Self::PlanningIncomplete(change) => {
                write!(formatter, "OpenSpec change {change:?} is not planning-complete")
            }
            Self::UnsafeArtifactPath(path) => {
                write!(formatter, "artifact path is outside the approved change: {}", path.display())
            }
            Self::ArtifactIo { path, source } => {
                write!(formatter, "cannot read {}: {source}", path.display())
            }
            Self::ArtifactLimit(message) => formatter.write_str(message),
            Self::Snapshot(source) => write!(formatter, "invalid contract snapshot: {source}"),
        }
    }
}

impl Error for AdapterError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Spawn { source, .. } | Self::ArtifactIo { source, .. } => Some(source),
            Self::Snapshot(source) => Some(source),
            _ => None,
        }
    }
}

impl From<SnapshotError> for AdapterError {
    fn from(source: SnapshotError) -> Self {
        Self::Snapshot(source)
    }
}

struct StatusPayload {
    change_name: String,
    schema_name: String,
    change_root: PathBuf,
    root: PathBuf,
    planning_complete: bool,
}

struct ApplyPayload {
    change_name: String,
    schema_name: String,
    change_dir: PathBuf,
    root: PathBuf,
    state: String,
    context_files: BTreeMap<String, Vec<PathBuf>>,
}

struct ProtocolClosure {
    change_name: String,
    schema_name: String,
    change_dir: PathBuf,
    root: PathBuf,
    context_files: BTreeMap<String, Vec<PathBuf>>,
}

struct OwnedArtifact {
    artifact_id: String,
    relative_path: String,
    content: Vec<u8>,
}

fn parse_status(input: &[u8]) -> Result<StatusPayload, AdapterError> {
    let value = json::parse(input).map_err(|error| AdapterError::InvalidJson(error.to_string()))?;
    let object = required_object(&value, "status response")?;
    Ok(StatusPayload {
        change_name: required_string(object, "changeName")?.to_owned(),
        schema_name: required_string(object, "schemaName")?.to_owned(),
        change_root: PathBuf::from(required_string(object, "changeRoot")?),
        root: parse_root(object)?,
        planning_complete: required_bool(object, "isPlanningComplete")?,
    })
}

fn parse_apply(input: &[u8]) -> Result<ApplyPayload, AdapterError> {
    let value = json::parse(input).map_err(|error| AdapterError::InvalidJson(error.to_string()))?;
    let object = required_object(&value, "apply response")?;
    let context_object = required_object(
        required_field(object, "contextFiles")?,
        "contextFiles",
    )?;
    if context_object.len() > MAX_ARTIFACT_COUNT {
        return Err(AdapterError::ArtifactLimit(format!(
            "OpenSpec returned more than {MAX_ARTIFACT_COUNT} artifact groups"
        )));
    }

    let mut context_files = BTreeMap::new();
    let mut file_count = 0_usize;
    for (artifact_id, paths) in context_object {
        let paths = paths.as_array().ok_or_else(|| {
            AdapterError::Protocol(format!("contextFiles.{artifact_id} must be an array"))
        })?;
        if paths.is_empty() {
            return Err(AdapterError::Protocol(format!(
                "contextFiles.{artifact_id} must not be empty"
            )));
        }
        file_count = file_count.saturating_add(paths.len());
        if file_count > MAX_ARTIFACT_COUNT {
            return Err(AdapterError::ArtifactLimit(format!(
                "OpenSpec returned more than {MAX_ARTIFACT_COUNT} artifact files"
            )));
        }
        let paths = paths
            .iter()
            .map(|path| {
                path.as_str().map(PathBuf::from).ok_or_else(|| {
                    AdapterError::Protocol(format!(
                        "contextFiles.{artifact_id} entries must be strings"
                    ))
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        context_files.insert(artifact_id.clone(), paths);
    }

    Ok(ApplyPayload {
        change_name: required_string(object, "changeName")?.to_owned(),
        schema_name: required_string(object, "schemaName")?.to_owned(),
        change_dir: PathBuf::from(required_string(object, "changeDir")?),
        root: parse_root(object)?,
        state: required_string(object, "state")?.to_owned(),
        context_files,
    })
}

fn reconcile(
    status: StatusPayload,
    apply: ApplyPayload,
    requested_change: &str,
) -> Result<ProtocolClosure, AdapterError> {
    if status.change_name != requested_change || apply.change_name != requested_change {
        return Err(AdapterError::Protocol(
            "changeName does not match the requested change".to_owned(),
        ));
    }
    if status.schema_name != apply.schema_name {
        return Err(AdapterError::Protocol(
            "status and apply schemaName values differ".to_owned(),
        ));
    }
    if status.change_root != apply.change_dir {
        return Err(AdapterError::Protocol(
            "status changeRoot and apply changeDir values differ".to_owned(),
        ));
    }
    if status.root != apply.root {
        return Err(AdapterError::Protocol(
            "status and apply root paths differ".to_owned(),
        ));
    }
    if apply.state == "blocked" {
        return Err(AdapterError::PlanningIncomplete(requested_change.to_owned()));
    }
    if !matches!(apply.state.as_str(), "ready" | "all_done") {
        return Err(AdapterError::Protocol(format!(
            "unknown apply state {:?}",
            apply.state
        )));
    }
    if apply.context_files.is_empty() {
        return Err(AdapterError::Protocol(
            "contextFiles must not be empty for a ready change".to_owned(),
        ));
    }

    Ok(ProtocolClosure {
        change_name: status.change_name,
        schema_name: status.schema_name,
        change_dir: status.change_root,
        root: status.root,
        context_files: apply.context_files,
    })
}

fn load_snapshot(protocol: ProtocolClosure) -> Result<Snapshot, AdapterError> {
    let root = canonicalize(&protocol.root)?;
    let change_dir = canonicalize(&protocol.change_dir)?;
    if !change_dir.starts_with(&root) {
        return Err(AdapterError::UnsafeArtifactPath(protocol.change_dir));
    }

    let mut total_bytes = 0_u64;
    let mut artifacts = Vec::new();
    for (artifact_id, paths) in protocol.context_files {
        for path in paths {
            if !path.is_absolute() {
                return Err(AdapterError::UnsafeArtifactPath(path));
            }
            let canonical_path = canonicalize(&path)?;
            if !canonical_path.starts_with(&change_dir) {
                return Err(AdapterError::UnsafeArtifactPath(path));
            }
            let metadata = fs::metadata(&canonical_path).map_err(|source| AdapterError::ArtifactIo {
                path: canonical_path.clone(),
                source,
            })?;
            if !metadata.is_file() {
                return Err(AdapterError::UnsafeArtifactPath(canonical_path));
            }
            if metadata.len() > MAX_ARTIFACT_BYTES {
                return Err(AdapterError::ArtifactLimit(format!(
                    "artifact {} exceeds the {} byte limit",
                    canonical_path.display(),
                    MAX_ARTIFACT_BYTES
                )));
            }
            let content = fs::read(&canonical_path).map_err(|source| AdapterError::ArtifactIo {
                path: canonical_path.clone(),
                source,
            })?;
            let content_len = u64::try_from(content.len())
                .map_err(|_| AdapterError::ArtifactLimit("artifact length exceeds u64".to_owned()))?;
            if content_len > MAX_ARTIFACT_BYTES {
                return Err(AdapterError::ArtifactLimit(format!(
                    "artifact {} changed beyond the {} byte limit while reading",
                    canonical_path.display(),
                    MAX_ARTIFACT_BYTES
                )));
            }
            total_bytes = total_bytes.saturating_add(content_len);
            if total_bytes > MAX_CLOSURE_BYTES {
                return Err(AdapterError::ArtifactLimit(format!(
                    "artifact closure exceeds the {MAX_CLOSURE_BYTES} byte limit"
                )));
            }

            let relative_path = canonical_path
                .strip_prefix(&root)
                .map_err(|_| AdapterError::UnsafeArtifactPath(canonical_path.clone()))?;
            artifacts.push(OwnedArtifact {
                artifact_id: artifact_id.clone(),
                relative_path: slash_path(relative_path)?,
                content,
            });
        }
    }

    let inputs = artifacts
        .iter()
        .map(|artifact| ArtifactInput {
            artifact_id: &artifact.artifact_id,
            relative_path: &artifact.relative_path,
            content: &artifact.content,
        })
        .collect::<Vec<_>>();
    Snapshot::build(&protocol.change_name, &protocol.schema_name, &inputs).map_err(Into::into)
}

fn canonicalize(path: &Path) -> Result<PathBuf, AdapterError> {
    fs::canonicalize(path).map_err(|source| AdapterError::ArtifactIo {
        path: path.to_owned(),
        source,
    })
}

fn slash_path(path: &Path) -> Result<String, AdapterError> {
    let mut output = String::new();
    for component in path.components() {
        let Component::Normal(value) = component else {
            return Err(AdapterError::UnsafeArtifactPath(path.to_owned()));
        };
        let value = value
            .to_str()
            .ok_or_else(|| AdapterError::Protocol("artifact path is not UTF-8".to_owned()))?;
        if !output.is_empty() {
            output.push('/');
        }
        output.push_str(value);
    }
    if output.is_empty() {
        Err(AdapterError::UnsafeArtifactPath(path.to_owned()))
    } else {
        Ok(output)
    }
}

fn parse_root(object: &BTreeMap<String, Value>) -> Result<PathBuf, AdapterError> {
    let root = required_object(required_field(object, "root")?, "root")?;
    Ok(PathBuf::from(required_string(root, "path")?))
}

fn required_field<'a>(
    object: &'a BTreeMap<String, Value>,
    field: &str,
) -> Result<&'a Value, AdapterError> {
    object
        .get(field)
        .ok_or_else(|| AdapterError::Protocol(format!("missing field {field:?}")))
}

fn required_object<'a>(
    value: &'a Value,
    name: &str,
) -> Result<&'a BTreeMap<String, Value>, AdapterError> {
    value
        .as_object()
        .ok_or_else(|| AdapterError::Protocol(format!("{name} must be an object")))
}

fn required_string<'a>(
    object: &'a BTreeMap<String, Value>,
    field: &str,
) -> Result<&'a str, AdapterError> {
    required_field(object, field)?
        .as_str()
        .ok_or_else(|| AdapterError::Protocol(format!("field {field:?} must be a string")))
}

fn required_bool(object: &BTreeMap<String, Value>, field: &str) -> Result<bool, AdapterError> {
    required_field(object, field)?
        .as_bool()
        .ok_or_else(|| AdapterError::Protocol(format!("field {field:?} must be a boolean")))
}

fn output_detail(stdout: &[u8], stderr: &[u8]) -> String {
    const MAX_DETAIL_BYTES: usize = 4_096;
    let bytes = if stdout.is_empty() { stderr } else { stdout };
    let end = bytes.len().min(MAX_DETAIL_BYTES);
    String::from_utf8_lossy(&bytes[..end]).trim().to_owned()
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::{load_snapshot, parse_apply, parse_status, reconcile, AdapterError};
    #[cfg(unix)]
    use super::Client;

    #[test]
    fn parses_and_reconciles_official_payload_shapes() {
        let root = temporary_root("ready");
        let change = root.join("openspec/changes/add-auth");
        fs::create_dir_all(&change).expect("create fixture change");
        let proposal = change.join("proposal.md");
        let tasks = change.join("tasks.md");
        fs::write(&proposal, "# Proposal\n").expect("write proposal");
        fs::write(&tasks, "- [ ] Implement\n").expect("write tasks");

        let status = parse_status(
            status_json(&root, &change, true, "add-auth", "spec-driven").as_bytes(),
        )
        .expect("valid status");
        let apply = parse_apply(
            apply_json(&root, &change, &proposal, &tasks, "ready").as_bytes(),
        )
        .expect("valid apply");
        let snapshot = load_snapshot(reconcile(status, apply, "add-auth").expect("matching payloads"))
            .expect("valid snapshot");

        assert_eq!(snapshot.change_id(), "add-auth");
        assert_eq!(snapshot.openspec_schema(), "spec-driven");
        assert_eq!(snapshot.artifacts().len(), 2);
        cleanup(&root);
    }

    #[test]
    fn rejects_incomplete_or_mismatched_protocol_state() {
        let root = temporary_root("blocked");
        let change = root.join("openspec/changes/add-auth");
        let status = parse_status(
            status_json(&root, &change, false, "add-auth", "spec-driven").as_bytes(),
        )
        .expect("valid status");
        assert!(!status.planning_complete);

        let status = parse_status(
            status_json(&root, &change, true, "add-auth", "spec-driven").as_bytes(),
        )
        .expect("valid status");
        let apply = parse_apply(
            apply_json(
                &root,
                &change,
                &change.join("proposal.md"),
                &change.join("tasks.md"),
                "blocked",
            )
            .as_bytes(),
        )
        .expect("valid apply");
        assert!(matches!(
            reconcile(status, apply, "add-auth"),
            Err(AdapterError::PlanningIncomplete(change)) if change == "add-auth"
        ));

        let status = parse_status(
            status_json(&root, &change, true, "add-auth", "spec-driven").as_bytes(),
        )
        .expect("valid status");
        let apply = parse_apply(
            apply_json(
                &root,
                &change,
                &change.join("proposal.md"),
                &change.join("tasks.md"),
                "paused",
            )
            .as_bytes(),
        )
        .expect("valid apply");
        assert!(matches!(
            reconcile(status, apply, "add-auth"),
            Err(AdapterError::Protocol(message)) if message.contains("unknown apply state")
        ));
    }

    #[test]
    fn rejects_artifact_outside_change_directory() {
        let root = temporary_root("escape");
        let change = root.join("openspec/changes/add-auth");
        fs::create_dir_all(&change).expect("create fixture change");
        let outside = root.join("secret.md");
        fs::write(&outside, "secret").expect("write outside file");
        let status = parse_status(
            status_json(&root, &change, true, "add-auth", "spec-driven").as_bytes(),
        )
        .expect("valid status");
        let apply = parse_apply(
            apply_json(&root, &change, &outside, &change.join("tasks.md"), "ready").as_bytes(),
        )
        .expect("valid apply");
        let protocol = reconcile(status, apply, "add-auth").expect("matching payloads");

        assert!(matches!(
            load_snapshot(protocol),
            Err(AdapterError::UnsafeArtifactPath(path)) if path == outside
        ));
        cleanup(&root);
    }

    #[cfg(unix)]
    #[test]
    fn invokes_cli_contract_without_an_installed_openspec() {
        use std::os::unix::fs::PermissionsExt;

        let root = temporary_root("fake-cli");
        let change = root.join("openspec/changes/add-auth");
        fs::create_dir_all(&change).expect("create fixture change");
        let proposal = change.join("proposal.md");
        let tasks = change.join("tasks.md");
        fs::write(&proposal, "# Proposal\n").expect("write proposal");
        fs::write(&tasks, "- [ ] Implement\n").expect("write tasks");

        let executable = root.join("fake-openspec");
        let status = status_json(&root, &change, true, "add-auth", "spec-driven");
        let apply = apply_json(&root, &change, &proposal, &tasks, "ready");
        let script = format!(
            "#!/bin/sh\ncase \"$1\" in\n  --version) printf '%s\\n' 'OpenSpec 1.2.3' ;;\n  status) printf '%s\\n' '{status}' ;;\n  instructions) printf '%s\\n' '{apply}' ;;\n  *) exit 64 ;;\nesac\n"
        );
        fs::write(&executable, script).expect("write fake executable");
        let mut permissions = fs::metadata(&executable)
            .expect("fake executable metadata")
            .permissions();
        permissions.set_mode(0o700);
        fs::set_permissions(&executable, permissions).expect("make fake executable runnable");

        let loaded = Client::new(&executable)
            .load_contract(&root, "add-auth")
            .expect("load through fake CLI");

        assert_eq!(loaded.openspec_version(), "OpenSpec 1.2.3");
        assert_eq!(loaded.snapshot().artifacts().len(), 2);
        cleanup(&root);
    }

    fn status_json(
        root: &Path,
        change: &Path,
        complete: bool,
        change_name: &str,
        schema_name: &str,
    ) -> String {
        include_str!("../tests/fixtures/status.json")
            .replace("__CHANGE_NAME__", change_name)
            .replace("__SCHEMA_NAME__", schema_name)
            .replace("__CHANGE_ROOT__", &json_path(change))
            .replace(
                "\"__PLANNING_COMPLETE__\"",
                if complete { "true" } else { "false" },
            )
            .replace("__ROOT__", &json_path(root))
    }

    fn apply_json(
        root: &Path,
        change: &Path,
        proposal: &Path,
        tasks: &Path,
        state: &str,
    ) -> String {
        include_str!("../tests/fixtures/instructions-apply.json")
            .replace("__CHANGE_ROOT__", &json_path(change))
            .replace("__PROPOSAL__", &json_path(proposal))
            .replace("__TASKS__", &json_path(tasks))
            .replace("__STATE__", state)
            .replace("__ROOT__", &json_path(root))
    }

    fn json_path(path: &Path) -> String {
        path.to_string_lossy().replace('\\', "\\\\")
    }

    fn temporary_root(label: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock after epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("sigillum-openspec-{label}-{}-{nonce}", std::process::id()))
    }

    fn cleanup(path: &Path) {
        if let Err(error) = fs::remove_dir_all(path) {
            eprintln!("failed to remove fixture {}: {error}", path.display());
        }
    }
}

