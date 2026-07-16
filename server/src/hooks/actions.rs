#[derive(Clone, Debug)]
pub struct ActionMetadata {
    pub service: String,
    pub event: String,
    pub description: String,
}

#[derive(Clone, Default)]
pub struct ActionCatalog {
    actions: Vec<ActionMetadata>,
}

impl ActionCatalog {
    pub fn builder() -> ActionCatalogBuilder {
        ActionCatalogBuilder {
            actions: Vec::new(),
        }
    }

    pub fn all(&self) -> &[ActionMetadata] {
        &self.actions
    }

    pub fn contains(&self, service: &str, event: &str) -> bool {
        self.actions
            .iter()
            .any(|a| a.service == service && a.event == event)
    }
}

pub struct ActionCatalogBuilder {
    actions: Vec<ActionMetadata>,
}

impl ActionCatalogBuilder {
    pub fn register(
        mut self,
        service: impl Into<String>,
        event: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        self.actions.push(ActionMetadata {
            service: service.into(),
            event: event.into(),
            description: description.into(),
        });
        self
    }

    pub fn build(self) -> ActionCatalog {
        ActionCatalog {
            actions: self.actions,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_catalog_has_no_actions() {
        let catalog = ActionCatalog::builder().build();
        assert!(catalog.all().is_empty());
    }

    #[test]
    fn registered_actions_are_findable() {
        let catalog = ActionCatalog::builder()
            .register("github", "workflow_run", "CI workflow finished")
            .register("github", "push", "Commits pushed to a branch")
            .build();

        let actions = catalog.all();
        assert_eq!(actions.len(), 2);
        assert_eq!(actions[0].service, "github");
        assert_eq!(actions[0].event, "workflow_run");
    }

    #[test]
    fn registration_order_is_preserved() {
        let catalog = ActionCatalog::builder()
            .register("a", "one", "d1")
            .register("b", "two", "d2")
            .register("c", "three", "d3")
            .build();

        let events: Vec<&str> = catalog.all().iter().map(|a| a.event.as_str()).collect();
        assert_eq!(events, vec!["one", "two", "three"]);
    }

    #[test]
    fn contains_matches_registered_pairs_only() {
        let catalog = ActionCatalog::builder()
            .register("github", "workflow_run", "CI finished")
            .build();

        assert!(catalog.contains("github", "workflow_run"));
        assert!(!catalog.contains("github", "push"));
        assert!(!catalog.contains("gitlab", "workflow_run"));
    }
}
