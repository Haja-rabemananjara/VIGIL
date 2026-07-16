use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WsEvent {
    Connected {
        user_id: Uuid,
    },

    IncidentStateChanged {
        team_id: Uuid,
        incident_id: Uuid,
        new_state: String,
        by: Uuid,
    },

    IncidentEscalated {
        team_id: Uuid,
        incident_id: Uuid,
        new_severity: String,
        by: Uuid,
    },

    IncidentAssigned {
        team_id: Uuid,
        incident_id: Uuid,
        assigned_to: Uuid,
        by: Uuid,
    },

    TimelineEntryAdded {
        team_id: Uuid,
        incident_id: Uuid,
        entry_id: Uuid,
        author_id: Uuid,
        content: String,
        at: i64,
    },

    PresenceUpdate {
        team_id: Uuid,
        resource_type: String,
        resource_id: Uuid,
        watchers: Vec<Uuid>,
    },

    MemberRoleChanged {
        team_id: Uuid,
        user_id: Uuid,
        new_role: String,
        by: Uuid,
    },

    ReleaseStateChanged {
        team_id: Uuid,
        release_id: Uuid,
        new_state: String,
    },

    ReleaseIncidentLinked {
        team_id: Uuid,
        release_id: Uuid,
        incident_id: Uuid,
    },

    ReleaseIncidentUnlinked {
        team_id: Uuid,
        release_id: Uuid,
        incident_id: Uuid,
    },

    ReleaseStepValidated {
        team_id: Uuid,
        release_id: Uuid,
        step_id: Uuid,
        step_name: String,
        by: Uuid,
    },

    RuleTriggered {
        team_id: Uuid,
        rule_id: Uuid,
        rule_name: String,
        reaction_type: String,
    },
    RuleFailed {
        team_id: Uuid,
        rule_id: Uuid,
        rule_name: String,
        reaction_type: String,
        error: String,
    },
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WsClientMessage {
    Watch {
        resource_type: String,
        resource_id: Uuid,
        team_id: Uuid,
    },
    Unwatch {
        resource_type: String,
        resource_id: Uuid,
        team_id: Uuid,
    },
}
