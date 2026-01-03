use archetype_ecs::hierarchy_system::HierarchyUpdateSystem;
use archetype_ecs::prelude::*;
use archetype_ecs::system::System;

#[test]
fn test_hierarchy_single_parent_child() {
    let mut world = World::new();

    // Create parent at (10, 0, 0)
    let parent = world.spawn_entity((
        LocalTransform::with_position(Vec3::new(10.0, 0.0, 0.0)),
        GlobalTransform::identity(),
    ));

    // Create child at local offset (5, 0, 0)
    let child = world.spawn_entity((
        LocalTransform::with_position(Vec3::new(5.0, 0.0, 0.0)),
        GlobalTransform::identity(),
        Parent::new(parent),
    ));

    // Add child to parent's Children component
    let mut children = Children::new();
    children.add_child(child);
    world.add_component(parent, children).unwrap();

    // Run hierarchy system
    let mut system = HierarchyUpdateSystem::new();
    let mut commands = CommandBuffer::new();
    system.run(&mut world, &mut commands).unwrap();
    commands.apply(&mut world).unwrap();

    // Verify child's global transform = parent_global * child_local
    let child_global = world.get_component::<GlobalTransform>(child).unwrap();
    let expected = Vec3::new(15.0, 0.0, 0.0); // 10 + 5

    assert_eq!(child_global.position, expected);
}

#[test]
fn test_hierarchy_multiple_children() {
    let mut world = World::new();

    // Create parent
    let parent = world.spawn_entity((
        LocalTransform::with_position(Vec3::new(10.0, 0.0, 0.0)),
        GlobalTransform::identity(),
    ));

    // Create 3 children at different offsets
    let mut children_component = Children::new();
    let mut child_ids = Vec::new();

    for i in 0..3 {
        let child = world.spawn_entity((
            LocalTransform::with_position(Vec3::new(i as f32, 0.0, 0.0)),
            GlobalTransform::identity(),
            Parent::new(parent),
        ));
        children_component.add_child(child);
        child_ids.push(child);
    }

    world.add_component(parent, children_component).unwrap();

    // Run hierarchy system
    let mut system = HierarchyUpdateSystem::new();
    let mut commands = CommandBuffer::new();
    system.run(&mut world, &mut commands).unwrap();
    commands.apply(&mut world).unwrap();

    // Verify each child's global transform
    for (i, &child_id) in child_ids.iter().enumerate() {
        let child_global = world.get_component::<GlobalTransform>(child_id).unwrap();
        let expected = Vec3::new(10.0 + i as f32, 0.0, 0.0);
        assert_eq!(child_global.position, expected);
    }
}

#[test]
fn test_hierarchy_deep_nesting() {
    let mut world = World::new();

    // Create 10-level hierarchy
    let mut entities = Vec::new();
    for i in 0..10 {
        let local = LocalTransform::with_position(Vec3::new(1.0, 0.0, 0.0));
        let global = GlobalTransform::identity();

        let entity = if i == 0 {
            // Root
            world.spawn_entity((local, global))
        } else {
            // Child of previous
            let parent_id = entities[i - 1];
            world.spawn_entity((local, global, Parent::new(parent_id)))
        };

        entities.push(entity);

        // Add to parent's Children
        if i > 0 {
            let parent_id = entities[i - 1];
            if let Some(children) = world.get_component_mut::<Children>(parent_id) {
                children.add_child(entity);
            } else {
                let mut children = Children::new();
                children.add_child(entity);
                world.add_component(parent_id, children).unwrap();
            }
        }
    }

    // Run hierarchy system
    let mut system = HierarchyUpdateSystem::new();
    let mut commands = CommandBuffer::new();
    system.run(&mut world, &mut commands).unwrap();
    commands.apply(&mut world).unwrap();

    // Verify deepest child is at (10, 0, 0)
    let deepest = entities[9];
    let global = world.get_component::<GlobalTransform>(deepest).unwrap();

    assert_eq!(global.position, Vec3::new(10.0, 0.0, 0.0));
}

