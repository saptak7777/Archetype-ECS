#[cfg(test)]
mod tests {
    use crate::component::Component;
    use crate::prelude::*;
    use crate::world::World;

    #[test]
    fn test_debug_add_component() {
        let mut world = World::new();
        #[derive(Debug, PartialEq)]
        struct TestComp(i32);

        // Manual component impl if needed, but blanket covers it
        // impl Component for TestComp {}

        let e = world.spawn(()); // Spawn empty
        world.add_component(e, TestComp(42)).unwrap();

        assert!(
            world.has_component::<TestComp>(e),
            "Component missing after add"
        );
        let c = world
            .get_component::<TestComp>(e)
            .expect("get_component failed");
        assert_eq!(c.0, 42);
    }
}
