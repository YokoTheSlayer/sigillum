//! Local, append-only persistence for Sigillum contract approvals.

#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use sigillum_core::contract::{Approval, Snapshot};

const MAX_APPROVAL_BYTES: u64 = 1_024;
const MAX_APPROVAL_RECORDS: usize = 4_096;

/// Current version of the persisted approval record.
pub const APPROVAL_RECORD_SCHEMA_VERSION: u32 = 1;

/// Result of comparing local approval history with a contract snapshot.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ApprovalStatus {
    /// No approval has been recorded for this change.
    Missing,
    /// An exact, well-formed approval exists for the current fingerprint.
    Valid,
    /// Approval history exists but cannot authorize the current snapshot.
    Invalid(InvalidationReason),
}

/// Stable reason why recorded approval cannot authorize a snapshot.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InvalidationReason {
    /// The current contract fingerprint differs from every recorded approval.
    ContractChanged,
    /// A record named for the current fingerprint does not contain canonical content.
    ApprovalRecordCorrupt,
}

impl fmt::Display for InvalidationReason {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ContractChanged => formatter.write_str("contract_changed"),
            Self::ApprovalRecordCorrupt => formatter.write_str("approval_record_corrupt"),
        }
    }
}

/// Filesystem-backed, append-only approval store rooted in an OpenSpec project.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApprovalStore {
    root: PathBuf,
}

impl ApprovalStore {
    /// Opens a store below an existing project directory.
    ///
    /// # Errors
    ///
    /// Returns [`StoreError`] when the project root cannot be canonicalized or is
    /// not a directory.
    pub fn open(project_root: &Path) -> Result<Self, StoreError> {
        let root = fs::canonicalize(project_root).map_err(|source| StoreError::Io {
            path: project_root.to_owned(),
            source,
        })?;
        if !root.is_dir() {
            return Err(StoreError::UnsafeStatePath(root));
        }
        Ok(Self { root })
    }

    /// Compares the current snapshot with append-only approval history.
    ///
    /// # Errors
    ///
    /// Returns [`StoreError`] when state paths are unsafe or cannot be read.
    pub fn status(&self, snapshot: &Snapshot) -> Result<ApprovalStatus, StoreError> {
        let directory = self.contract_directory(snapshot.change_id());
        if !directory.exists() {
            return Ok(ApprovalStatus::Missing);
        }
        let directory = canonical_state_directory(&self.root, &directory)?;
        let current_path = directory.join(approval_filename(snapshot.fingerprint()));
        match fs::symlink_metadata(&current_path) {
            Ok(metadata) => {
                if metadata.file_type().is_symlink()
                    || !metadata.is_file()
                    || metadata.len() > MAX_APPROVAL_BYTES
                {
                    return Ok(ApprovalStatus::Invalid(
                        InvalidationReason::ApprovalRecordCorrupt,
                    ));
                }
                let expected = approval_json(snapshot);
                let actual = fs::read(&current_path).map_err(|source| StoreError::Io {
                    path: current_path,
                    source,
                })?;
                if actual == expected.as_bytes() {
                    Ok(ApprovalStatus::Valid)
                } else {
                    Ok(ApprovalStatus::Invalid(
                        InvalidationReason::ApprovalRecordCorrupt,
                    ))
                }
            }
            Err(source) if source.kind() == io::ErrorKind::NotFound => {
                if contains_approval_records(&directory)? {
                    Ok(ApprovalStatus::Invalid(InvalidationReason::ContractChanged))
                } else {
                    Ok(ApprovalStatus::Missing)
                }
            }
            Err(source) => Err(StoreError::Io {
                path: current_path,
                source,
            }),
        }
    }

