use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReleaseStatus {
    Created,
    InProgress,
    Completed,
    Cancelled,
    Blocked,
}

impl ReleaseStatus {
    pub fn from_db(s: &str) -> Option<Self> {
        match s {
            "created" => Some(Self::Created),
            "in_progress" => Some(Self::InProgress),
            "completed" => Some(Self::Completed),
            "cancelled" => Some(Self::Cancelled),
            "blocked" => Some(Self::Blocked),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
            Self::Blocked => "blocked",
        }
    }

    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Completed | Self::Cancelled)
    }

    pub fn can_cancel(&self) -> bool {
        matches!(self, Self::Created | Self::InProgress | Self::Blocked)
    }
}

pub fn can_transition(from: &ReleaseStatus, to: &ReleaseStatus) -> bool {
    matches!(
        (from, to),
        (ReleaseStatus::Created, ReleaseStatus::InProgress)
            | (ReleaseStatus::Created, ReleaseStatus::Cancelled)
            | (ReleaseStatus::InProgress, ReleaseStatus::Completed)
            | (ReleaseStatus::InProgress, ReleaseStatus::Cancelled)
            | (ReleaseStatus::InProgress, ReleaseStatus::Blocked)
            | (ReleaseStatus::Blocked, ReleaseStatus::InProgress)
            | (ReleaseStatus::Blocked, ReleaseStatus::Cancelled)
    )
}

pub fn can_validate_step(step_position: i32, all_steps: &[ReleaseStepRow]) -> Result<(), String> {
    for s in all_steps {
        if s.position < step_position && s.validated_by.is_none() {
            return Err(format!(
                "Step '{}' (position {}) must be validated first",
                s.name, s.position
            ));
        }
    }
    Ok(())
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ReleaseRow {
    pub id: Uuid,
    pub team_id: Uuid,
    pub title: String,
    pub body: String,
    pub status: String,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub cancelled_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ReleaseStepRow {
    pub id: Uuid,
    pub release_id: Uuid,
    pub name: String,
    pub position: i32,
    pub validated_by: Option<Uuid>,
    pub validated_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct ReleaseResponse {
    pub id: Uuid,
    pub team_id: Uuid,
    pub title: String,
    pub body: String,
    pub status: ReleaseStatus,
    pub created_by: Uuid,
    pub created_at: i64,
    pub updated_at: i64,
    pub started_at: Option<i64>,
    pub completed_at: Option<i64>,
    pub cancelled_at: Option<i64>,
    pub steps: Vec<ReleaseStepResponse>,
    pub progress: ReleaseProgress,
    pub linked_incidents: Vec<LinkedIncidentInfo>,
}

#[derive(Debug, Serialize)]
pub struct ReleaseStepResponse {
    pub id: Uuid,
    pub name: String,
    pub position: i32,
    pub validated_by: Option<Uuid>,
    pub validated_at: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct ReleaseProgress {
    pub completed: usize,
    pub total: usize,
}

#[derive(Debug, Serialize)]
pub struct LinkedIncidentInfo {
    pub id: Uuid,
    pub title: String,
    pub status: String,
    pub severity: String,
}

impl ReleaseResponse {
    pub fn from_row(
        row: ReleaseRow,
        step_rows: Vec<ReleaseStepRow>,
        linked_incidents: Vec<LinkedIncidentInfo>,
    ) -> Self {
        let status = ReleaseStatus::from_db(&row.status).unwrap_or(ReleaseStatus::Created);

        let steps: Vec<ReleaseStepResponse> = step_rows
            .iter()
            .map(|s| ReleaseStepResponse {
                id: s.id,
                name: s.name.clone(),
                position: s.position,
                validated_by: s.validated_by,
                validated_at: s.validated_at.map(|dt| dt.timestamp()),
            })
            .collect();

        let progress = ReleaseProgress {
            completed: steps.iter().filter(|s| s.validated_by.is_some()).count(),
            total: steps.len(),
        };

        Self {
            id: row.id,
            team_id: row.team_id,
            title: row.title,
            body: row.body,
            status,
            created_by: row.created_by,
            created_at: row.created_at.timestamp(),
            updated_at: row.updated_at.timestamp(),
            started_at: row.started_at.map(|dt| dt.timestamp()),
            completed_at: row.completed_at.map(|dt| dt.timestamp()),
            cancelled_at: row.cancelled_at.map(|dt| dt.timestamp()),
            steps,
            progress,
            linked_incidents,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ReleaseListItem {
    pub id: Uuid,
    pub title: String,
    pub status: ReleaseStatus,
    pub created_by: Uuid,
    pub created_at: i64,
    pub updated_at: i64,
    pub progress: ReleaseProgress,
}

impl ReleaseListItem {
    pub fn from_row(row: ReleaseRow, step_rows: &[ReleaseStepRow]) -> Self {
        let status = ReleaseStatus::from_db(&row.status).unwrap_or(ReleaseStatus::Created);

        Self {
            id: row.id,
            title: row.title,
            status,
            created_by: row.created_by,
            created_at: row.created_at.timestamp(),
            updated_at: row.updated_at.timestamp(),
            progress: ReleaseProgress {
                completed: step_rows
                    .iter()
                    .filter(|s| s.validated_by.is_some())
                    .count(),
                total: step_rows.len(),
            },
        }
    }
}
