pub mod asset_types;
pub mod handle;
pub mod loader;
pub mod manager;
pub mod pool;
pub mod resource;

pub use asset_types::{AudioResource, DataResource, TextureResource};
pub use handle::{GenerationTracker, Handle};
pub use loader::ResourceLoader;
pub use manager::ResourceManager;
pub use pool::MemoryPool;
pub use resource::{Resource, ResourceStats};
