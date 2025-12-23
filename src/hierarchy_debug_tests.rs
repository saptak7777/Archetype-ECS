#[cfg(test)]
mod hierarchy_debug_tests {
    use crate::hierarchy::{Children, Parent};
    use crate::transform::LocalTransform;
    use crate::World;

    #[test]
    fn test_add_component_minimal() {
        let mut world = World::new();

        // Spawn entity with LocalTransform
        let entity = world.spawn((LocalTransform::identity(),));
        println!("Spawned entity: {:?}", entity);
        println!(
            "Entity exists before add_component: {}",
            world.entity_locations.contains_key(entity)
        );

        // Try to add Parent component
        let result = world.add_component(entity, Parent::new(entity));

        match result {
            Ok(_) => {
                println!("Successfully added Parent component");
                println!(
                    "Entity exists after add_component: {}",
                    world.entity_locations.contains_key(entity)
                );
                assert!(
                    world.has_component::<Parent>(entity),
                    "Parent component not found after add"
                );
            }
            Err(e) => {
                println!("Failed to add component: {:?}", e);
                println!(
                    "Entity exists after failed add_component: {}",
                    world.entity_locations.contains_key(entity)
                );
                panic!("add_component failed: {:?}", e);
            }
        }
    }

    #[test]
    fn test_double_add_component() {
        let mut world = World::new();

        let parent = world.spawn((LocalTransform::identity(),));
        let child = world.spawn((LocalTransform::identity(),));

        println!("Parent: {:?}, Child: {:?}", parent, child);
        println!(
            "Parent exists: {}",
            world.entity_locations.contains_key(parent)
        );
        println!(
            "Child exists: {}",
            world.entity_locations.contains_key(child)
        );

        // First add_component
        println!("\n=== Adding Parent to child ===");
        world
            .add_component(child, Parent::new(parent))
            .expect("Failed to add Parent");
        println!(
            "Parent exists after first add: {}",
            world.entity_locations.contains_key(parent)
        );
        println!(
            "Child exists after first add: {}",
            world.entity_locations.contains_key(child)
        );

        // Second add_component
        println!("\n=== Adding Children to parent ===");
        let result = world.add_component(parent, Children::new());
        match result {
            Ok(_) => println!("Successfully added Children"),
            Err(e) => {
                println!("Failed to add Children: {:?}", e);
                println!(
                    "Parent exists: {}",
                    world.entity_locations.contains_key(parent)
                );
                panic!("Second add_component failed: {:?}", e);
            }
        }
    }
}
