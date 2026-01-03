//! System trait and access metadata

use crate::error::Result;
use crate::world::{UnsafeWorldCell, World};
use std::any::TypeId;

/// System ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SystemId(pub u32); // Made public

/// Component ID wrapper
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ComponentId(pub TypeId);

impl ComponentId {
    pub fn of<T: 'static>() -> Self {
        Self(TypeId::of::<T>())
    }
}

impl From<TypeId> for ComponentId {
    fn from(id: TypeId) -> Self {
        Self(id)
    }
}

/// System access metadata
#[derive(Debug, Clone)]
pub struct SystemAccess {
    pub reads: Vec<ComponentId>,
    pub writes: Vec<ComponentId>,
}

impl Default for SystemAccess {
    fn default() -> Self {
        Self::new()
    }
}

impl SystemAccess {
    /// Create empty access
    pub fn empty() -> Self {
        Self {
            reads: Vec::new(),
            writes: Vec::new(),
        }
    }

    /// Create new access (alias for empty)
    pub fn new() -> Self {
        Self::empty()
    }

    /// Declare read access to a component
    pub fn read<T: 'static>(mut self) -> Self {
        self.reads.push(ComponentId::of::<T>());
        self
    }

    /// Declare write access to a component
    pub fn write<T: 'static>(mut self) -> Self {
        self.writes.push(ComponentId::of::<T>());
        self
    }

    /// Merge two accesses (union of all reads/writes)
    pub fn merge(&self, other: &SystemAccess) -> SystemAccess {
        // Pre-allocate to avoid multiple reallocations during extend
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
        // Check write conflicts
        // NOTE: Could use HashSet for O(1) lookup, but small N makes linear scan faster
        for write in &self.writes {
            if other.writes.contains(write) {
                return true;
            }
            if other.reads.contains(write) {
                return true;
            }
        }

        for write in &other.writes {
            if self.reads.contains(write) {
                return true;
            }
        }

        false
    }

    /// Check if two systems can run in parallel
    pub fn can_run_parallel(&self, other: &SystemAccess) -> bool {
        !self.conflicts_with(other)
    }

    /// Declare read access to a resource
    pub fn resource<R: 'static>(mut self) -> Self {
        self.reads.push(ComponentId::of::<R>());
        self
    }

    /// Declare write access to a resource
    pub fn resource_mut<R: 'static>(mut self) -> Self {
        self.writes.push(ComponentId::of::<R>());
        self
    }
}

/// System trait
pub trait System: Send + Sync {
    /// Get system access metadata
    fn accesses(&self) -> SystemAccess;

    /// Get system name
    fn name(&self) -> &'static str;

    /// Run system with world access
    fn run(
        &mut self,
        world: &mut World,
        commands: &mut crate::command::CommandBuffer,
    ) -> Result<()>;

    /// Run system in parallel using UnsafeWorldCell
    ///
    /// # Safety
    /// Caller must ensure disjoint access to components as declared in `accesses()`.
    unsafe fn run_parallel(
        &mut self,
        world: UnsafeWorldCell,
        commands: &mut crate::command::CommandBuffer,
    ) -> Result<()> {
        // Default implementation: cast to &mut World
        // This is safe IF the scheduler has guaranteed disjointness.
        let world_mut = &mut *world.world_ptr();
        self.run(world_mut, commands)
    }
}

/// Boxed system
pub type BoxedSystem = Box<dyn System>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_access_conflicts() {
        let mut access1 = SystemAccess::empty();
        access1.writes.push(ComponentId::of::<i32>());

        let mut access2 = SystemAccess::empty();
        access2.writes.push(ComponentId::of::<i32>());

        assert!(access1.conflicts_with(&access2));
    }

    #[test]
    fn test_system_access_no_conflicts() {
        let mut access1 = SystemAccess::empty();
        access1.reads.push(ComponentId::of::<i32>());

        let mut access2 = SystemAccess::empty();
        access2.reads.push(ComponentId::of::<i32>());

        assert!(!access1.conflicts_with(&access2));
    }

    #[derive(Default)]
    struct DummySystem;

    impl System for DummySystem {
        fn accesses(&self) -> SystemAccess {
            SystemAccess::empty()
        }

        fn name(&self) -> &'static str {
            "dummy_system"
        }

        fn run(
            &mut self,
            world: &mut World,
            _commands: &mut crate::command::CommandBuffer,
        ) -> Result<()> {
            // Spawn and immediately despawn to ensure mutable access works
            let entity = world.spawn_entity((42i32,));
            world.despawn(entity).ok();
            Ok(())
        }
    }

    #[test]
    fn test_system_run_signature() {
        let mut world = World::new();
        let mut commands = crate::command::CommandBuffer::new();
        let mut system = DummySystem;
        system
            .run(&mut world, &mut commands)
            .expect("system should run");
    }
}
