//! System trait and access metadata

use crate::error::Result;
use crate::World;
use std::any::TypeId;

/// System ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SystemId(pub u32); // Made public

/// System access metadata
#[derive(Debug, Clone)]
pub struct SystemAccess {
    pub reads: Vec<TypeId>,
    pub writes: Vec<TypeId>,
}

impl SystemAccess {
    /// Create empty access
    pub fn empty() -> Self {
        Self {
            reads: Vec::new(),
            writes: Vec::new(),
        }
    }

    /// Merge two accesses (union of all reads/writes)
    pub fn merge(&self, other: &SystemAccess) -> SystemAccess {
        let mut reads = Vec::with_capacity(self.reads.len() + other.reads.len());
        let mut writes = Vec::with_capacity(self.writes.len() + other.writes.len());

        // Add our reads/writes first
        reads.extend_from_slice(&self.reads);
        writes.extend_from_slice(&self.writes);

        // Add other's reads if not already present
        for read in &other.reads {
            if !reads.contains(read) {
                reads.push(*read);
            }
        }

        // Add other's writes if not already present
        for write in &other.writes {
            if !writes.contains(write) {
                writes.push(*write);
            }
        }

        SystemAccess { reads, writes }
    }

    /// Check if this access conflicts with another
    pub fn conflicts_with(&self, other: &SystemAccess) -> bool {
        // Conflict if:
        // - Both write to same component
        // - One writes, other reads same component

        for write in &self.writes {
            if other.writes.contains(write) {
                return true; // Both write
            }
            if other.reads.contains(write) {
                return true; // One writes, other reads
            }
        }

        for write in &other.writes {
            if self.reads.contains(write) {
                return true; // Other writes, we read
            }
        }

        false
    }

    /// Check if two systems can run in parallel
    pub fn can_run_parallel(&self, other: &SystemAccess) -> bool {
        !self.conflicts_with(other)
    }
}

/// System trait
pub trait System: Send + Sync {
    /// Get system access metadata
    fn access(&self) -> SystemAccess;

    /// Get system name
    fn name(&self) -> &'static str;

    /// Run system logic against the world
    fn run(&mut self, world: &mut World) -> Result<()>;
}

/// Boxed system
pub type BoxedSystem = Box<dyn System>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_access_conflicts() {
        let mut access1 = SystemAccess::empty();
        access1.writes.push(TypeId::of::<i32>());

        let mut access2 = SystemAccess::empty();
        access2.writes.push(TypeId::of::<i32>());

        assert!(access1.conflicts_with(&access2));
    }

    #[test]
    fn test_system_access_no_conflicts() {
        let mut access1 = SystemAccess::empty();
        access1.reads.push(TypeId::of::<i32>());

        let mut access2 = SystemAccess::empty();
        access2.reads.push(TypeId::of::<i32>());

        assert!(!access1.conflicts_with(&access2));
    }

    #[derive(Default)]
    struct DummySystem;

    impl System for DummySystem {
        fn access(&self) -> SystemAccess {
            SystemAccess::empty()
        }

        fn name(&self) -> &'static str {
            "dummy_system"
        }

        fn run(&mut self, world: &mut World) -> Result<()> {
            // Spawn and immediately despawn to ensure mutable access works
            let entity = world.spawn((42i32,))?;
            world.despawn(entity).ok();
            Ok(())
        }
    }

    #[test]
    fn test_system_run_signature() {
        let mut world = World::new();
        let mut system = DummySystem;
        system.run(&mut world).expect("system should run");
    }
}
