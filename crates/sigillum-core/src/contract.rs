//! Canonical contract snapshots built from resolved OpenSpec artifacts.

use std::collections::HashSet;
use std::error::Error;
use std::fmt;

use crate::digest::sha256_hex;

/// Current version of the canonical contract snapshot format.
pub const CONTRACT_SNAPSHOT_SCHEMA_VERSION: u32 = 1;

/// Borrowed OpenSpec artifact content supplied by a planning adapter.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ArtifactInput<'a> {
    /// Artifact identifier from the active OpenSpec schema.
    pub artifact_id: &'a str,
    /// Slash-separated path relative to the OpenSpec planning root.
    pub relative_path: &'a str,
    /// Exact artifact bytes read by the planning adapter.
    pub content: &'a [u8],
}

/// Content-addressed reference stored in a contract snapshot.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArtifactReference {
    artifact_id: String,
    relative_path: String,
    content_sha256: String,
}

impl ArtifactReference {
    /// Returns the artifact identifier from the OpenSpec schema.
    #[must_use]
    pub fn artifact_id(&self) -> &str {
        &self.artifact_id
    }

    /// Returns the canonical slash-separated artifact path.
    #[must_use]
    pub fn relative_path(&self) -> &str {
        &self.relative_path
    }

    /// Returns the lowercase SHA-256 digest of the exact artifact bytes.
    #[must_use]
    pub fn content_sha256(&self) -> &str {
        &self.content_sha256
    }
}

/// Immutable, canonical view of the OpenSpec artifact closure at approval time.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContractSnapshot {
    schema_version: u32,
    change_id: String,
    openspec_schema: String,
    artifacts: Vec<ArtifactReference>,
    fingerprint: String,
}

impl ContractSnapshot {
    /// Builds and fingerprints a snapshot from a fully resolved artifact closure.
    ///
    /// The caller is responsible for resolving readiness and dependency edges from
    /// OpenSpec. Sigillum validates identity, path safety, and uniqueness before
    /// sorting the closure into its canonical order.
    ///
    /// # Errors
    ///
    /// Returns [`SnapshotError`] when an identifier or path is invalid, the
    /// closure is empty, or artifact identities collide.
    pub fn build(
        change_id: &str,
        openspec_schema: &str,
        artifacts: &[ArtifactInput<'_>],
    ) -> Result<Self, SnapshotError> {
        validate_identifier("change_id", change_id)?;
        validate_identifier("openspec_schema", openspec_schema)?;
        if artifacts.is_empty() {
            return Err(SnapshotError::EmptyArtifactClosure);
        }

        let mut artifact_ids_and_paths = HashSet::with_capacity(artifacts.len());
        let mut paths = HashSet::with_capacity(artifacts.len());
        let mut references = Vec::with_capacity(artifacts.len());

        for artifact in artifacts {
            validate_identifier("artifact_id", artifact.artifact_id)?;
            validate_relative_path(artifact.relative_path)?;

            let identity = (artifact.artifact_id, artifact.relative_path);
            if !artifact_ids_and_paths.insert(identity) {
                return Err(SnapshotError::DuplicateArtifact {
                    artifact_id: artifact.artifact_id.to_owned(),
                    relative_path: artifact.relative_path.to_owned(),
                });
            }
            if !paths.insert(artifact.relative_path) {
                return Err(SnapshotError::DuplicatePath(
                    artifact.relative_path.to_owned(),
                ));
            }

            references.push(ArtifactReference {
                artifact_id: artifact.artifact_id.to_owned(),
                relative_path: artifact.relative_path.to_owned(),
                content_sha256: sha256_hex(artifact.content),
            });
        }

        references.sort_by(|left, right| {
            left.artifact_id
                .cmp(&right.artifact_id)
                .then_with(|| left.relative_path.cmp(&right.relative_path))
        });

        let mut snapshot = Self {
            schema_version: CONTRACT_SNAPSHOT_SCHEMA_VERSION,
            change_id: change_id.to_owned(),
            openspec_schema: openspec_schema.to_owned(),
            artifacts: references,
            fingerprint: String::new(),
        };
        snapshot.fingerprint = sha256_hex(&snapshot.canonical_bytes());
        Ok(snapshot)
    }

    /// Returns the snapshot schema version.
    #[must_use]
    pub const fn schema_version(&self) -> u32 {
        self.schema_version
    }

    /// Returns the OpenSpec change identifier.
    #[must_use]
    pub fn change_id(&self) -> &str {
        &self.change_id
    }

    /// Returns the OpenSpec workflow schema identifier.
    #[must_use]
    pub fn openspec_schema(&self) -> &str {
        &self.openspec_schema
    }

    /// Returns artifact references in canonical order.
    #[must_use]
    pub fn artifacts(&self) -> &[ArtifactReference] {
        &self.artifacts
    }

    /// Returns the lowercase SHA-256 digest of the canonical snapshot.
    #[must_use]
    pub fn fingerprint(&self) -> &str {
        &self.fingerprint
    }

    fn canonical_bytes(&self) -> Vec<u8> {
        let mut output = Vec::new();
        output.extend_from_slice(b"sigillum-contract-snapshot\0");
        append_u32(&mut output, self.schema_version);
        append_field(&mut output, self.change_id.as_bytes());
        append_field(&mut output, self.openspec_schema.as_bytes());
        append_u64(
            &mut output,
            u64::try_from(self.artifacts.len()).expect("artifact count must fit into u64"),
        );
        for artifact in &self.artifacts {
            append_field(&mut output, artifact.artifact_id.as_bytes());
            append_field(&mut output, artifact.relative_path.as_bytes());
            append_field(&mut output, artifact.content_sha256.as_bytes());
        }
        output
    }
}

/// Hash-pinned approval tied to exactly one contract snapshot.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContractApproval {
    contract_fingerprint: String,
}

