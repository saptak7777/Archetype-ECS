// Quick test to isolate the add_component issue
use archetype_ecs::{World, LocalTransform, Parent};

#[test]
fn test_add_component_simple() {
    let mut world = World::new();
    let entity = world.spawn((LocalTransform::identity(),));

    // This should work - adding a new component
    let result = world.add_component(entity, Parent::new(entity));
    assert!(result.is_ok(), "add_component failed: {:?}", result.err());

    assert!(world.has_component::<Parent>(entity));
}
