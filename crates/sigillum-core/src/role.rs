//! Stable role identities and their minimum capability contracts.

/// Current version of the role-contract schema.
pub const ROLE_CONTRACT_SCHEMA_VERSION: u32 = 1;

/// Logical responsibility within a Sigillum run.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Role {
    /// Discovers repository evidence and prepares bounded context.
    Scout,
    /// Clarifies and reviews plans using brokered context.
    Architect,
    /// Applies one approved task within its execution scope.
    Implementer,
    /// Runs deterministic gates and reports contract findings.
    Verifier,
    /// Produces the final verdict from immutable evidence.
    Judge,
}

impl Role {
    /// Returns the stable serialized role identifier.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Scout => "scout",
            Self::Architect => "architect",
            Self::Implementer => "implementer",
            Self::Verifier => "verifier",
            Self::Judge => "judge",
        }
    }

    /// Returns the built-in minimum contract for this role.
    #[must_use]
    pub const fn contract(self) -> RoleContract {
        match self {
            Self::Scout => RoleContract::new(self, SCOUT_CAPABILITIES, false),
            Self::Architect => RoleContract::new(self, ARCHITECT_CAPABILITIES, false),
            Self::Implementer => RoleContract::new(self, IMPLEMENTER_CAPABILITIES, false),
            Self::Verifier => RoleContract::new(self, VERIFIER_CAPABILITIES, true),
            Self::Judge => RoleContract::new(self, JUDGE_CAPABILITIES, true),
        }
    }
}

/// An operation that the runtime may expose to a role.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Capability {
    /// Read files in the repository through the discovery boundary.
    ReadRepository,
    /// Search repository content and metadata through the discovery boundary.
    SearchRepository,
    /// Read editable planning artifacts before contract approval.
    ReadPlanningArtifacts,
    /// Emit a bounded context pack with provenance.
    WriteContextPack,
    /// Read a bounded context pack selected by the broker.
    ReadContextPack,
    /// Ask the context broker for specific additional evidence.
    RequestContext,
    /// Read the immutable approved contract.
    ReadContract,
    /// Read the currently assigned approved task.
    ReadApprovedTask,
    /// Read repository paths explicitly scoped to the assigned task.
    ReadTaskPaths,
    /// Write repository paths explicitly scoped to the assigned task.
    WriteTaskPaths,
    /// Run commands explicitly allowed for the assigned task.
    RunApprovedCommands,
    /// Read the implementation diff without modifying it.
    ReadDiff,
    /// Run deterministic gates required by policy.
    RunRequiredGates,
    /// Record structured verification findings.
    WriteFindings,
    /// Read the immutable proofpack assembled for decision.
    ReadProofpack,
    /// Record the final structured verdict.
    WriteVerdict,
}

impl Capability {
    /// Returns the stable serialized capability identifier.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ReadRepository => "read_repository",
            Self::SearchRepository => "search_repository",
            Self::ReadPlanningArtifacts => "read_planning_artifacts",
            Self::WriteContextPack => "write_context_pack",
            Self::ReadContextPack => "read_context_pack",
            Self::RequestContext => "request_context",
            Self::ReadContract => "read_contract",
            Self::ReadApprovedTask => "read_approved_task",
            Self::ReadTaskPaths => "read_task_paths",
            Self::WriteTaskPaths => "write_task_paths",
            Self::RunApprovedCommands => "run_approved_commands",
            Self::ReadDiff => "read_diff",
            Self::RunRequiredGates => "run_required_gates",
            Self::WriteFindings => "write_findings",
            Self::ReadProofpack => "read_proofpack",
            Self::WriteVerdict => "write_verdict",
        }
    }
}

/// Immutable minimum capability policy for one logical role.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RoleContract {
    role: Role,
    capabilities: &'static [Capability],
    requires_independent_session: bool,
}

impl RoleContract {
    const fn new(
        role: Role,
        capabilities: &'static [Capability],
        requires_independent_session: bool,
    ) -> Self {
        Self {
            role,
            capabilities,
            requires_independent_session,
        }
    }