impl ContractApproval {
    /// Creates approval for the exact supplied snapshot.
    #[must_use]
    pub fn for_snapshot(snapshot: &ContractSnapshot) -> Self {
        Self {
            contract_fingerprint: snapshot.fingerprint.clone(),
        }
    }

    /// Returns whether this approval still matches the supplied snapshot.
    #[must_use]
    pub fn is_valid_for(&self, snapshot: &ContractSnapshot) -> bool {
        self.contract_fingerprint == snapshot.fingerprint
    }

    /// Returns the approved contract fingerprint.
    #[must_use]
    pub fn contract_fingerprint(&self) -> &str {
        &self.contract_fingerprint
    }
}

/// Validation error produced while constructing a contract snapshot.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SnapshotError {
    /// The resolved OpenSpec closure contained no files.
    EmptyArtifactClosure,
    /// An identifier was empty or used unsupported characters.
    InvalidIdentifier {
        /// Name of the invalid field.
        field: &'static str,
        /// Rejected value.
        value: String,
    },
    /// An artifact path was absolute, non-canonical, or traversed upward.
    InvalidRelativePath(String),
    /// The same artifact identifier and path appeared more than once.
    DuplicateArtifact {
        /// Duplicate artifact identifier.
        artifact_id: String,
        /// Duplicate relative path.
        relative_path: String,
    },
    /// Two artifact identifiers attempted to claim the same path.
    DuplicatePath(String),
}

impl fmt::Display for SnapshotError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyArtifactClosure => formatter.write_str("artifact closure must not be empty"),
            Self::InvalidIdentifier { field, value } => {
                write!(formatter, "invalid {field}: {value:?}")
            }
            Self::InvalidRelativePath(path) => {
                write!(formatter, "invalid relative artifact path: {path:?}")
            }
            Self::DuplicateArtifact {
                artifact_id,
                relative_path,
            } => write!(
                formatter,
                "duplicate artifact {artifact_id:?} at {relative_path:?}"
            ),
            Self::DuplicatePath(path) => write!(formatter, "duplicate artifact path: {path:?}"),
        }
    }
}

impl Error for SnapshotError {}

fn validate_identifier(field: &'static str, value: &str) -> Result<(), SnapshotError> {
    let valid = !value.is_empty()
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase()
                || byte.is_ascii_digit()
                || matches!(byte, b'-' | b'_' | b'.')
        });
    if valid {
        Ok(())
    } else {
        Err(SnapshotError::InvalidIdentifier {
            field,
            value: value.to_owned(),
        })
    }
}

