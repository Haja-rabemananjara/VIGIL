use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;

use crate::error::AppError;
use crate::hooks::context::ReactionContext;

#[async_trait]
pub trait ReactionExecutor: Send + Sync {
    fn kind(&self) -> &'static str;
    fn service_name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn payload_example(&self) -> &'static str;
    async fn execute(&self, ctx: &ReactionContext<'_>) -> Result<(), AppError>;
}

#[derive(Clone, Default)]
pub struct ReactionRegistry {
    reactions: Arc<HashMap<&'static str, Arc<dyn ReactionExecutor>>>,
}

impl ReactionRegistry {
    pub fn builder() -> ReactionRegistryBuilder {
        ReactionRegistryBuilder {
            reactions: HashMap::new(),
        }
    }

    pub fn get(&self, kind: &str) -> Option<Arc<dyn ReactionExecutor>> {
        self.reactions.get(kind).cloned()
    }

    pub fn contains(&self, kind: &str) -> bool {
        self.reactions.contains_key(kind)
    }

    pub fn all(&self) -> impl Iterator<Item = &Arc<dyn ReactionExecutor>> {
        self.reactions.values()
    }
}

pub struct ReactionRegistryBuilder {
    reactions: HashMap<&'static str, Arc<dyn ReactionExecutor>>,
}

impl ReactionRegistryBuilder {
    pub fn register(mut self, executor: Arc<dyn ReactionExecutor>) -> Self {
        self.reactions.insert(executor.kind(), executor);
        self
    }

    pub fn build(self) -> ReactionRegistry {
        ReactionRegistry {
            reactions: Arc::new(self.reactions),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct DummyReaction {
        calls: AtomicUsize,
    }

    impl DummyReaction {
        fn new() -> Self {
            Self {
                calls: AtomicUsize::new(0),
            }
        }
    }

    #[async_trait]
    impl ReactionExecutor for DummyReaction {
        fn kind(&self) -> &'static str {
            "dummy"
        }
        fn service_name(&self) -> &'static str {
            "test"
        }
        fn description(&self) -> &'static str {
            "Test-only reaction that increments a counter"
        }
        fn payload_example(&self) -> &'static str {
            "{}"
        }

        async fn execute(&self, _ctx: &ReactionContext<'_>) -> Result<(), AppError> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
    }

    #[test]
    fn empty_registry_finds_nothing() {
        let reg = ReactionRegistry::builder().build();
        assert!(!reg.contains("dummy"));
        assert!(reg.get("dummy").is_none());
    }

    #[test]
    fn registered_reaction_is_findable_by_kind() {
        let reg = ReactionRegistry::builder()
            .register(Arc::new(DummyReaction::new()))
            .build();

        assert!(reg.contains("dummy"));
        let found = reg.get("dummy").expect("dummy should be registered");
        assert_eq!(found.kind(), "dummy");
        assert_eq!(
            found.description(),
            "Test-only reaction that increments a counter"
        );
    }

    #[test]
    fn duplicate_registration_replaces() {
        struct A;
        struct B;

        #[async_trait]
        impl ReactionExecutor for A {
            fn kind(&self) -> &'static str {
                "same"
            }
            fn service_name(&self) -> &'static str {
                "test"
            }
            fn description(&self) -> &'static str {
                "first"
            }
            fn payload_example(&self) -> &'static str {
                "{}"
            }
            async fn execute(&self, _: &ReactionContext<'_>) -> Result<(), AppError> {
                Ok(())
            }
        }

        #[async_trait]
        impl ReactionExecutor for B {
            fn kind(&self) -> &'static str {
                "same"
            }
            fn service_name(&self) -> &'static str {
                "test"
            }
            fn description(&self) -> &'static str {
                "second"
            }
            fn payload_example(&self) -> &'static str {
                "{}"
            }
            async fn execute(&self, _: &ReactionContext<'_>) -> Result<(), AppError> {
                Ok(())
            }
        }

        let reg = ReactionRegistry::builder()
            .register(Arc::new(A))
            .register(Arc::new(B))
            .build();

        let found = reg.get("same").unwrap();
        assert_eq!(found.description(), "second");
    }

    #[test]
    fn all_returns_every_registered_executor() {
        struct A;
        struct B;

        #[async_trait]
        impl ReactionExecutor for A {
            fn kind(&self) -> &'static str {
                "a"
            }
            fn service_name(&self) -> &'static str {
                "test"
            }
            fn description(&self) -> &'static str {
                ""
            }
            fn payload_example(&self) -> &'static str {
                "{}"
            }
            async fn execute(&self, _: &ReactionContext<'_>) -> Result<(), AppError> {
                Ok(())
            }
        }

        #[async_trait]
        impl ReactionExecutor for B {
            fn kind(&self) -> &'static str {
                "b"
            }
            fn service_name(&self) -> &'static str {
                "test"
            }
            fn description(&self) -> &'static str {
                ""
            }
            fn payload_example(&self) -> &'static str {
                "{}"
            }
            async fn execute(&self, _: &ReactionContext<'_>) -> Result<(), AppError> {
                Ok(())
            }
        }

        let reg = ReactionRegistry::builder()
            .register(Arc::new(A))
            .register(Arc::new(B))
            .build();

        let mut kinds: Vec<&str> = reg.all().map(|r| r.kind()).collect();
        kinds.sort();
        assert_eq!(kinds, vec!["a", "b"]);
    }
}
