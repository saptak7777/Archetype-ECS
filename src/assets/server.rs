use crate::assets::{Asset, AssetCache, AssetHandle, AssetLoader, LoadContext};
use crate::error::{EcsError, Result};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Events emitted by the asset server
#[derive(Clone, Debug)]
pub enum AssetEvent<T: Asset> {
    /// Asset was loaded
    Loaded { handle: AssetHandle<T> },
    /// Asset was modified (hot-reload)
    Modified { handle: AssetHandle<T> },
    /// Asset was unloaded
    Unloaded { handle: AssetHandle<T> },
}

/// Asset server for managing asset loading and caching
pub struct AssetServer {
    loaders: HashMap<String, Arc<dyn LoaderWrapper>>,
    cache: Arc<RwLock<AssetCache>>,
    base_path: PathBuf,
    next_id: Arc<RwLock<u64>>,
    handle_to_path: Arc<RwLock<HashMap<u64, PathBuf>>>,
}

/// Wrapper trait for type-erased loaders
trait LoaderWrapper: Send + Sync {
    fn load_asset(&self, path: &Path, bytes: &[u8])
        -> Result<Box<dyn std::any::Any + Send + Sync>>;
    fn extensions(&self) -> &[&str];
}

struct TypedLoaderWrapper<L: AssetLoader> {
    loader: L,
}

impl<L: AssetLoader + 'static> LoaderWrapper for TypedLoaderWrapper<L> {
    fn load_asset(
        &self,
        path: &Path,
        bytes: &[u8],
    ) -> Result<Box<dyn std::any::Any + Send + Sync>> {
        let context = LoadContext { path, bytes };
        let settings = L::Settings::default();
        let asset = self.loader.load(context, &settings)?;
        Ok(Box::new(asset))
    }

    fn extensions(&self) -> &[&str] {
        self.loader.extensions()
    }
}

impl AssetServer {
    /// Create new asset server
    pub fn new<P: Into<PathBuf>>(base_path: P) -> Self {
        Self {
            loaders: HashMap::new(),
            cache: Arc::new(RwLock::new(AssetCache::new(512 * 1024 * 1024))), // 512MB default
            base_path: base_path.into(),
            next_id: Arc::new(RwLock::new(1)),
            handle_to_path: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register an asset loader
    pub fn register_loader<L: AssetLoader + 'static>(&mut self, loader: L) {
        let wrapper = Arc::new(TypedLoaderWrapper { loader });
        for ext in wrapper.extensions() {
            self.loaders.insert(ext.to_string(), wrapper.clone());
        }
    }

    /// Load an asset from a file
    pub fn load<T: Asset>(&self, path: impl AsRef<Path>) -> Result<AssetHandle<T>> {
        let path = path.as_ref();
        let full_path = self.base_path.join(path);

        // Read file
        let bytes = std::fs::read(&full_path)
            .map_err(|e| EcsError::AssetLoadError(format!("Failed to read file: {e}")))?;

        // Find loader by extension
        let extension = path
            .extension()
            .and_then(|e| e.to_str())
            .ok_or_else(|| EcsError::AssetLoadError("No file extension".to_string()))?;

        let loader = self.loaders.get(extension).ok_or_else(|| {
            EcsError::AssetLoadError(format!("No loader for extension: {extension}"))
        })?;

        // Load asset
        let any_asset = loader.load_asset(path, &bytes)?;
        let asset = any_asset
            .downcast::<T>()
            .map_err(|_| EcsError::AssetLoadError("Type mismatch".to_string()))?;

        // Generate handle
        let id = {
            let mut next_id = self.next_id.write();
            let id = *next_id;
            *next_id += 1;
            id
        };

        // Store in cache
        let mut cache = self.cache.write();
        cache.insert(id, *asset);

        // Track path
        self.handle_to_path.write().insert(id, path.to_path_buf());

        Ok(AssetHandle::new(id, 0))
    }

    /// Get a loaded asset
    pub fn get<T: Asset>(&self, handle: AssetHandle<T>) -> Option<Arc<RwLock<T>>> {
        let mut cache = self.cache.write();
        cache.get(handle.id())
    }

    /// Unload an asset
    pub fn unload<T: Asset>(&self, handle: AssetHandle<T>) -> bool {
        let mut cache = self.cache.write();
        let removed = cache.remove(handle.id());
        if removed {
            self.handle_to_path.write().remove(&handle.id());
        }
        removed
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> crate::assets::cache::CacheStats {
        self.cache.read().stats().clone()
    }

    /// Get memory usage
    pub fn memory_usage(&self) -> usize {
        self.cache.read().memory_usage()
    }

    /// Clear all cached assets
    pub fn clear_cache(&self) {
        self.cache.write().clear();
        self.handle_to_path.write().clear();
    }

    /// Get number of loaded assets
    pub fn loaded_count(&self) -> usize {
        self.cache.read().len()
    }
}

impl Default for AssetServer {
    fn default() -> Self {
        Self::new("assets")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assets::loader::{BinaryLoader, JsonLoader, TextLoader};

    #[test]
    fn test_asset_server_creation() {
        let server = AssetServer::new("test_assets");
        assert_eq!(server.loaded_count(), 0);
    }

    #[test]
    fn test_loader_registration() {
        let mut server = AssetServer::new("test_assets");
        server.register_loader(BinaryLoader);
        server.register_loader(JsonLoader);
        server.register_loader(TextLoader);

        assert_eq!(server.loaders.len(), 8); // bin, dat, json, txt, md, toml, yaml, yml
    }
}