fn validate_relative_path(path: &str) -> Result<(), SnapshotError> {
    let valid = !path.is_empty()
        && !path.starts_with('/')
        && !path.contains('\\')
        && path
            .split('/')
            .all(|component| !component.is_empty() && !matches!(component, "." | ".."));
    if valid {
        Ok(())
    } else {
        Err(SnapshotError::InvalidRelativePath(path.to_owned()))
    }
}

fn append_field(output: &mut Vec<u8>, value: &[u8]) {
    append_u64(
        output,
        u64::try_from(value.len()).expect("field length must fit into u64"),
    );
    output.extend_from_slice(value);
}

fn append_u32(output: &mut Vec<u8>, value: u32) {
    output.extend_from_slice(&value.to_be_bytes());
}

fn append_u64(output: &mut Vec<u8>, value: u64) {
    output.extend_from_slice(&value.to_be_bytes());
}

#[cfg(test)]
mod tests {
    use super::{
        ArtifactInput, ContractApproval, ContractSnapshot, SnapshotError,
        CONTRACT_SNAPSHOT_SCHEMA_VERSION,
    };

    const PROPOSAL: ArtifactInput<'_> = ArtifactInput {
        artifact_id: "proposal",
        relative_path: "changes/add-auth/proposal.md",
        content: b"# Proposal\n",
    };
    const TASKS: ArtifactInput<'_> = ArtifactInput {
        artifact_id: "tasks",
        relative_path: "changes/add-auth/tasks.md",
        content: b"- [ ] Implement auth\n",
    };

    #[test]
    fn snapshot_is_independent_of_adapter_order() {
        let first = ContractSnapshot::build("add-auth", "spec-driven", &[PROPOSAL, TASKS])
            .expect("valid snapshot");
        let second = ContractSnapshot::build("add-auth", "spec-driven", &[TASKS, PROPOSAL])
            .expect("valid snapshot");

        assert_eq!(first, second);
        assert_eq!(first.schema_version(), CONTRACT_SNAPSHOT_SCHEMA_VERSION);
        assert_eq!(first.artifacts()[0].artifact_id(), "proposal");
        assert_eq!(first.artifacts()[1].artifact_id(), "tasks");
    }

    #[test]
    fn artifact_edit_invalidates_approval() {
        let original = ContractSnapshot::build("add-auth", "spec-driven", &[PROPOSAL, TASKS])
            .expect("valid snapshot");
        let approval = ContractApproval::for_snapshot(&original);
        let changed_tasks = ArtifactInput {
            content: b"- [ ] Implement stronger auth\n",
            ..TASKS
        };
        let changed =
            ContractSnapshot::build("add-auth", "spec-driven", &[PROPOSAL, changed_tasks])
                .expect("valid changed snapshot");

        assert!(approval.is_valid_for(&original));
        assert!(!approval.is_valid_for(&changed));
        assert_ne!(original.fingerprint(), changed.fingerprint());
    }

    #[test]
    fn rejects_non_canonical_or_unsafe_paths() {
        for path in [
            "",
            "/proposal.md",
            "../proposal.md",
            "changes/../proposal.md",
            "changes\\proposal.md",
            "changes//proposal.md",
        ] {
            let invalid = ArtifactInput {
                relative_path: path,
                ..PROPOSAL
            };
            assert!(matches!(
                ContractSnapshot::build("add-auth", "spec-driven", &[invalid]),
                Err(SnapshotError::InvalidRelativePath(rejected)) if rejected == path
            ));
        }
    }

    #[test]
    fn rejects_duplicate_paths_even_with_different_ids() {
        let duplicate = ArtifactInput {
            artifact_id: "design",
            ..PROPOSAL
        };

        assert_eq!(
            ContractSnapshot::build("add-auth", "spec-driven", &[PROPOSAL, duplicate]),
            Err(SnapshotError::DuplicatePath(
                "changes/add-auth/proposal.md".to_owned()
            ))
        );
    }

    #[test]
    fn records_known_content_digest() {
        let snapshot = ContractSnapshot::build(
            "digest-check",
            "spec-driven",
            &[ArtifactInput {
                artifact_id: "proposal",
                relative_path: "changes/digest-check/proposal.md",
                content: b"abc",
            }],
        )
        .expect("valid snapshot");

        assert_eq!(
            snapshot.artifacts()[0].content_sha256(),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }
}