    /// Returns the schema version used to serialize this contract.
    #[must_use]
    pub const fn schema_version(self) -> u32 {
        ROLE_CONTRACT_SCHEMA_VERSION
    }

    /// Returns the logical role governed by this contract.
    #[must_use]
    pub const fn role(self) -> Role {
        self.role
    }

    /// Returns the complete minimum capability allowlist.
    #[must_use]
    pub const fn capabilities(self) -> &'static [Capability] {
        self.capabilities
    }

    /// Returns whether the role must not share the implementer's mutable session.
    #[must_use]
    pub const fn requires_independent_session(self) -> bool {
        self.requires_independent_session
    }

    /// Returns whether the capability is explicitly allowed.
    #[must_use]
    pub fn allows(self, capability: Capability) -> bool {
        self.capabilities.contains(&capability)
    }
}

const SCOUT_CAPABILITIES: &[Capability] = &[
    Capability::ReadRepository,
    Capability::SearchRepository,
    Capability::ReadPlanningArtifacts,
    Capability::WriteContextPack,
];
const ARCHITECT_CAPABILITIES: &[Capability] = &[
    Capability::ReadContextPack,
    Capability::RequestContext,
    Capability::ReadContract,
];
const IMPLEMENTER_CAPABILITIES: &[Capability] = &[
    Capability::ReadContextPack,
    Capability::ReadContract,
    Capability::ReadApprovedTask,
    Capability::ReadTaskPaths,
    Capability::WriteTaskPaths,
    Capability::RunApprovedCommands,
];
const VERIFIER_CAPABILITIES: &[Capability] = &[
    Capability::ReadContract,
    Capability::ReadContextPack,
    Capability::ReadDiff,
    Capability::RunRequiredGates,
    Capability::WriteFindings,
];
const JUDGE_CAPABILITIES: &[Capability] = &[
    Capability::ReadContract,
    Capability::ReadProofpack,
    Capability::WriteVerdict,
];

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::{Capability, Role, ROLE_CONTRACT_SCHEMA_VERSION};

    const ROLES: [Role; 5] = [
        Role::Scout,
        Role::Architect,
        Role::Implementer,
        Role::Verifier,
        Role::Judge,
    ];

    #[test]
    fn all_role_identities_and_capabilities_are_stable_and_unique() {
        let names = ROLES.map(Role::as_str).into_iter().collect::<HashSet<_>>();
        assert_eq!(names.len(), ROLES.len());

        for role in ROLES {
            let contract = role.contract();
            assert_eq!(contract.schema_version(), ROLE_CONTRACT_SCHEMA_VERSION);
            assert_eq!(contract.role(), role);
            assert!(!contract.capabilities().is_empty());
            assert_eq!(
                contract.capabilities().iter().collect::<HashSet<_>>().len(),
                contract.capabilities().len()
            );
        }
    }

    #[test]
    fn non_implementers_cannot_write_task_paths_or_run_task_commands() {
        for role in [Role::Scout, Role::Architect, Role::Verifier, Role::Judge] {
            assert!(!role.contract().allows(Capability::WriteTaskPaths));
            assert!(!role.contract().allows(Capability::RunApprovedCommands));
        }
        assert!(Role::Implementer
            .contract()
            .allows(Capability::WriteTaskPaths));
        assert!(Role::Implementer
            .contract()
            .allows(Capability::RunApprovedCommands));
    }

    #[test]
    fn architect_has_brokered_context_without_repository_crawl() {
        let architect = Role::Architect.contract();

        assert!(architect.allows(Capability::ReadContextPack));
        assert!(architect.allows(Capability::RequestContext));
        assert!(!architect.allows(Capability::ReadRepository));
        assert!(!architect.allows(Capability::SearchRepository));
    }

    #[test]
    fn verifier_and_judge_require_independent_sessions() {
        assert!(Role::Verifier.contract().requires_independent_session());
        assert!(Role::Judge.contract().requires_independent_session());
        assert!(!Role::Implementer.contract().requires_independent_session());
    }
}
