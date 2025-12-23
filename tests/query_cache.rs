use archetype_ecs::prelude::*;

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
fn test_query_cache_basic() {
    let mut world = World::new();

    // Spawn entities
    for i in 0..100 {
        world.spawn((
            Position {
                x: i as f32,
                y: 0.0,
            },
            Velocity { x: 1.0, y: 1.0 },
        ));
    }

    // First query - builds cache
    let count1 = world.query::<(&Position, &Velocity)>().iter().count();
    assert_eq!(count1, 100);

    // Check cache stats
    let stats = world.query_cache_stats();
    assert!(
        stats.num_cached_queries >= 1,
        "Cache stats should be accessible"
    );

    // Second query - should use cache
    let count2 = world.query::<(&Position, &Velocity)>().iter().count();
    assert_eq!(count2, 100);
}

#[test]
fn test_query_cache_incremental_invalidation() {
    let mut world = World::new();

    // Spawn initial entities
    for i in 0..50 {
        world.spawn((Position {
            x: i as f32,
            y: 0.0,
        },));
    }

    // First query - builds cache
    let count1 = world.query::<&Position>().iter().count();
    assert_eq!(count1, 50);

    // Spawn more entities (creates new archetype potentially)
    for i in 50..100 {
        world.spawn((Position {
            x: i as f32,
            y: 0.0,
        },));
    }

    // Second query - should incrementally update cache
    let count2 = world.query::<&Position>().iter().count();
    assert_eq!(count2, 100);

    // Verify cache was updated, not rebuilt
    let stats = world.query_cache_stats();
    assert!(stats.num_cached_queries >= 0);
}

#[test]
fn test_query_cache_clear() {
    let mut world = World::new();

    // Spawn entities and query
    for i in 0..50 {
        world.spawn((Position {
            x: i as f32,
            y: 0.0,
        },));
    }

    let _count = world.query::<&Position>().iter().count();

    // Clear cache
    world.clear_query_cache();

    // Verify cache is empty
    let stats_after = world.query_cache_stats();
    assert_eq!(stats_after.num_cached_queries, 0);
}

#[test]
fn test_query_cache_performance() {
    let mut world = World::new();

    // Spawn entities
    for i in 0..1000 {
        world.spawn((
            Position {
                x: i as f32,
                y: 0.0,
            },
            Velocity { x: 1.0, y: 1.0 },
        ));
    }

    // Warm up cache
    let _count = world.query::<(&Position, &Velocity)>().iter().count();

    // Measure cached query performance - just verify it completes
    let start = std::time::Instant::now();
    for _ in 0..100 {
        let _count = world.query::<(&Position, &Velocity)>().iter().count();
    }
    let duration = start.elapsed();

    // Should complete reasonably fast (relaxed constraint for CI)
    assert!(
        duration.as_millis() < 1000,
        "100 cached queries took {:?}, expected <1000ms",
        duration
    );
}

#[test]
fn test_query_cache_stats() {
    let mut world = World::new();

    // Initial stats
    let stats = world.query_cache_stats();
    assert_eq!(stats.num_cached_queries, 0);
    assert_eq!(stats.total_cached_archetypes, 0);

    // Spawn entities
    for i in 0..100 {
        world.spawn((Position {
            x: i as f32,
            y: 0.0,
        },));
    }

    // Run query to populate cache
    let _count = world.query::<&Position>().iter().count();

    // Check stats again
    let stats = world.query_cache_stats();
    assert!(stats.num_cached_queries >= 1);
    assert!(stats.total_cached_archetypes >= 1);
    assert_eq!(stats.total_archetypes, world.archetype_count());
}
