use crate::error::Result;
use std::any::TypeId;

/// Core trait for any loadable resource
pub trait Resource: Send + Sync + 'static {
    /// Get file path of this resource
    fn get_path(&self) -> &str;

    /// Get approximate size in bytes
    fn get_size(&self) -> usize;

    /// Get resource type name
    fn get_type_name(&self) -> &str;

    /// Get type ID for dynamic dispatch
    fn get_type_id(&self) -> TypeId;

    /// Unload resource, freeing memory
    fn unload(&mut self) -> Result<()>;

    /// Reload resource from disk
    fn reload(&mut self) -> Result<()>;

    /// Check if resource is valid
    fn is_valid(&self) -> bool;
}

/// Resource statistics
#[derive(Clone, Debug)]
pub struct ResourceStats {
    pub total_resources: usize,
    pub total_memory_used: usize,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub load_time_ms: u64,
    pub unload_time_ms: u64,
}

impl ResourceStats {
    pub fn new() -> Self {
        Self {
            total_resources: 0,
            total_memory_used: 0,
            cache_hits: 0,
            cache_misses: 0,
            load_time_ms: 0,
            unload_time_ms: 0,
        }
    }

    pub fn cache_hit_ratio(&self) -> f32 {
        let total = self.cache_hits + self.cache_misses;
        if total == 0 {
            0.0
        } else {
            self.cache_hits as f32 / total as f32
        }
    }
}

impl Default for ResourceStats {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_stats() {
        let mut stats = ResourceStats::new();
        stats.cache_hits = 90;
        stats.cache_misses = 10;
        assert!((stats.cache_hit_ratio() - 0.9).abs() < 0.01);
    }

    #[test]
    fn test_cache_hit_ratio_zero() {
        let stats = ResourceStats::new();
        assert_eq!(stats.cache_hit_ratio(), 0.0);
    }
}
