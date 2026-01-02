use archetype_ecs::query::{Added, Changed, Entity, QueryMut, With};
use archetype_ecs::World;

#[derive(Debug, Clone, Copy, PartialEq)]
struct Position {
    x: f32,
    y: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Velocity {
    x: f32,
    y: f32,
}

#[test]
fn test_change_detection_flow() {
    let mut world = World::new();

    // 1. Initial Spawn (Tick = 1)
    let e1 = world.spawn_entity((Position { x: 0.0, y: 0.0 }, Velocity { x: 1.0, y: 1.0 }));
    let e2 = world.spawn_entity((Position { x: 10.0, y: 10.0 },));

    {
        // Everything is "Added" since tick 0
        let mut query = QueryMut::<(Added<Position>,)>::new(&mut world);
        assert_eq!(query.iter_since(0).count(), 2);
    }

    // 2. Frame 2 (Tick = 2)
    world.increment_tick();

    {
        // Nothing added since tick 1
        let mut query = QueryMut::<(Added<Position>,)>::new(&mut world);
        assert_eq!(query.iter_since(1).count(), 0);

        // Nothing changed since tick 1
        let mut query = QueryMut::<(Changed<Position>,)>::new(&mut world);
        assert_eq!(query.iter_since(1).count(), 0);
    }

    // 3. Modify e1 (Tick = 2)
    if let Some(pos) = world.get_component_mut::<Position>(e1) {
        pos.x = 1.0;
    }

    {
        // e1 changed since tick 1
        let mut query = QueryMut::<(Changed<Position>,)>::new(&mut world);
        assert_eq!(query.iter_since(1).count(), 1);

        // e2 did not change
        let mut query = QueryMut::<(With<Position>, Changed<Position>)>::new(&mut world);
        let changed_entities = query.iter_since(1).count();
        assert_eq!(changed_entities, 1);
    }

    // 4. Frame 3 (Tick = 3)
    world.increment_tick();

    // Add component to e2
    world
        .add_component(e2, Velocity { x: 0.0, y: 0.0 })
        .unwrap();

    {
        // Velocity added to e2 since tick 2
        let mut query = QueryMut::<(Added<Velocity>,)>::new(&mut world);
        assert_eq!(query.iter_since(2).count(), 1);
    }
}

#[test]
fn test_complex_change_filter() {
    let mut world = World::new();

    world.spawn_entity((Position { x: 0.0, y: 0.0 },));
    world.spawn_entity((Position { x: 1.0, y: 1.0 },));

    world.increment_tick(); // Tick 2

    // Modify first one
    let (e1, _) = world
        .query_mut::<(Entity, With<Position>)>()
        .iter()
        .next()
        .expect("Entity not found");
    if let Some(pos) = world.get_component_mut::<Position>(e1) {
        pos.x += 1.0;
    }

    // Query for both &Position and Changed<Position>
    let mut query = QueryMut::<(&Position, Changed<Position>)>::new(&mut world);
    let results: Vec<_> = query.iter_since(1).collect();

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].0.x, 1.0); // The modified one
}
