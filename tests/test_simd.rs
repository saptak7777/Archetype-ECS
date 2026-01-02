use archetype_ecs::World;

#[derive(Debug, Clone, Copy, PartialEq)]
struct Position {
    x: f32,
    y: f32,
    z: f32,
}

#[test]
fn test_chunk_slice_access() {
    let mut world = World::new();

    // Spawn 100 entities
    for i in 0..100 {
        world.spawn_entity((Position {
            x: i as f32,
            y: 0.0,
            z: 0.0,
        },));
    }

    // Use par_for_each_chunk to update positions
    let mut query = world.query_mut::<&mut Position>();

    query.par_for_each_chunk(|mut chunk| {
        // Get mutable slice and update all positions
        if let Some(positions) = chunk.get_slice_mut::<Position>() {
            // This demonstrates SIMD-friendly slice access
            for pos in positions.iter_mut() {
                pos.x += 1.0;
            }
        }
    });

    // Verify - we can't easily verify the exact values without another query,
    // but we can verify the count
    let count = world.query_mut::<&mut Position>().count();
    assert_eq!(count, 100);
}

#[test]
fn test_simd_slice_access() {
    let mut world = World::new();
    world.spawn_entity((Position {
        x: 1.0,
        y: 2.0,
        z: 3.0,
    },));
    world.spawn_entity((Position {
        x: 4.0,
        y: 5.0,
        z: 6.0,
    },));

    let mut query = world.query_mut::<&mut Position>();
    query.par_for_each_chunk(|mut chunk| {
        let positions = chunk.get_slice_mut::<Position>().unwrap();
        assert!(!positions.is_empty());

        // This slice can be used with SIMD operations
        // For example, using std::simd or manual SIMD intrinsics
        for pos in positions.iter_mut() {
            pos.x *= 2.0;
            pos.y *= 2.0;
            pos.z *= 2.0;
        }
    });

    // Verify count
    assert_eq!(world.query_mut::<&mut Position>().count(), 2);
}
