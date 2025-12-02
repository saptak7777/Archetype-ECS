use aaa_ecs::{
    impl_reflect,
    query::{Added, CachedQuery, Changed, Read},
    reflection::{Reflect, TypeRegistry},
    World,
};

#[derive(Debug, Clone, Copy, Default, PartialEq)]
struct Position {
    x: f32,
    y: f32,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
struct Velocity {
    x: f32,
    y: f32,
}

impl_reflect!(Position);
impl_reflect!(Velocity);

#[test]
fn test_change_detection() {
    let mut world = World::new();
    let mut query = CachedQuery::<(Read<Position>, Changed<Position>)>::new(&world);

    // Spawn entity
    let _entity = world.spawn((Position { x: 0.0, y: 0.0 },)).unwrap();

    // First run: should be detected as changed (added is also changed)
    let count = query.iter(&world).count();
    assert_eq!(count, 1);

    // Second run: no changes
    let count = query.iter(&world).count();
    assert_eq!(count, 0);

    // Increment world tick (simulating new frame)
    world.increment_tick();

    // Modify component
    {
        let mut q_mut = world.query_mut::<&mut Position>();
        for pos in q_mut.iter() {
            pos.x += 1.0;
        }
    }

    // Third run: should be detected
    let count = query.iter(&world).count();
    assert_eq!(count, 1);
}

#[test]
fn test_added_detection() {
    let mut world = World::new();
    let mut query = CachedQuery::<(Read<Position>, Added<Position>)>::new(&world);

    // Spawn entity
    world.spawn((Position { x: 0.0, y: 0.0 },)).unwrap();

    // First run: detected
    assert_eq!(query.iter(&world).count(), 1);

    // Second run: not detected
    assert_eq!(query.iter(&world).count(), 0);

    // Modify component
    {
        let mut q_mut = world.query_mut::<&mut Position>();
        for pos in q_mut.iter() {
            pos.x += 1.0;
        }
    }
    world.increment_tick();

    // Third run: not detected (changed but not added)
    assert_eq!(query.iter(&world).count(), 0);
}

#[test]
fn test_reflection() {
    let mut registry = TypeRegistry::new();
    registry.register::<Position>();
    registry.register::<Velocity>();

    let pos = Position { x: 10.0, y: 20.0 };
    let boxed_pos: Box<dyn Reflect> = Box::new(pos);

    assert_eq!(boxed_pos.type_name(), std::any::type_name::<Position>());

    // Test downcast
    if let Some(p) = boxed_pos.as_any().downcast_ref::<Position>() {
        assert_eq!(p.x, 10.0);
        assert_eq!(p.y, 20.0);
    } else {
        panic!("Failed to downcast");
    }

    // Test apply
    let mut pos2 = Position::default();
    pos2.apply(boxed_pos.as_ref());
    assert_eq!(pos2.x, 10.0);
    assert_eq!(pos2.y, 20.0);

    // Test clone
    let cloned = boxed_pos.reflect_clone();
    if let Some(p) = cloned.as_any().downcast_ref::<Position>() {
        assert_eq!(p.x, 10.0);
        assert_eq!(p.y, 20.0);
    } else {
        panic!("Failed to clone");
    }
}
