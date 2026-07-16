use std::collections::BTreeMap;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::domain::service_connections::ServiceName;
use crate::hooks::{ActionCatalog, ReactionRegistry};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct AboutResponse {
    pub client: ClientInfo,
    pub server: ServerInfo,
}

#[derive(Debug, Serialize)]
pub struct ClientInfo {
    pub host: String,
}

#[derive(Debug, Serialize)]
pub struct ServerInfo {
    pub current_time: i64,
    pub services: Vec<ServiceCatalog>,
    pub token: String,
}

#[derive(Debug, Serialize)]
pub struct ServiceCatalog {
    pub name: String,
    pub connectable: bool,
    pub actions: Vec<CatalogEntry>,
    pub reactions: Vec<CatalogEntry>,
}

#[derive(Debug, Serialize)]
pub struct CatalogEntry {
    pub name: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload_example: Option<String>,
}

pub fn build_response(
    client_host: String,
    catalog: &ActionCatalog,
    registry: &ReactionRegistry,
    kickoff_token: String,
) -> AboutResponse {
    let services = group_by_service(catalog, registry);

    AboutResponse {
        client: ClientInfo { host: client_host },
        server: ServerInfo {
            current_time: current_unix_time(),
            services,
            token: kickoff_token,
        },
    }
}

fn group_by_service(catalog: &ActionCatalog, registry: &ReactionRegistry) -> Vec<ServiceCatalog> {
    let mut services: BTreeMap<String, ServiceCatalog> = BTreeMap::new();

    for action in catalog.all() {
        services
            .entry(action.service.clone())
            .or_insert_with(|| new_service(&action.service))
            .actions
            .push(CatalogEntry {
                name: action.event.clone(),
                description: action.description.clone(),
                payload_example: None,
            });
    }

    for reaction in registry.all() {
        services
            .entry(reaction.service_name().to_string())
            .or_insert_with(|| new_service(reaction.service_name()))
            .reactions
            .push(CatalogEntry {
                name: reaction.kind().to_string(),
                description: reaction.description().to_string(),
                payload_example: Some(reaction.payload_example().to_string()),
            });
    }

    services.into_values().collect()
}

fn new_service(name: &str) -> ServiceCatalog {
    ServiceCatalog {
        name: name.to_string(),
        connectable: ServiceName::from_db(name).is_some(),
        actions: Vec::new(),
        reactions: Vec::new(),
    }
}

fn current_unix_time() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hooks::ReactionRegistry;

    #[test]
    fn empty_catalog_and_registry_produce_no_services() {
        let catalog = ActionCatalog::builder().build();
        let registry = ReactionRegistry::builder().build();

        let response = build_response(
            "127.0.0.1".to_string(),
            &catalog,
            &registry,
            "token123".to_string(),
        );

        assert!(response.server.services.is_empty());
        assert_eq!(response.client.host, "127.0.0.1");
        assert_eq!(response.server.token, "token123");
    }

    #[test]
    fn actions_are_grouped_by_service() {
        let catalog = ActionCatalog::builder()
            .register("github", "workflow_run", "CI finished")
            .register("github", "push", "Commits pushed")
            .register("gitlab", "pipeline", "Pipeline finished")
            .build();
        let registry = ReactionRegistry::builder().build();

        let response = build_response("10.0.0.1".to_string(), &catalog, &registry, "t".to_string());

        assert_eq!(response.server.services.len(), 2);
        assert_eq!(response.server.services[0].name, "github");
        assert_eq!(response.server.services[0].actions.len(), 2);
        assert_eq!(response.server.services[1].name, "gitlab");
        assert_eq!(response.server.services[1].actions.len(), 1);
    }

    #[test]
    fn current_time_is_populated() {
        let catalog = ActionCatalog::builder().build();
        let registry = ReactionRegistry::builder().build();

        let response = build_response(
            "127.0.0.1".to_string(),
            &catalog,
            &registry,
            "t".to_string(),
        );

        assert!(response.server.current_time > 0);
    }

    #[test]
    fn connectable_reflects_service_name_enum() {
        let catalog = ActionCatalog::builder()
            .register("github", "workflow_run", "CI finished")
            .build();
        let registry = ReactionRegistry::builder().build();

        let response = build_response(
            "127.0.0.1".to_string(),
            &catalog,
            &registry,
            "t".to_string(),
        );
        assert!(response.server.services[0].connectable);
    }
}
