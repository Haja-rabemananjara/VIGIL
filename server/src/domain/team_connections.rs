use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct TeamServiceConnection {
    pub id: Uuid,
    pub team_id: Uuid,
    pub service: String,
    pub created_by: Uuid,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug)]
pub struct TeamConnectionWithToken {
    pub id: Uuid,
    pub team_id: Uuid,
    pub service: String,
    pub encrypted_token: Vec<u8>,
}
