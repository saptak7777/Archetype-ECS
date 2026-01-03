#[cfg(test)]
mod tests {
    use archetype_ecs::prelude::*;

    #[derive(Debug, PartialEq)]
    struct Position {
        x: f32,
        y: f32,
    }

    #[test]
    fn test_despawn_bug_reproduction() {
        let mut world = World::new();

        // Spawn an entity
        let entity = world.spawn_entity((Position { x: 1.0, y: 2.0 },));

        // Verify it exists
        assert!(
            world.get_component::<Position>(entity).is_some(),
            "Entity should exist before despawn"
        );

        // Despawn it
        world.despawn(entity).unwrap();

        // Verify it's gone
        assert!(
            world.get_component::<Position>(entity).is_none(),
            "Entity should NOT exist after despawn"
        );

        // Verify query doesn't find it
        let mut count = 0;
        for _ in world.query_mut::<&Position>() {
            count += 1;
        }
        assert_eq!(count, 0, "Query should find 0 entities");
    }
}
