use crate::resources::Resource;
use std::marker::PhantomData;

/// Type-safe handle to a resource
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Handle<T: Resource + ?Sized> {
    id: u64,
    generation: u32,
    _phantom: PhantomData<T>,
}

impl<T: Resource + ?Sized> Handle<T> {
    /// Create a new handle
    pub fn new(id: u64, generation: u32) -> Self {
        Self {
            id,
            generation,
            _phantom: PhantomData,
        }
    }

    /// Get the handle's ID
    pub fn id(&self) -> u64 {
        self.id
    }

    /// Get the handle's generation (for version tracking)
    pub fn generation(&self) -> u32 {
        self.generation
    }

    /// Check if handle is valid (generation matches)
    pub fn is_valid_generation(&self, current_generation: u32) -> bool {
        self.generation == current_generation
    }
}

/// Generation tracking for reused IDs
pub struct GenerationTracker {
    generations: Vec<u32>,
    available_ids: Vec<u64>,
}

impl GenerationTracker {
    pub fn new(capacity: usize) -> Self {
        let mut available = Vec::with_capacity(capacity);
        for i in 0..capacity as u64 {
            available.push(i);
        }
        available.reverse();

        Self {
            generations: vec![0; capacity],
            available_ids: available,
        }
    }

    pub fn allocate(&mut self) -> u64 {
        self.available_ids.pop().unwrap_or(0)
    }

    pub fn deallocate(&mut self, id: u64) {
        if (id as usize) < self.generations.len() {
            self.generations[id as usize] += 1;
            self.available_ids.push(id);
        }
    }

    pub fn get_generation(&self, id: u64) -> u32 {
        self.generations.get(id as usize).copied().unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resources::asset_types::DataResource;

    #[test]
    fn test_handle_creation() {
        let handle: Handle<DataResource> = Handle::new(42, 1);
        assert_eq!(handle.id(), 42);
        assert_eq!(handle.generation(), 1);
    }

    #[test]
    fn test_generation_tracker() {
        let mut tracker = GenerationTracker::new(10);
        let id = tracker.allocate();
        assert_eq!(tracker.get_generation(id), 0);

        tracker.deallocate(id);
        assert_eq!(tracker.get_generation(id), 1);
    }

    #[test]
    fn test_handle_validation() {
        let handle: Handle<DataResource> = Handle::new(5, 2);
        assert!(handle.is_valid_generation(2));
        assert!(!handle.is_valid_generation(3));
    }
}
