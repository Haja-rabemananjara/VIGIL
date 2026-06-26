use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IncidentStatus {
    Open,
    Acknowledged,
    Escalated,
    Resolved,
}

impl TryFrom<&str> for IncidentStatus {
    type Error = String;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s {
            "open" => Ok(Self::Open),
            "acknowledged" => Ok(Self::Acknowledged),
            "escalated" => Ok(Self::Escalated),
            "resolved" => Ok(Self::Resolved),
            other => Err(format!("unknown incident status: {other}")),
        }
    }
}

impl fmt::Display for IncidentStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Open => "open",
            Self::Acknowledged => "acknowledged",
            Self::Escalated => "escalated",
            Self::Resolved => "resolved",
        };
        write!(f, "{s}")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IncidentSeverity {
    Low,
    Medium,
    High,
    Critical,
}

impl TryFrom<&str> for IncidentSeverity {
    type Error = String;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s {
            "low" => Ok(Self::Low),
            "medium" => Ok(Self::Medium),
            "high" => Ok(Self::High),
            "critical" => Ok(Self::Critical),
            other => Err(format!("unknown incident severity: {other}")),
        }
    }
}

impl fmt::Display for IncidentSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
            Self::Critical => "critical",
        };
        write!(f, "{s}")
    }
}

pub fn can_transition(from: &IncidentStatus, to: &IncidentStatus) -> bool {
    matches!(
        (from, to),
        (IncidentStatus::Open, IncidentStatus::Acknowledged)
            | (IncidentStatus::Acknowledged, IncidentStatus::Escalated)
            | (IncidentStatus::Acknowledged, IncidentStatus::Resolved)
            | (IncidentStatus::Escalated, IncidentStatus::Resolved)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_transitions() {
        assert!(can_transition(
            &IncidentStatus::Open,
            &IncidentStatus::Acknowledged
        ));
        assert!(can_transition(
            &IncidentStatus::Acknowledged,
            &IncidentStatus::Escalated
        ));
        assert!(can_transition(
            &IncidentStatus::Acknowledged,
            &IncidentStatus::Resolved
        ));
        assert!(can_transition(
            &IncidentStatus::Escalated,
            &IncidentStatus::Resolved
        ));
    }

    #[test]
    fn invalid_transitions() {
        assert!(!can_transition(
            &IncidentStatus::Open,
            &IncidentStatus::Escalated
        ));
        assert!(!can_transition(
            &IncidentStatus::Open,
            &IncidentStatus::Resolved
        ));

        assert!(!can_transition(
            &IncidentStatus::Acknowledged,
            &IncidentStatus::Open
        ));
        assert!(!can_transition(
            &IncidentStatus::Resolved,
            &IncidentStatus::Open
        ));

        assert!(!can_transition(
            &IncidentStatus::Open,
            &IncidentStatus::Open
        ));

        assert!(!can_transition(
            &IncidentStatus::Escalated,
            &IncidentStatus::Acknowledged
        ));
    }

    #[test]
    fn status_roundtrip() {
        for s in ["open", "acknowledged", "escalated", "resolved"] {
            let status = IncidentStatus::try_from(s).unwrap();
            assert_eq!(status.to_string(), s);
        }
    }

    #[test]
    fn severity_roundtrip() {
        for s in ["low", "medium", "high", "critical"] {
            let sev = IncidentSeverity::try_from(s).unwrap();
            assert_eq!(sev.to_string(), s);
        }
    }
}
