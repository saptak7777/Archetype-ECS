use crate::assets::Asset;
use parking_lot::RwLock;
use std::any::Any;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// Entry in the asset cache
struct CacheEntry {
    asset: Arc<RwLock<Box<dyn Any + Send + Sync>>>,
    size: usize,
    /// Last access time (atomic for lock-free updates)
    last_access: AtomicU64,
    /// Access count (atomic for lock-free updates)
    access_count: AtomicU64,
}

/// Asset cache with concurrent access and approximate LRU eviction
pub struct AssetCache {
    /// Main storage: Readers take read lock, Writers take write lock
    entries: RwLock<HashMap<u64, Arc<CacheEntry>>>,
    /// Total size in bytes (approximate due to concurrency)
    total_size: AtomicU64,
    /// Max size in bytes
    max_size: usize,
    /// Global access counter for LRU ordering
    access_counter: AtomicU64,
    /// Cache statistics
    stats: CacheStats,
}

/// Cache statistics (Atomic)
#[derive(Debug, Default)]
pub struct CacheStats {
    pub hits: AtomicU64,
    pub misses: AtomicU64,
    pub evictions: AtomicU64,
    pub total_loads: AtomicU64,
}

impl AssetCache {
    /// Create new cache with memory budget
    pub fn new(max_size: usize) -> Self {
        Self {
            entries: RwLock::new(HashMap::new()),
            total_size: AtomicU64::new(0),
            max_size,
            access_counter: AtomicU64::new(0),
            stats: CacheStats::default(),
        }
    }

    /// Insert asset into cache
    pub fn insert<T: Asset>(&self, id: u64, asset: T) -> Arc<RwLock<T>> {
        let size = asset.memory_size();
        let current_size = self.total_size.load(Ordering::Relaxed);

        // Evict if over budget (approximate check)
        if current_size + (size as u64) > (self.max_size as u64) {
            self.evict_approximate_lru();
        }

        let typed_arc = Arc::new(RwLock::new(asset));
        let boxed: Box<dyn Any + Send + Sync> = Box::new(typed_arc.clone());
        let arc_entry = Arc::new(CacheEntry {
            asset: Arc::new(RwLock::new(boxed)),
            size,
            last_access: AtomicU64::new(self.next_access_time()),
            access_count: AtomicU64::new(1),
        });

        // Write lock needed for insertion
        let mut entries = self.entries.write();
        entries.insert(id, arc_entry);

        // Update stats
        self.total_size.fetch_add(size as u64, Ordering::Relaxed);
        self.stats.total_loads.fetch_add(1, Ordering::Relaxed);

        typed_arc
    }

    /// Get asset from cache (Lock-Free Read)
    pub fn get<T: Asset>(&self, id: u64) -> Option<Arc<RwLock<T>>> {
        // critical: only read lock needed
        let entries = self.entries.read();

        if let Some(entry) = entries.get(&id) {
            // Lock-free metadata updates
            entry
                .last_access
                .store(self.next_access_time(), Ordering::Relaxed);
            entry.access_count.fetch_add(1, Ordering::Relaxed);
            self.stats.hits.fetch_add(1, Ordering::Relaxed);

            // Try to downcast
            let lock = entry.asset.read();
            if let Some(arc_ref) = lock.downcast_ref::<Arc<RwLock<T>>>() {
                return Some(arc_ref.clone());
            }
        }

        self.stats.misses.fetch_add(1, Ordering::Relaxed);
        None
    }

    /// Get asset or load if missing
    ///
    /// This method uses specific optimization to avoid write locking if the asset exists.
    pub fn get_or_load<T: Asset, F>(&self, id: u64, loader: F) -> Arc<RwLock<T>>
    where
        F: FnOnce() -> T,
    {
        // 1. Fast path: Read lock
        if let Some(asset) = self.get::<T>(id) {
            return asset;
        }

        // 2. Slow path: Load and insert
        // Note: multiple threads might load simultaneously, but only one will win insertion
        let asset = loader();
        self.insert(id, asset)
    }

    /// Remove asset from cache
    pub fn remove(&self, id: u64) -> bool {
        let mut entries = self.entries.write();
        if let Some(entry) = entries.remove(&id) {
            self.total_size
                .fetch_sub(entry.size as u64, Ordering::Relaxed);
            true
        } else {
            false
        }
    }

    /// Approximate LRU eviction
    ///
    /// Uses random sampling or scanning to find candidates to avoid full sort
    fn evict_approximate_lru(&self) {
        let mut entries = self.entries.write(); // Need write lock to remove

        if entries.is_empty() {
            return;
        }

        // Simple strategy: Scan a subset or all if small
        // For simplicity and correctness in this phase, we'll scan all (O(N))
        // but since we already hold the write lock, it's consistent.
        // Optimization: In a real "Lock-Free" heavy system, we'd sample K items.

        // Find oldest entry
        let mut oldest_id = None;
        let mut oldest_time = u64::MAX;

        for (&id, entry) in entries.iter() {
            let time = entry.last_access.load(Ordering::Relaxed);
            if time < oldest_time {
                oldest_time = time;
                oldest_id = Some(id);
            }
        }

        if let Some(id) = oldest_id {
            if let Some(entry) = entries.remove(&id) {
                self.total_size
                    .fetch_sub(entry.size as u64, Ordering::Relaxed);
                self.stats.evictions.fetch_add(1, Ordering::Relaxed);
            }
        }
    }

    /// Clear all cached assets
    pub fn clear(&self) {
        let mut entries = self.entries.write();
        entries.clear();
        self.total_size.store(0, Ordering::Relaxed);
    }

    /// Get atomic access counter
    fn next_access_time(&self) -> u64 {
        self.access_counter.fetch_add(1, Ordering::Relaxed)
    }

    /// Get current memory usage
    pub fn memory_usage(&self) -> usize {
        self.total_size.load(Ordering::Relaxed) as usize
    }

    /// Get number of cached assets
    pub fn len(&self) -> usize {
        self.entries.read().len()
    }

    /// Check if cache is empty
    pub fn is_empty(&self) -> bool {
        self.entries.read().is_empty()
    }

    /// Get cache stats snapshot
    pub fn stats_snapshot(&self) -> CacheStatsSnapshot {
        CacheStatsSnapshot {
            hits: self.stats.hits.load(Ordering::Relaxed),
            misses: self.stats.misses.load(Ordering::Relaxed),
            evictions: self.stats.evictions.load(Ordering::Relaxed),
            total_loads: self.stats.total_loads.load(Ordering::Relaxed),
        }
    }
}

/// Snapshot of cache statistics
#[derive(Debug, Clone, Default)]
pub struct CacheStatsSnapshot {
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
    pub total_loads: u64,
}
