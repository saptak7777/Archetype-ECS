use crate::error::Result;
use std::collections::HashMap;

/// Memory pool for efficient allocation
pub struct MemoryPool {
    total_capacity: usize,
    used: usize,
    allocations: HashMap<String, usize>,
}

impl MemoryPool {
    pub fn new(capacity: usize) -> Self {
        Self {
            total_capacity: capacity,
            used: 0,
            allocations: HashMap::new(),
        }
    }

    /// Allocate memory for a resource
    pub fn allocate(&mut self, name: &str, size: usize) -> Result<()> {
        if self.used + size > self.total_capacity {
            return Err(crate::error::EcsError::ResourceMemoryOverflow(format!(
                "Memory pool overflow: {} + {} > {}",
                self.used, size, self.total_capacity
            )));
        }

        self.used += size;
        *self.allocations.entry(name.to_string()).or_insert(0) += size;
        Ok(())
    }

    /// Deallocate memory
    pub fn deallocate(&mut self, name: &str, size: usize) -> Result<()> {
        if let Some(allocated) = self.allocations.get_mut(name) {
            if *allocated >= size {
                *allocated -= size;
                self.used -= size;
                if *allocated == 0 {
                    self.allocations.remove(name);
                }
                Ok(())
            } else {
                Err(crate::error::EcsError::ResourceDeallocError(format!(
                    "Deallocating more than allocated for {name}"
                )))
            }
        } else {
            Err(crate::error::EcsError::ResourceNotFound(format!(
                "No allocation found for: {name}"
            )))
        }
    }

    /// Get available memory
    pub fn get_available(&self) -> usize {
        self.total_capacity - self.used
    }

    /// Get used memory
    pub fn get_used(&self) -> usize {
        self.used
    }

    /// Get utilization percentage
    pub fn get_utilization(&self) -> f32 {
        self.used as f32 / self.total_capacity as f32
    }

    /// Get memory used by specific resource
    pub fn get_allocation(&self, name: &str) -> usize {
        *self.allocations.get(name).unwrap_or(&0)
    }

    /// Clear all allocations
    pub fn clear(&mut self) {
        self.used = 0;
        self.allocations.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_pool_allocation() {
        let mut pool = MemoryPool::new(1000);
        assert_eq!(pool.get_available(), 1000);

        pool.allocate("texture", 500).unwrap();
        assert_eq!(pool.get_available(), 500);
        assert_eq!(pool.get_allocation("texture"), 500);
    }

    #[test]
    fn test_memory_pool_overflow() {
        let mut pool = MemoryPool::new(100);
        assert!(pool.allocate("big", 150).is_err());
    }

    #[test]
    fn test_memory_pool_deallocation() {
        let mut pool = MemoryPool::new(1000);
        pool.allocate("texture", 500).unwrap();
        pool.deallocate("texture", 500).unwrap();
        assert_eq!(pool.get_used(), 0);
    }

    #[test]
    fn test_memory_utilization() {
        let mut pool = MemoryPool::new(1000);
        pool.allocate("texture", 250).unwrap();
        assert!((pool.get_utilization() - 0.25).abs() < 0.01);
    }
}
