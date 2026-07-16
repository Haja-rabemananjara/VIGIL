use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ServiceName {
    Github,
    Gitlab,
    Discord,
}

impl ServiceName {
    pub fn as_str(&self) -> &'static str {
        match self {
            ServiceName::Github => "github",
            ServiceName::Gitlab => "gitlab",
            ServiceName::Discord => "discord",
        }
    }

    pub fn from_db(s: &str) -> Option<Self> {
        match s {
            "github" => Some(ServiceName::Github),
            "gitlab" => Some(ServiceName::Gitlab),
            "discord" => Some(ServiceName::Discord),
            _ => None,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ServiceConnection {
    pub id: Uuid,
    pub service: ServiceName,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug)]
pub struct ServiceConnectionWithToken {
    pub id: Uuid,
    pub user_id: Uuid,
    pub service: ServiceName,
    pub encrypted_token: Vec<u8>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
