//! Whether FastCtx is currently connected to the host, as the control terminal reports it.
//!
//! Apply writes three things: the stable binary, the `mcp_servers` entry, and the marker block in
//! the host's `AGENTS.md`. Product updates may refresh one byte-frozen legacy block, but that
//! deliberately does not complete a new Apply. This module keeps the main menu aligned with the
//! more detailed Status diagnosis without hiding that remaining action.

use std::fs;

use crate::control::agents;
use crate::control::paths::ControlPaths;
use crate::control::settings::AppliedRecord;

/// How the connection to the host compares to the running build.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LinkState {
    /// Apply has never recorded a connection.
    Absent,
    /// The managed guidance block matches what this build writes.
    Current,
    /// Current bytes were found, but the receipt does not record the current Apply contract.
    ApplyRequired,
    /// An exact managed block from a superseded release remains on disk.
    KnownLegacy,
    /// An Apply receipt exists, but the managed block or file is absent.
    Missing,
    /// The marker pair is valid, but the managed bytes were changed or use another shell shape.
    Drifted,
    /// The file is not UTF-8 or its managed markers are structurally damaged.
    Malformed,
    /// The AGENTS file exists but could not be read.
    Unreadable,
}

impl LinkState {
    /// Whether the visible connection still needs an explicit Apply before a Codex restart.
    pub(crate) const fn requires_apply(self) -> bool {
        !matches!(self, Self::Absent | Self::Current)
    }
}

/// Classifies the recorded connection against the guidance this build would write.
///
/// The block is compared with the receipt's shell state rather than the currently saved one so
/// this agrees with the `AGENTS guidance` doctor check; a pending extension-tool change is a
/// separate matter that the config page already explains.
pub fn link_state(paths: &ControlPaths, applied: Option<&AppliedRecord>) -> LinkState {
    let Some(record) = applied else {
        return LinkState::Absent;
    };
    match fs::read(&paths.codex_agents) {
        Ok(bytes) => match agents::classify_managed_section(&bytes, record.fastshell_enabled) {
            agents::ManagedSectionState::Current
                if record.agents_contract_id.as_deref()
                    == Some(agents::MANAGED_SECTION_CONTRACT_ID) =>
            {
                LinkState::Current
            }
            agents::ManagedSectionState::Current => LinkState::ApplyRequired,
            agents::ManagedSectionState::KnownLegacy => LinkState::KnownLegacy,
            agents::ManagedSectionState::Missing => LinkState::Missing,
            agents::ManagedSectionState::Drifted => LinkState::Drifted,
            agents::ManagedSectionState::Malformed(_) => LinkState::Malformed,
        },
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => LinkState::Missing,
        Err(_) => LinkState::Unreadable,
    }
}
