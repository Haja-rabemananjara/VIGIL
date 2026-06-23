use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    Observer,
    Responder,
    Manager,
}

impl Role {
    pub fn as_str(self) -> &'static str {
        match self {
            Role::Observer => "observer",
            Role::Responder => "responder",
            Role::Manager => "manager",
        }
    }

    pub fn from_db(s: &str) -> Option<Self> {
        match s {
            "observer" => Some(Role::Observer),
            "responder" => Some(Role::Responder),
            "manager" => Some(Role::Manager),
            _ => None,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct TeamView {
    pub id: Uuid,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub role: Role,
}

pub fn validate_team_name(raw: &str) -> Result<String, &'static str> {
    let name = raw.trim();
    if name.is_empty() {
        return Err("team name must not be empty");
    }
    if name.chars().count() > 100 {
        return Err("team name must be at most 100 characters");
    }
    Ok(name.to_string())
}

impl Role {
    pub fn level(self) -> u8 {
        match self {
            Role::Observer => 0,
            Role::Responder => 1,
            Role::Manager => 2,
        }
    }

    pub fn has_at_least(self, required: Role) -> bool {
        self.level() >= required.level()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn role_hierarchy() {
        assert!(Role::Manager.has_at_least(Role::Observer));
        assert!(Role::Manager.has_at_least(Role::Responder));
        assert!(Role::Manager.has_at_least(Role::Manager));

        assert!(Role::Responder.has_at_least(Role::Observer));
        assert!(Role::Responder.has_at_least(Role::Responder));
        assert!(!Role::Responder.has_at_least(Role::Manager));

        assert!(Role::Observer.has_at_least(Role::Observer));
        assert!(!Role::Observer.has_at_least(Role::Responder));
        assert!(!Role::Observer.has_at_least(Role::Manager));
    }
}
