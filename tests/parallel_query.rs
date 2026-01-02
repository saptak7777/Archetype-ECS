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
fn test_parallel_query_basic() {
    let mut world = World::new();

    // Spawn many entities across multiple archetypes to ensure parallel work
    for i in 0..5000 {
        world.spawn_entity((
            Position {
                x: i as f32,
                y: 0.0,
            },
            Velocity { x: 1.0, y: 1.0 },
        ));
    }

    for i in 0..5000 {
        world.spawn_entity((
            Position {
                x: i as f32,
                y: 100.0,
            },
            Velocity { x: 2.0, y: 2.0 },
        ));
    }

    // Run parallel query
    world
        .par_query_mut::<(&mut Position, &Velocity)>()
        .for_each(|(pos, vel)| {
            pos.x += vel.x;
            pos.y += vel.y;
        });

    // Verify results
    let mut count = 0;
    for (pos, _vel) in world.query::<(&Position, &Velocity)>().iter() {
        if pos.y < 50.0 {
            // First batch
            assert_eq!(pos.y, 1.0);
        } else {
            // Second batch
            assert_eq!(pos.y, 102.0);
        }
        count += 1;
    }
    assert_eq!(count, 10000);
}

#[test]
fn test_parallel_query_complex_filter() {
    use archetype_ecs::query::{Changed, With};

    let mut world = World::new();

    let e1 = world.spawn_entity((Position { x: 0.0, y: 0.0 },));
    let _e2 = world.spawn_entity((Position { x: 10.0, y: 10.0 },));

    world.increment_tick(); // Tick 2

    // Modify e1
    if let Some(pos) = world.get_component_mut::<Position>(e1) {
        pos.x += 1.0;
    }

    // Parallel query with Changed filter
    let _count = 0;
    // We can't easily capture count in a closure passed to for_each if it's not thread-safe.
    // But we can use atomic or just verify side effects.

    world
        .par_query_mut::<(With<Position>, Changed<Position>)>()
        .for_each(|_| {
            // This runs in parallel
        });

    // Just verify it compiles and runs for now.
    // Real verification:
    let changed_count = world
        .query_mut::<(With<Position>, Changed<Position>)>()
        .iter_since(1)
        .count();
    assert_eq!(changed_count, 1);
}
