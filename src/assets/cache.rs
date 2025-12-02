use crate::assets::Asset;
use parking_lot::RwLock;
use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;

/// Entry in the asset cache
struct CacheEntry {
    asset: Arc<RwLock<Box<dyn Any + Send + Sync>>>,
    size: usize,
    access_count: u64,
    last_access: u64,
}

/// Asset cache with LRU eviction
pub struct AssetCache {
    entries: HashMap<u64, CacheEntry>,
    total_size: usize,
    max_size: usize,
    access_counter: u64,
    stats: CacheStats,
}

/// Cache statistics
#[derive(Clone, Debug, Default)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
    pub total_loads: u64,
}

impl AssetCache {
    /// Create new cache with memory budget
    pub fn new(max_size: usize) -> Self {
        Self {
            entries: HashMap::new(),
            total_size: 0,
            max_size,
            access_counter: 0,
            stats: CacheStats::default(),
        }
    }

    /// Insert asset into cache
    pub fn insert<T: Asset>(&mut self, id: u64, asset: T) -> Arc<RwLock<T>> {
        let size = asset.memory_size();

        // Evict if necessary
        while self.total_size + size > self.max_size && !self.entries.is_empty() {
            self.evict_lru();
        }

        let typed_arc = Arc::new(RwLock::new(asset));
        let boxed: Box<dyn Any + Send + Sync> = Box::new(typed_arc.clone());
        let arc = Arc::new(RwLock::new(boxed));

        self.entries.insert(
            id,
            CacheEntry {
                asset: arc,
                size,
                access_count: 1,
                last_access: self.access_counter,
            },
        );

        self.access_counter += 1;
        self.total_size += size;
        self.stats.total_loads += 1;

        typed_arc
    }

    /// Get asset from cache
    pub fn get<T: Asset>(&mut self, id: u64) -> Option<Arc<RwLock<T>>> {
        if let Some(entry) = self.entries.get_mut(&id) {
            entry.access_count += 1;
            entry.last_access = self.access_counter;
            self.access_counter += 1;
            self.stats.hits += 1;

            // Try to downcast
            let lock = entry.asset.read();
            if let Some(arc_ref) = lock.downcast_ref::<Arc<RwLock<T>>>() {
                return Some(arc_ref.clone());
            }
        }

        self.stats.misses += 1;
        None
    }

    /// Remove asset from cache
    pub fn remove(&mut self, id: u64) -> bool {
        if let Some(entry) = self.entries.remove(&id) {
            self.total_size = self.total_size.saturating_sub(entry.size);
            true
        } else {
            false
        }
    }

    /// Evict least recently used asset
    fn evict_lru(&mut self) {
        if let Some((&id, _)) = self
            .entries
            .iter()
            .min_by_key(|(_, entry)| entry.last_access)
        {
            self.remove(id);
            self.stats.evictions += 1;
        }
    }

    /// Clear all cached assets
    pub fn clear(&mut self) {
        self.entries.clear();
        self.total_size = 0;
    }

    /// Get cache statistics
    pub fn stats(&self) -> &CacheStats {
        &self.stats
    }

    /// Get current memory usage
    pub fn memory_usage(&self) -> usize {
        self.total_size
    }

    /// Get memory utilization (0.0 to 1.0)
    pub fn utilization(&self) -> f32 {
        if self.max_size == 0 {
            0.0
        } else {
            self.total_size as f32 / self.max_size as f32
        }
    }

    /// Get number of cached assets
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if cache is empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}
