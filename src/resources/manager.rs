use crate::error::Result;
use crate::resources::{GenerationTracker, Handle, MemoryPool, Resource, ResourceStats};
use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::Arc;

/// Central resource manager
pub struct ResourceManager {
    resources: HashMap<String, Arc<Mutex<Box<dyn Resource>>>>,
    generation_tracker: GenerationTracker,
    memory_pool: MemoryPool,
    stats: ResourceStats,
}

impl ResourceManager {
    /// Create new resource manager with capacity
    pub fn new(memory_capacity: usize) -> Self {
        Self {
            resources: HashMap::new(),
            generation_tracker: GenerationTracker::new(10000),
            memory_pool: MemoryPool::new(memory_capacity),
            stats: ResourceStats::new(),
        }
    }

    /// Load a resource from path
    pub fn load<T: Resource + 'static>(&mut self, path: &str, resource: T) -> Result<Handle<T>> {
        let size = resource.get_size();

        // Allocate memory
        self.memory_pool.allocate(path, size)?;

        // Store resource
        let boxed: Box<dyn Resource> = Box::new(resource);
        let arc = Arc::new(Mutex::new(boxed));
        self.resources.insert(path.to_string(), arc);

        // Update stats
        self.stats.total_resources += 1;
        self.stats.total_memory_used += size;

        // Create handle
        let id = self.generation_tracker.allocate();
        let generation = self.generation_tracker.get_generation(id);
        Ok(Handle::new(id, generation))
    }

    /// Get resource by path (returns Arc for shared access)
    pub fn get(&mut self, path: &str) -> Option<Arc<Mutex<Box<dyn Resource>>>> {
        if self.resources.contains_key(path) {
            self.stats.cache_hits += 1;
            self.resources.get(path).cloned()
        } else {
            self.stats.cache_misses += 1;
            None
        }
    }

    /// Unload a resource
    pub fn unload(&mut self, path: &str) -> Result<()> {
        if let Some(arc) = self.resources.remove(path) {
            let mut resource = arc.lock();
            let size = resource.get_size();
            resource.unload()?;
            self.memory_pool.deallocate(path, size)?;
            self.stats.total_resources = self.stats.total_resources.saturating_sub(1);
            self.stats.total_memory_used = self.stats.total_memory_used.saturating_sub(size);
            Ok(())
        } else {
            Err(crate::error::EcsError::ResourceNotFound(format!(
                "Resource not found: {path}"
            )))
        }
    }

    /// Get resource statistics
    pub fn get_stats(&self) -> ResourceStats {
        self.stats.clone()
    }

    /// Get memory utilization
    pub fn get_memory_utilization(&self) -> f32 {
        self.memory_pool.get_utilization()
    }

    /// List all loaded resources
    pub fn list_resources(&self) -> Vec<String> {
        self.resources.keys().cloned().collect()
    }

    /// Get resource count
    pub fn resource_count(&self) -> usize {
        self.resources.len()
    }

    /// Clear all resources
    pub fn clear(&mut self) -> Result<()> {
        let paths: Vec<_> = self.resources.keys().cloned().collect();
        for path in paths {
            self.unload(&path)?;
        }
        self.memory_pool.clear();
        Ok(())
    }
}

impl Default for ResourceManager {
    fn default() -> Self {
        Self::new(1024 * 1024 * 512) // 512 MB default
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resources::asset_types::DataResource;

    #[test]
    fn test_resource_manager_load() {
        let mut manager = ResourceManager::new(1024 * 1024);
        let data = DataResource::new("test.bin".to_string(), vec![0u8; 100]);
        let _handle = manager.load("test.bin", data).unwrap();
        assert!(manager.resource_count() > 0);
    }

    #[test]
    fn test_resource_manager_unload() {
        let mut manager = ResourceManager::new(1024 * 1024);
        let data = DataResource::new("test.bin".to_string(), vec![0u8; 100]);
        manager.load("test.bin", data).unwrap();
        manager.unload("test.bin").unwrap();
        assert_eq!(manager.resource_count(), 0);
    }

    #[test]
    fn test_resource_manager_memory() {
        let mut manager = ResourceManager::new(1000);
        let data = DataResource::new("test.bin".to_string(), vec![0u8; 500]);
        manager.load("test.bin", data).unwrap();
        assert!(manager.get_memory_utilization() > 0.0);
    }

    #[test]
    fn test_resource_manager_get() {
        let mut manager = ResourceManager::new(1024 * 1024);
        let data = DataResource::new("test.bin".to_string(), vec![0u8; 100]);
        manager.load("test.bin", data).unwrap();

        let resource = manager.get("test.bin");
        assert!(resource.is_some());

        assert_eq!(manager.get_stats().cache_hits, 1);
    }
}
