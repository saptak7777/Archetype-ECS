// Asset Pipeline Module
//
// Provides advanced asset management with:
// - Hot-reloading
// - Dependency tracking
// - Async loading
// - Smart caching

pub mod cache;
pub mod loader;
pub mod server;

pub use cache::AssetCache;
pub use loader::{AssetLoader, LoadContext};
pub use server::{AssetEvent, AssetServer};

/// Trait for assets that can be loaded
pub trait Asset: Send + Sync + 'static {
    /// Get asset type name
    fn type_name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }

    /// Get approximate memory size in bytes
    fn memory_size(&self) -> usize {
        std::mem::size_of_val(self)
    }

    /// Called when asset is unloaded
    fn on_unload(&mut self) {}
}

/// Strong handle to an asset
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct AssetHandle<T: Asset> {
    id: u64,
    generation: u32,
    _phantom: std::marker::PhantomData<T>,
}

impl<T: Asset> AssetHandle<T> {
    pub fn new(id: u64, generation: u32) -> Self {
        Self {
            id,
            generation,
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn generation(&self) -> u32 {
        self.generation
    }
}

/// Weak handle that doesn't prevent asset unloading
#[derive(Clone, Copy, Debug)]
pub struct WeakAssetHandle<T: Asset> {
    id: u64,
    _phantom: std::marker::PhantomData<T>,
}

impl<T: Asset> WeakAssetHandle<T> {
    pub fn new(id: u64) -> Self {
        Self {
            id,
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn id(&self) -> u64 {
        self.id
    }
}