    /// Records explicit approval for the exact expected fingerprint.
    ///
    /// Records are immutable and named by fingerprint. Repeating the same approval
    /// is idempotent; a changed contract creates a new record instead of replacing
    /// history.
    ///
    /// # Errors
    ///
    /// Returns [`StoreError`] when the expected fingerprint differs, a state path
    /// is unsafe, an existing record is corrupt, or the record cannot be written.
    pub fn approve(
        &self,
        snapshot: &Snapshot,
        expected_fingerprint: &str,
    ) -> Result<PathBuf, StoreError> {
        if snapshot.fingerprint() != expected_fingerprint {
            return Err(StoreError::FingerprintMismatch {
                expected: expected_fingerprint.to_owned(),
                actual: snapshot.fingerprint().to_owned(),
            });
        }

        let directory = ensure_contract_directory(&self.root, snapshot.change_id())?;
        let path = directory.join(approval_filename(snapshot.fingerprint()));
        let content = approval_json(snapshot);
        match OpenOptions::new().write(true).create_new(true).open(&path) {
            Ok(mut file) => {
                if let Err(source) = file
                    .write_all(content.as_bytes())
                    .and_then(|()| file.sync_all())
                {
                    drop(file);
                    let _ = fs::remove_file(&path);
                    return Err(StoreError::Io { path, source });
                }
                Ok(path)
            }
            Err(source) if source.kind() == io::ErrorKind::AlreadyExists => {
                if self.status(snapshot)? == ApprovalStatus::Valid {
                    Ok(path)
                } else {
                    Err(StoreError::CorruptApproval(path))
                }
            }
            Err(source) => Err(StoreError::Io { path, source }),
        }
    }

    fn contract_directory(&self, change_id: &str) -> PathBuf {
        self.root
            .join(".sigillum")
            .join("contracts")
            .join(change_id)
    }
}

/// Failure while reading or writing local Sigillum approval state.
#[derive(Debug)]
pub enum StoreError {
    /// A state path escaped the project or traversed a symlink/non-directory.
    UnsafeStatePath(PathBuf),
    /// The user-confirmed fingerprint does not match the freshly resolved contract.
    FingerprintMismatch {
        /// Fingerprint supplied by the user.
        expected: String,
        /// Fingerprint calculated from current OpenSpec artifacts.
        actual: String,
    },
    /// An immutable approval record already exists with unexpected content.
    CorruptApproval(PathBuf),
    /// A filesystem operation failed.
    Io {
        /// Path involved in the failed operation.
        path: PathBuf,
        /// Operating-system error.
        source: io::Error,
    },
    /// The approval directory exceeded its deterministic entry limit.
    StateLimit(String),
}

impl fmt::Display for StoreError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsafeStatePath(path) => {
                write!(formatter, "unsafe Sigillum state path: {}", path.display())
            }
            Self::FingerprintMismatch { expected, actual } => write!(
                formatter,
                "approval fingerprint mismatch: expected {expected}, current contract is {actual}"
            ),
            Self::CorruptApproval(path) => {
                write!(formatter, "approval record is corrupt: {}", path.display())
            }
            Self::Io { path, source } => {
                write!(formatter, "Sigillum state I/O failed at {}: {source}", path.display())
            }
            Self::StateLimit(message) => formatter.write_str(message),
        }
    }
}

impl Error for StoreError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            _ => None,
        }
    }
}

fn approval_filename(fingerprint: &str) -> String {
    format!("approval-{fingerprint}.json")
}

fn approval_json(snapshot: &Snapshot) -> String {
    let approval = Approval::for_snapshot(snapshot);
    format!(
        "{{\n  \"schema_version\": {APPROVAL_RECORD_SCHEMA_VERSION},\n  \"change_id\": \"{}\",\n  \"contract_fingerprint\": \"{}\"\n}}\n",
        snapshot.change_id(),
        approval.contract_fingerprint()
    )
}

fn contains_approval_records(directory: &Path) -> Result<bool, StoreError> {
    let entries = fs::read_dir(directory).map_err(|source| StoreError::Io {
        path: directory.to_owned(),
        source,
    })?;
    for (index, entry) in entries.enumerate() {
        if index >= MAX_APPROVAL_RECORDS {
            return Err(StoreError::StateLimit(format!(
                "approval directory exceeds the {MAX_APPROVAL_RECORDS} entry limit"
            )));
        }
        let entry = entry.map_err(|source| StoreError::Io {
            path: directory.to_owned(),
            source,
        })?;
        if entry
            .file_name()
            .to_str()
            .is_some_and(|name| name.starts_with("approval-") && name.ends_with(".json"))
        {
            return Ok(true);
        }
    }
    Ok(false)
}

