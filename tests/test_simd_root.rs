#[test]
fn test_simd_chunks() {
    use archetype_ecs::{QueryState, World};

    #[derive(Debug, Copy, Clone)]
    #[allow(dead_code)] // Test struct for SIMD operations
    struct Position {
        x: f32,
        y: f32,
        z: f32,
    }

    let mut world = World::new();
    for _ in 0..100 {
        world.spawn_entity((Position {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        },));
    }

    let mut query = QueryState::<&mut Position>::new(&world);
    let chunks = query.iter_simd_chunks::<Position>(&mut world);

    assert!(!chunks.is_empty());
    println!("Found {} chunks", chunks.len());
}