#[test]
fn test_hierarchy_reparenting() {
    let mut world = World::new();

    // Create two parents
    let parent_a = world.spawn_entity((
        LocalTransform::with_position(Vec3::new(10.0, 0.0, 0.0)),
        GlobalTransform::identity(),
    ));

    let parent_b = world.spawn_entity((
        LocalTransform::with_position(Vec3::new(20.0, 0.0, 0.0)),
        GlobalTransform::identity(),
    ));

    // Create child initially under parent_a
    let child = world.spawn_entity((
        LocalTransform::with_position(Vec3::new(5.0, 0.0, 0.0)),
        GlobalTransform::identity(),
        Parent::new(parent_a),
    ));

    let mut children_a = Children::new();
    children_a.add_child(child);
    world.add_component(parent_a, children_a).unwrap();

    // Run hierarchy system
    let mut system = HierarchyUpdateSystem::new();
    let mut commands = CommandBuffer::new();
    system.run(&mut world, &mut commands).unwrap();
    commands.apply(&mut world).unwrap();

    // Verify child is at (15, 0, 0)
    let global = world.get_component::<GlobalTransform>(child).unwrap();
    assert_eq!(global.position, Vec3::new(15.0, 0.0, 0.0));

    // Reparent to parent_b
    world.add_component(child, Parent::new(parent_b)).unwrap();

    // Remove from parent_a's children
    if let Some(children) = world.get_component_mut::<Children>(parent_a) {
        children.remove_child(child);
    }

    // Add to parent_b's children
    let mut children_b = Children::new();
    children_b.add_child(child);
    world.add_component(parent_b, children_b).unwrap();

    // Run hierarchy system again
    let mut commands = CommandBuffer::new();
    system.run(&mut world, &mut commands).unwrap();
    commands.apply(&mut world).unwrap();

    // Verify child is now at (25, 0, 0)
    let global = world.get_component::<GlobalTransform>(child).unwrap();
    assert_eq!(global.position, Vec3::new(25.0, 0.0, 0.0));
}

#[test]
fn test_hierarchy_performance_1000_entities() {
    let mut world = World::new();

    // Create wide hierarchy: 1 root, 999 children
    let root = world.spawn_entity((LocalTransform::identity(), GlobalTransform::identity()));

    let mut children_component = Children::new();
    for i in 0..999 {
        let child = world.spawn_entity((
            LocalTransform::with_position(Vec3::new(i as f32, 0.0, 0.0)),
            GlobalTransform::identity(),
            Parent::new(root),
        ));
        children_component.add_child(child);
    }

    world.add_component(root, children_component).unwrap();

    // Benchmark update
    let start = std::time::Instant::now();
    let mut system = HierarchyUpdateSystem::new();
    let mut commands = CommandBuffer::new();
    system.run(&mut world, &mut commands).unwrap();
    commands.apply(&mut world).unwrap();
    let duration = start.elapsed();

    // Should complete in <10ms
    assert!(
        duration.as_millis() < 10,
        "Hierarchy update too slow: {duration:?}"
    );
}

#[test]
fn test_hierarchy_no_parent() {
    let mut world = World::new();

    // Create entity with LocalTransform but no Parent
    let entity = world.spawn_entity((
        LocalTransform::with_position(Vec3::new(5.0, 0.0, 0.0)),
        GlobalTransform::identity(),
    ));

    // Run hierarchy system
    let mut system = HierarchyUpdateSystem::new();
    let mut commands = CommandBuffer::new();
    system.run(&mut world, &mut commands).unwrap();
    commands.apply(&mut world).unwrap();

    // Verify global transform equals local transform (no parent)
    let global = world.get_component::<GlobalTransform>(entity).unwrap();
    assert_eq!(global.position, Vec3::new(5.0, 0.0, 0.0));
}
