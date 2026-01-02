#[cfg(test)]
mod remove_component_tests {
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

    #[derive(Debug, Clone, Copy, PartialEq)]
    struct Health(i32);

    /// Test: Remove middle component preserves others
    /// This was the primary failure case in v1.1.6
    #[test]
    fn test_remove_component_middle_preserves_data() {
        let mut world = World::new();

        let entity = world.spawn_entity((
            Position { x: 1.0, y: 2.0 },
            Velocity { x: 3.0, y: 4.0 },
            Health(100),
        ));

        // Before removal - verify all components exist
        assert_eq!(
            world.get_component::<Position>(entity).unwrap(),
            &Position { x: 1.0, y: 2.0 }
        );
        assert_eq!(
            world.get_component::<Velocity>(entity).unwrap(),
            &Velocity { x: 3.0, y: 4.0 }
        );
        assert_eq!(world.get_component::<Health>(entity).unwrap(), &Health(100));

        // Remove middle component
        world
            .remove_component::<Velocity>(entity)
            .expect("remove failed");

        // After removal - CRITICAL: Verify other data intact
        assert_eq!(
            world.get_component::<Position>(entity).unwrap(),
            &Position { x: 1.0, y: 2.0 },
            "Position data lost during removal!"
        );
        assert!(
            world.get_component::<Velocity>(entity).is_none(),
            "Velocity should be removed"
        );
        assert_eq!(
            world.get_component::<Health>(entity).unwrap(),
            &Health(100),
            "Health data lost during removal!"
        );
    }

    /// Test: Multiple entities with same removal pattern
    #[test]
    fn test_remove_component_multiple_entities() {
        let mut world = World::new();

        let e1 = world.spawn_entity((Position { x: 1.0, y: 1.0 }, Velocity { x: 1.0, y: 1.0 }));
        let e2 = world.spawn_entity((Position { x: 2.0, y: 2.0 }, Velocity { x: 2.0, y: 2.0 }));
        let e3 = world.spawn_entity((Position { x: 3.0, y: 3.0 }, Velocity { x: 3.0, y: 3.0 }));

        // Remove from all
        world.remove_component::<Velocity>(e1).unwrap();
        world.remove_component::<Velocity>(e2).unwrap();
        world.remove_component::<Velocity>(e3).unwrap();

        // Verify all retained Position data
        assert_eq!(world.get_component::<Position>(e1).unwrap().x, 1.0);
        assert_eq!(world.get_component::<Position>(e2).unwrap().x, 2.0);
        assert_eq!(world.get_component::<Position>(e3).unwrap().x, 3.0);
    }

    /// Test: Stress test - sequential removals
    #[test]
    fn test_remove_component_sequential_stress() {
        let mut world = World::new();

        let entity = world.spawn_entity((
            Position { x: 1.0, y: 1.0 },
            Velocity { x: 2.0, y: 2.0 },
            Health(100),
        ));

        // Remove sequentially and verify each step
        world.remove_component::<Health>(entity).unwrap();
        assert_eq!(world.get_component::<Position>(entity).unwrap().x, 1.0);
        assert_eq!(world.get_component::<Velocity>(entity).unwrap().x, 2.0);

        world.remove_component::<Velocity>(entity).unwrap();
        assert_eq!(world.get_component::<Position>(entity).unwrap().x, 1.0);

        world.remove_component::<Position>(entity).unwrap();
        assert!(world.get_component::<Position>(entity).is_none());
    }

    /// Test: Remove from empty component set should fail gracefully
    #[test]
    fn test_remove_nonexistent_component() {
        let mut world = World::new();

        let entity = world.spawn_entity((Position { x: 1.0, y: 1.0 },));

        // Attempting to remove non-existent component should error
        let result = world.remove_component::<Health>(entity);
        assert!(
            result.is_err(),
            "Should fail when removing non-existent component"
        );
    }
}