fn ensure_contract_directory(root: &Path, change_id: &str) -> Result<PathBuf, StoreError> {
    let mut current = root.to_owned();
    for component in [".sigillum", "contracts", change_id] {
        let next = current.join(component);
        match fs::symlink_metadata(&next) {
            Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_dir() => {
                return Err(StoreError::UnsafeStatePath(next));
            }
            Ok(_) => {}
            Err(source) if source.kind() == io::ErrorKind::NotFound => {
                fs::create_dir(&next).map_err(|source| StoreError::Io {
                    path: next.clone(),
                    source,
                })?;
            }
            Err(source) => return Err(StoreError::Io { path: next, source }),
        }
        current = fs::canonicalize(&next).map_err(|source| StoreError::Io {
            path: next,
            source,
        })?;
        if !current.starts_with(root) {
            return Err(StoreError::UnsafeStatePath(current));
        }
    }
    Ok(current)
}

fn canonical_state_directory(root: &Path, directory: &Path) -> Result<PathBuf, StoreError> {
    let metadata = fs::symlink_metadata(directory).map_err(|source| StoreError::Io {
        path: directory.to_owned(),
        source,
    })?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(StoreError::UnsafeStatePath(directory.to_owned()));
    }
    let canonical = fs::canonicalize(directory).map_err(|source| StoreError::Io {
        path: directory.to_owned(),
        source,
    })?;
    if canonical.starts_with(root) {
        Ok(canonical)
    } else {
        Err(StoreError::UnsafeStatePath(canonical))
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    use sigillum_core::contract::{ArtifactInput, Snapshot};

    use super::{ApprovalStatus, ApprovalStore, InvalidationReason, StoreError};

    #[test]
    fn approval_is_explicit_idempotent_and_hash_pinned() {
        let root = temporary_root("approval");
        fs::create_dir(&root).expect("create project root");
        let store = ApprovalStore::open(&root).expect("open store");
        let original = snapshot(b"original");

        assert_eq!(store.status(&original).expect("missing status"), ApprovalStatus::Missing);
        assert!(matches!(
            store.approve(&original, "wrong"),
            Err(StoreError::FingerprintMismatch { .. })
        ));
        let first = store
            .approve(&original, original.fingerprint())
            .expect("record approval");
        let second = store
            .approve(&original, original.fingerprint())
            .expect("repeat approval");

        assert_eq!(first, second);
        assert_eq!(store.status(&original).expect("valid status"), ApprovalStatus::Valid);

        let changed = snapshot(b"changed");
        assert_eq!(
            store.status(&changed).expect("changed status"),
            ApprovalStatus::Invalid(InvalidationReason::ContractChanged)
        );
        cleanup(&root);
    }

    #[test]
    fn detects_corrupt_current_record() {
        let root = temporary_root("corrupt");
        fs::create_dir(&root).expect("create project root");
        let store = ApprovalStore::open(&root).expect("open store");
        let snapshot = snapshot(b"original");
        let path = store
            .approve(&snapshot, snapshot.fingerprint())
            .expect("record approval");
        fs::write(path, "tampered\n").expect("tamper with record");

        assert_eq!(
            store.status(&snapshot).expect("corrupt status"),
            ApprovalStatus::Invalid(InvalidationReason::ApprovalRecordCorrupt)
        );
        cleanup(&root);
    }

    #[cfg(unix)]
    #[test]
    fn refuses_symlinked_state_directory() {
        use std::os::unix::fs::symlink;

        let root = temporary_root("symlink-root");
        let outside = temporary_root("symlink-outside");
        fs::create_dir(&root).expect("create project root");
        fs::create_dir(&outside).expect("create outside directory");
        symlink(&outside, root.join(".sigillum")).expect("create state symlink");
        let store = ApprovalStore::open(&root).expect("open store");
        let snapshot = snapshot(b"original");

        assert!(matches!(
            store.approve(&snapshot, snapshot.fingerprint()),
            Err(StoreError::UnsafeStatePath(_))
        ));
        cleanup(&root);
        cleanup(&outside);
    }

    fn snapshot(content: &[u8]) -> Snapshot {
        Snapshot::build(
            "add-auth",
            "spec-driven",
            &[ArtifactInput {
                artifact_id: "proposal",
                relative_path: "openspec/changes/add-auth/proposal.md",
                content,
            }],
        )
        .expect("valid snapshot")
    }

    fn temporary_root(label: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock after epoch")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "sigillum-store-{label}-{}-{nonce}",
            std::process::id()
        ))
    }

    fn cleanup(path: &Path) {
        if let Err(error) = fs::remove_dir_all(path) {
            eprintln!("failed to remove fixture {}: {error}", path.display());
        }
    }
}
