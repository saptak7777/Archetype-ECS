use crate::entity::EntityId;
use crate::error::Result;
use crate::hierarchy::{Children, Parent};
use crate::system::{System, SystemAccess};
use crate::transform::{GlobalTransform, LocalTransform};
use crate::world::World;
use std::any::TypeId;

/// System that updates global transforms based on hierarchy
pub struct HierarchyUpdateSystem;

impl HierarchyUpdateSystem {
    pub fn new() -> Self {
        Self
    }

    /// Update transforms recursively starting from an entity
    fn _update_transform_recursive(
        &self,
        _entity: EntityId,
        _parent_global: &GlobalTransform,
        _world: &mut World,
    ) -> Result<()> {
        // Simplified implementation stub
        // In a full implementation, this would:
        // 1. Get local transform from entity
        // 2. Calculate global = parent_global + local
        // 3. Update global transform on entity
        // 4. Recursively update children

        Ok(())
    }
}

impl System for HierarchyUpdateSystem {
    fn name(&self) -> &'static str {
        "HierarchyUpdateSystem"
    }

    fn access(&self) -> SystemAccess {
        let mut access = SystemAccess::empty();
        access.reads.push(TypeId::of::<LocalTransform>());
        access.reads.push(TypeId::of::<Parent>());
        access.reads.push(TypeId::of::<Children>());
        access.writes.push(TypeId::of::<GlobalTransform>());
        access
    }

    fn run(&mut self, _world: &mut World) -> Result<()> {
        // In a simple implementation, we'd iterate through all entities
        // and find roots (entities without Parent), then update recursively
        //
        // This is a simplified stub - a real implementation would:
        // 1. Query for all entities without Parent component
        // 2. For each root, call update_transform_recursive
        // 3. Handle the borrow checker issues properly

        Ok(())
    }
}

impl Default for HierarchyUpdateSystem {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper to establish parent-child relationships
pub struct HierarchyBuilder;

impl HierarchyBuilder {
    /// Attach child entity to parent
    ///
    /// This establishes a parent-child relationship by:
    /// 1. Adding Parent component to child
    /// 2. Adding child to parent's Children component
    pub fn attach(_world: &mut World, _parent: EntityId, _child: EntityId) -> Result<()> {
        // In a full implementation, this would:
        // 1. Add Parent(parent) component to child
        // 2. Get or create Children component on parent and add child
        // 3. Mark transforms as dirty for update

        // For now, this is a stub
        // You would need world.add_component() or similar API

        Ok(())
    }

    /// Detach child from parent
    pub fn detach(_world: &mut World, _parent: EntityId, _child: EntityId) -> Result<()> {
        // In a full implementation, this would:
        // 1. Remove Parent component from child
        // 2. Remove child from parent's Children component

        Ok(())
    }

    /// Create hierarchy structure
    /// Attaches multiple children to a parent
    pub fn create_hierarchy(
        world: &mut World,
        parent: EntityId,
        children: Vec<EntityId>,
    ) -> Result<()> {
        for child in children {
            Self::attach(world, parent, child)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hierarchy_system_creation() {
        let system = HierarchyUpdateSystem::new();
        assert_eq!(system.name(), "HierarchyUpdateSystem");
    }

    #[test]
    fn test_hierarchy_system_access() {
        let system = HierarchyUpdateSystem::new();
        let access = system.access();

        // Should read LocalTransform, Parent, Children
        assert_eq!(access.reads.len(), 3);
        // Should write GlobalTransform
        assert_eq!(access.writes.len(), 1);
    }
}
