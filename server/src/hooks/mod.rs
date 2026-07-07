pub mod context;
pub mod engine;
pub mod matcher;
pub mod reactions;
pub mod registry;
pub mod templating;

pub use context::ReactionContext;
pub use matcher::matches;
pub use registry::{ReactionExecutor, ReactionRegistry};
pub use templating::render;
