use crate::entity::EntityId;
use crate::error::Result;
use crate::hierarchy::{Children, Parent};
use crate::system::{System, SystemAccess};
use crate::transform::{GlobalTransform, LocalTransform};
use crate::world::World;

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

    fn accesses(&self) -> SystemAccess {
        let mut access = SystemAccess::empty();
        access
            .reads
            .push(crate::system::ComponentId::of::<LocalTransform>());
        access
            .reads
            .push(crate::system::ComponentId::of::<Parent>());
        access
            .reads
            .push(crate::system::ComponentId::of::<Children>());
        access
            .writes
            .push(crate::system::ComponentId::of::<GlobalTransform>());
        access
    }

    fn run(
        &mut self,
        world: &mut World,
        _commands: &mut crate::command::CommandBuffer,
    ) -> Result<()> {
        // Strategy:
        // 1. Find all "roots" (Entities with LocalTransform but NO Parent)
        // 2. Recursively calculate GlobalTransform from top down
        //
        // Borrow checker: We can't query and mutate recursively easily.
        // Solution: Collect a flattened list of updates, then apply them.

        // Step 1: Find roots
        // This is O(N) over all entities without caching, but acceptable for now
        let mut roots = Vec::new();

        // We need a query for (Entity, &LocalTransform, Without<Parent>)
        // But our query API is basic. Let's iterate all entities with LocalTransform check for Parent.
        // Optimization: In Phase 2, use Without<Parent> filter.

        // Since we can't iterate and check parent easily without filters,
        // let's grab all entities with LocalTransform.
        // NOTE: This assumes standard query iteration availability
        for (entity, _) in world
            .query::<(crate::query::Entity, &LocalTransform)>()
            .iter()
        {
            if !world.has_component::<Parent>(entity) {
                roots.push(entity);
            }
        }

        // Step 2: Compute updates (BFS/DFS)
        // We store (entity_id, new_global_transform)
        let mut updates: Vec<(EntityId, GlobalTransform)> = Vec::with_capacity(roots.len() * 4);

        // Stack for DFS: (entity_id, parent_global_transform)
        let mut stack = Vec::with_capacity(64);

        for root in roots {
            let root_local = match world.get_component::<LocalTransform>(root) {
                Some(l) => *l,
                None => continue, // Should be impossible given query
            };

            // Root global is just local (conceptually local * identity)
            let root_global =
                GlobalTransform::from_local(&GlobalTransform::identity(), &root_local);

            updates.push((root, root_global));
            stack.push((root, root_global));

            // Process children
            while let Some((parent_id, parent_global)) = stack.pop() {
                // Get children of this parent
                if let Some(children) = world.get_component::<Children>(parent_id) {
                    // Clone indices to avoid borrowing
                    // PERF: This clone is unfortunate but safe.
                    // With a specialized hierarchy iterator we could avoid it.
                    let child_ids = children.get_children();

                    for child_id in child_ids {
                        if let Some(child_local) = world.get_component::<LocalTransform>(child_id) {
                            let child_global =
                                GlobalTransform::from_local(&parent_global, child_local);
                            updates.push((child_id, child_global));

                            // Push to stack to process *its* children
                            stack.push((child_id, child_global));
                        }
                    }
                }
            }
        }

        // Step 3: Apply updates
        // This is safe because we're done reading
        for (entity, global) in updates {
            // If entity already has correct global, we could skip writing (change detection optimization)
            // For now, simple write.
            if let Some(g) = world.get_component_mut::<GlobalTransform>(entity) {
                *g = global;
            } else {
                // If it doesn't have GlobalTransform, should we add it?
                // Rule: Entities participating in hierarchy MUST have GlobalTransform.
                // BEHAVIOR CHOICE: Fail silently or add?
                // Implicit adding is magic. Better to assume user added it or ignore.
                // Let's ignore to avoid structural changes during update.
            }
        }

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
    pub fn attach(world: &mut World, parent: EntityId, child: EntityId) -> Result<()> {
        // Prevent cycles/self-attachment (basic check)
        if parent == child {
            return Err(crate::error::EcsError::HierarchyError(
                "Cannot attach entity to itself".to_string(),
            ));
        }

        // 1. Add Parent component to child
        // If child already has a parent, we should probably detach first?
        // use strict mode: fail if already has parent
        if world.has_component::<Parent>(child) {
            return Err(crate::error::EcsError::HierarchyError(format!(
                "Entity {child:?} already has a parent"
            )));
        }

        world.add_component(child, Parent::new(parent))?;

        // 2. Add child to parent's Children list
        // If parent doesn't have Children component, add it
        if !world.has_component::<Children>(parent) {
            world.add_component(parent, Children::new())?;
        }

        // We can safely unwrap here because we just ensured it exists or failed add_component
        let children = world
            .get_component_mut::<Children>(parent)
            .ok_or(crate::error::EcsError::EntityNotFound)?; // Should be unreachable

        children.add_child(child);

        Ok(())
    }

    /// Detach child from parent
    pub fn detach(world: &mut World, parent: EntityId, child: EntityId) -> Result<()> {
        // 1. Remove Parent component from child
        // Check if it actually points to us?
        if let Some(p) = world.get_component::<Parent>(child) {
            if p.entity_id() != parent {
                return Err(crate::error::EcsError::HierarchyError(format!(
                    "Entity {child:?} is not a child of {parent:?}"
                )));
            }
        } else {
            return Err(crate::error::EcsError::HierarchyError(format!(
                "Entity {child:?} has no parent"
            )));
        }

        world.remove_component::<Parent>(child)?;

        // 2. Remove child from parent's Children component
        if let Some(children) = world.get_component_mut::<Children>(parent) {
            children.remove_child(child);
            // Optimization: could remove Children component if empty, but maybe not worth the churn
        }

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
        let access = system.accesses();

        // Should read LocalTransform, Parent, Children
        assert_eq!(access.reads.len(), 3);
        // Should write GlobalTransform
        assert_eq!(access.writes.len(), 1);
    }

    #[test]
    fn test_attach_detach() {
        let mut world = World::new();
        let parent = world.spawn_entity((LocalTransform::identity(),));
        let child = world.spawn_entity((LocalTransform::identity(),));

        // Attach
        HierarchyBuilder::attach(&mut world, parent, child).unwrap();

        // Verify components
        assert!(world.has_component::<Parent>(child));
        assert!(world.has_component::<Children>(parent));

        let children = world.get_component::<Children>(parent).unwrap();
        assert!(children.contains(child));

        // Detach
        HierarchyBuilder::detach(&mut world, parent, child).unwrap();
        assert!(!world.has_component::<Parent>(child));

        let children = world.get_component::<Children>(parent).unwrap();
        assert!(!children.contains(child));
    }

    #[test]
    fn test_transform_propagation() {
        let mut world = World::new();
        let parent = world.spawn_entity((
            LocalTransform::with_position(crate::transform::Vec3::new(10.0, 0.0, 0.0)),
            GlobalTransform::identity(),
        ));
        let child = world.spawn_entity((
            LocalTransform::with_position(crate::transform::Vec3::new(5.0, 0.0, 0.0)),
            GlobalTransform::identity(),
        ));

        HierarchyBuilder::attach(&mut world, parent, child).unwrap();

        let mut system = HierarchyUpdateSystem::new();
        let mut commands = crate::command::CommandBuffer::new();
        system.run(&mut world, &mut commands).unwrap();

        let child_global = world.get_component::<GlobalTransform>(child).unwrap();
        // Parent (10,0,0) + Child (5,0,0) = (15,0,0)
        assert!((child_global.position.x - 15.0).abs() < 0.001);
    }
}
