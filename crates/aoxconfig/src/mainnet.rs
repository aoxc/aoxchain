// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use serde::{Deserialize, Serialize};

/// Nine mandatory tracks to move AOXC from testnet maturity into mainnet launch quality.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MainnetTrack {
    Governance,
    ConsensusSafety,
    NetworkResilience,
    StorageState,
    SmartContractSecurity,
    Observability,
    DevExTooling,
    IncidentResponse,
    ReleaseControls,
}

impl MainnetTrack {
    /// Returns all tracks in deterministic order for reporting and scoring.
    pub fn all() -> [Self; 9] {
        [
            Self::Governance,
            Self::ConsensusSafety,
            Self::NetworkResilience,
            Self::StorageState,
            Self::SmartContractSecurity,
            Self::Observability,
            Self::DevExTooling,
            Self::IncidentResponse,
            Self::ReleaseControls,
        ]
    }

    /// Human-readable label used in reports and dashboards.
    pub fn label(self) -> &'static str {
        match self {
            Self::Governance => "governance",
            Self::ConsensusSafety => "consensus_safety",
            Self::NetworkResilience => "network_resilience",
            Self::StorageState => "storage_state",
            Self::SmartContractSecurity => "smart_contract_security",
            Self::Observability => "observability",
            Self::DevExTooling => "devex_tooling",
            Self::IncidentResponse => "incident_response",
            Self::ReleaseControls => "release_controls",
        }
    }
}

/// Delivery state for each track.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrackStatus {
    Planned,
    Building,
    Hardening,
    Ready,
}

impl TrackStatus {
    /// Numeric completion used for weighted readiness scoring.
    pub fn completion(self) -> u8 {
        match self {
            Self::Planned => 25,
            Self::Building => 55,
            Self::Hardening => 80,
            Self::Ready => 100,
        }
    }
}

/// One roadmap item in the mainnet program.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MainnetMilestone {
    pub track: MainnetTrack,
    pub owner: String,
    pub status: TrackStatus,
    pub gate: String,
}

impl MainnetMilestone {
    pub fn validate(&self) -> Result<(), String> {
        if self.owner.trim().is_empty() {
            return Err(format!("{}: owner must not be empty", self.track.label()));
        }
        if self.gate.trim().is_empty() {
            return Err(format!("{}: gate must not be empty", self.track.label()));
        }
        Ok(())
    }
}

/// Configurable nine-track roadmap to drive mainnet readiness.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MainnetProgram {
    pub min_readiness_score: u8,
    pub milestones: Vec<MainnetMilestone>,
}

impl MainnetProgram {
    /// Computes weighted completion as an integer from 0 to 100.
    pub fn readiness_score(&self) -> u8 {
        if self.milestones.is_empty() {
            return 0;
        }

        let total: u32 = self
            .milestones
            .iter()
            .map(|m| u32::from(m.status.completion()))
            .sum();

        (total / self.milestones.len() as u32) as u8
    }

    /// Returns whether the program can be considered mainnet-ready.
    pub fn is_mainnet_ready(&self) -> bool {
        self.readiness_score() >= self.min_readiness_score
            && self
                .milestones
                .iter()
                .all(|m| matches!(m.status, TrackStatus::Hardening | TrackStatus::Ready))
    }

    /// Validate the roadmap integrity and return all violations.
    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();

        if self.min_readiness_score < 80 || self.min_readiness_score > 100 {
            errors.push("mainnet.min_readiness_score must be between 80 and 100".to_string());
        }

        if self.milestones.len() != MainnetTrack::all().len() {
            errors.push("mainnet.milestones must include exactly 9 tracks".to_string());
        }

        for track in MainnetTrack::all() {
            let count = self.milestones.iter().filter(|m| m.track == track).count();

            if count == 0 {
                errors.push(format!("mainnet missing required track: {}", track.label()));
            } else if count > 1 {
                errors.push(format!(
                    "mainnet contains duplicate track entries: {}",
                    track.label()
                ));
            }
        }

        errors.extend(self.milestones.iter().filter_map(|m| m.validate().err()));
        errors
    }
}

impl Default for MainnetProgram {
    fn default() -> Self {
        let milestones = MainnetTrack::all()
            .into_iter()
            .map(|track| MainnetMilestone {
                track,
                owner: "core-team".to_string(),
                status: TrackStatus::Planned,
                gate: format!("{}_gate", track.label()),
            })
            .collect();

        Self {
            min_readiness_score: 90,
            milestones,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{MainnetProgram, MainnetTrack, TrackStatus};

    #[test]
    fn default_program_is_valid() {
        let program = MainnetProgram::default();
        assert!(program.validate().is_empty());
        assert_eq!(program.readiness_score(), 25);
        assert!(!program.is_mainnet_ready());
    }

    #[test]
    fn readiness_requires_all_tracks_hardening_or_ready() {
        let mut program = MainnetProgram::default();
        for milestone in &mut program.milestones {
            milestone.status = TrackStatus::Ready;
        }

        assert!(program.is_mainnet_ready());
        assert_eq!(program.readiness_score(), 100);

        let governance = program
            .milestones
            .iter_mut()
            .find(|m| m.track == MainnetTrack::Governance)
            .expect("governance track exists");
        governance.status = TrackStatus::Building;

        assert!(!program.is_mainnet_ready());
    }

    #[test]
    fn validate_rejects_missing_track() {
        let mut program = MainnetProgram::default();
        program.milestones.pop();

        let errs = program.validate();
        assert!(errs.iter().any(|e| e.contains("exactly 9 tracks")));
        assert!(errs.iter().any(|e| e.contains("missing required track")));
    }
}
