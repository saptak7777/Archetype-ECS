// Copyright 2024 Saptak Santra
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Integration tests for Phase 1 ECS

#[cfg(test)]
mod tests {
    #![allow(dead_code)]
    #![allow(clippy::module_inception)]
    use crate::{
        CommandBuffer, Executor, Query, QueryState, Schedule, System, SystemAccess, World,
    };
    use crate::{EcsError, Result};
    use std::any::TypeId;

    #[test]
    fn test_basic_spawn_despawn() -> Result<()> {
        let mut world = World::new();

        #[allow(dead_code)]
        #[derive(Debug)]
        struct Position {
            x: f32,
            y: f32,
        }

        // Spawn entity
        let entity = world.spawn((Position { x: 1.0, y: 2.0 },))?;
        assert!(world.get_entity_location(entity).is_some());

        // Despawn entity
        world.despawn(entity)?;
        world.flush_removals()?; // Process deferred removals
        assert!(world.get_entity_location(entity).is_none());
        Ok(())
    }

    #[test]
    fn test_spawn_despawn_errors() -> Result<()> {
        let mut world = World::new();

        #[derive(Debug)]
        struct Position {
            x: f32,
            y: f32,
        }

        let entity = world.spawn((Position { x: 1.0, y: 2.0 },)).unwrap();

        // Despawn should succeed
        world.despawn(entity)?;
        world.flush_removals()?; // Process deferred removals

        // Double despawn should fail
        assert!(world.despawn(entity).is_err());
        Ok(())
    }

    #[test]
    fn test_archetype_segregation() {
        let mut world = World::new();

        #[allow(dead_code)]
        struct A;
        #[allow(dead_code)]
        struct B;
        #[allow(dead_code)]
        struct C;

        // Spawn entities with different component sets
        world.spawn((A, B)).unwrap();
        world.spawn((A, C)).unwrap();
        world.spawn((B, C)).unwrap();
        world.spawn((A, B, C)).unwrap();

        // Should create at least 4 archetypes (+ empty one)
        assert!(world.archetype_count() >= 4);
    }

    #[test]
    fn test_entity_location_tracking() {
        let mut world = World::new();

        #[allow(dead_code)]
        struct Comp;

        let entity = world.spawn((Comp,)).unwrap();
        let location = world.get_entity_location(entity).unwrap();

        assert_eq!(location.archetype_id, 1); // First non-empty archetype
        assert_eq!(location.archetype_row, 0); // First row
    }

    #[test]
    fn test_entity_exists() -> Result<()> {
        let mut world = World::new();

        struct Comp;

        let entity = world.spawn((Comp,)).unwrap();
        assert!(world.entity_exists(entity));

        world.despawn(entity)?;
        world.flush_removals()?; // Process deferred removals
        assert!(!world.entity_exists(entity));
        Ok(())
    }

    #[test]
    fn test_multiple_components() {
        let mut world = World::new();

        #[allow(dead_code)]
        #[derive(Debug, Clone, Copy)]
        struct Position {
            x: f32,
            y: f32,
        }

        #[allow(dead_code)]
        #[derive(Debug, Clone, Copy)]
        struct Velocity {
            x: f32,
            y: f32,
        }

        #[allow(dead_code)]
        #[derive(Debug, Clone, Copy)]
        struct Health(u32);

        // Spawn with 3 components
        let e1 = world
            .spawn((
                Position { x: 0.0, y: 0.0 },
                Velocity { x: 1.0, y: 1.0 },
                Health(100),
            ))
            .unwrap();

        assert!(world.entity_exists(e1));

        // Spawn with 2 components
        let e2 = world
            .spawn((Position { x: 5.0, y: 5.0 }, Health(50)))
            .unwrap();

        assert!(world.entity_exists(e2));
        assert_eq!(world.archetype_count(), 3); // empty + 3comp + 2comp
    }

    #[test]
    fn test_entity_count() -> Result<()> {
        let mut world = World::new();

        struct Comp;

        assert_eq!(world.entity_count(), 0);

        let entities: Vec<_> = (0..10).map(|_| world.spawn((Comp,)).unwrap()).collect();

        assert_eq!(world.entity_count(), 10);

        for entity in entities {
            world.despawn(entity)?;
        }
        world.flush_removals()?; // Process deferred removals
        assert_eq!(world.entity_count(), 0);
        Ok(())
    }

    #[test]
    fn test_recycled_entity_count() -> Result<()> {
        let mut world = World::new();

        struct Comp;

        let e1 = world.spawn((Comp,)).unwrap();
        assert_eq!(world.recycled_entity_count(), 0);

        world.despawn(e1)?;
        world.flush_removals()?; // Process deferred removals
        assert_eq!(world.recycled_entity_count(), 1);

        // Spawn again should reuse the ID
        let e2 = world.spawn((Comp,)).unwrap();
        assert_eq!(world.recycled_entity_count(), 0);
        // Slotmap keys are opaque; ensure entity now exists again
        assert!(world.entity_exists(e2));
        Ok(())
    }

    #[test]
    fn test_world_clear() {
        let mut world = World::new();

        struct Comp;

        for _ in 0..100 {
            let _ = world.spawn((Comp,));
        }

        assert_eq!(world.entity_count(), 100);

        world.clear();

        assert_eq!(world.entity_count(), 0);
        assert_eq!(world.archetype_count(), 1); // Just empty archetype
    }

    #[test]
    fn test_memory_stats() {
        let mut world = World::new();

        struct Comp;

        for _ in 0..1000 {
            let _ = world.spawn((Comp,));
        }

        let stats = world.memory_stats();
        assert!(stats.total_memory > 0);
        assert!(stats.entity_index_memory > 0);
    }

    #[test]
    fn test_query_state_creation() {
        let mut world = World::new();

        #[derive(Debug)]
        struct Position {
            x: f32,
        }

        #[allow(dead_code)]
        #[derive(Debug)]
        struct Velocity {
            x: f32,
        }

        // Spawn some entities
        for i in 0..10 {
            let _ = world.spawn((Position { x: i as f32 }, Velocity { x: 1.0 }));
        }

        let state = QueryState::<(&Position, &Velocity)>::new(&world);
        assert!(state.matched_archetype_count() > 0);
    }

    #[test]
    fn test_query_count() {
        let mut world = World::new();

        struct Comp;

        for _ in 0..10 {
            let _ = world.spawn((Comp,));
        }

        let query = Query::<&Comp>::new(&world);
        assert_eq!(query.count(), 10);
    }

    #[test]
    fn test_command_buffer_capacity() {
        let buffer = CommandBuffer::with_capacity(100);
        assert!(buffer.is_empty());
    }

    #[test]
    fn test_large_scale_spawn() {
        let mut world = World::new();

        #[derive(Debug)]
        struct Position {
            x: f32,
        }

        for i in 0..10_000 {
            let _ = world.spawn((Position { x: i as f32 },));
        }

        assert_eq!(world.entity_count(), 10_000);
    }

    #[test]
    fn test_mixed_spawn_patterns() {
        let mut world = World::new();

        #[allow(dead_code)]
        #[derive(Debug, Clone, Copy)]
        struct A(i32);
        #[allow(dead_code)]
        #[derive(Debug, Clone, Copy)]
        struct B(i32);
        #[allow(dead_code)]
        #[derive(Debug, Clone, Copy)]
        struct C(i32);

        for i in 0..100 {
            let _ = world.spawn((A(i), B(i)));
            let _ = world.spawn((A(i), C(i)));
            let _ = world.spawn((B(i), C(i)));
            let _ = world.spawn((A(i), B(i), C(i)));
            let _ = world.spawn((A(i),));
        }

        assert!(world.archetype_count() >= 5);
    }

    #[test]
    fn test_get_component_api() {
        let mut world = World::new();

        #[derive(Debug, PartialEq)]
        struct Position {
            x: f32,
            y: f32,
        }

        let entity = world
            .spawn((Position { x: 1.0, y: 2.0 },))
            .expect("spawn should succeed");

        let pos = world
            .get_component::<Position>(entity)
            .expect("component should exist");
        assert_eq!(pos.x, 1.0);
        assert_eq!(pos.y, 2.0);
    }

    #[test]
    fn test_get_components_tuple_api() {
        let mut world = World::new();

        #[derive(Debug, PartialEq)]
        struct Position {
            x: f32,
        }

        #[derive(Debug, PartialEq)]
        struct Velocity {
            x: f32,
        }

        let entity = world
            .spawn((Position { x: 3.0 }, Velocity { x: 4.0 }))
            .expect("spawn should succeed");

        let (pos, vel) = world
            .get_components::<(&Position, &Velocity)>(entity)
            .expect("components should exist");

        assert_eq!(pos.x, 3.0);
        assert_eq!(vel.x, 4.0);
    }

    #[test]
    fn test_get_component_mut_api() {
        let mut world = World::new();

        #[derive(Debug, PartialEq)]
        struct Position {
            x: f32,
        }

        let entity = world
            .spawn((Position { x: 0.0 },))
            .expect("spawn should succeed");

        {
            let pos = world
                .get_component_mut::<Position>(entity)
                .expect("component should exist");
            pos.x = 42.0;
        }

        let pos = world
            .get_component::<Position>(entity)
            .expect("component should exist");
        assert_eq!(pos.x, 42.0);
    }

    #[test]
    fn test_get_components_mut_tuple_api() {
        let mut world = World::new();

        #[derive(Debug, PartialEq)]
        struct Position(f32);

        #[derive(Debug, PartialEq)]
        struct Velocity(f32);

        let entity = world
            .spawn((Position(1.0), Velocity(2.0)))
            .expect("spawn should succeed");

        {
            let (pos, vel) = world
                .get_components_mut::<(&mut Position, &mut Velocity)>(entity)
                .expect("components should exist");
            pos.0 += 1.0;
            vel.0 += 2.0;
        }

        let (pos, vel) = world
            .get_components::<(&Position, &Velocity)>(entity)
            .expect("components should exist");
        assert_eq!(pos.0, 2.0);
        assert_eq!(vel.0, 4.0);
    }

    #[test]
    fn test_query_mut_iteration() {
        let mut world = World::new();

        #[derive(Debug, PartialEq)]
        struct Position(f32);

        #[derive(Debug, PartialEq)]
        struct Velocity(f32);

        for i in 0..10 {
            let _ = world
                .spawn((Position(i as f32), Velocity(1.0)))
                .expect("spawn should succeed");
        }

        {
            let mut query = world.query_mut::<(&mut Position, &mut Velocity)>();
            for (pos, vel) in query.iter() {
                pos.0 += vel.0;
                vel.0 += 1.0;
            }
        }

        let query = Query::<(&Position, &Velocity)>::new(&world);
        assert!(query.count() > 0);
        for (pos, vel) in query.iter() {
            assert!(pos.0 >= 1.0);
            assert!(vel.0 >= 2.0);
        }
    }

    #[derive(Debug, Default, Clone)]
    struct LogComponent {
        entries: Vec<&'static str>,
    }

    struct LoggingSystem {
        name: &'static str,
    }

    impl System for LoggingSystem {
        fn access(&self) -> SystemAccess {
            let mut access = SystemAccess::empty();
            access.writes.push(TypeId::of::<LogComponent>());
            access
        }

        fn name(&self) -> &'static str {
            self.name
        }

        fn run(&mut self, world: &mut World) -> Result<()> {
            let mut query = world.query_mut::<&mut LogComponent>();
            for log in query.iter() {
                log.entries.push(self.name);
            }
            Ok(())
        }
    }

    struct FailingSystem;

    impl System for FailingSystem {
        fn access(&self) -> SystemAccess {
            SystemAccess::empty()
        }

        fn name(&self) -> &'static str {
            "failing_system"
        }

        fn run(&mut self, _world: &mut World) -> Result<()> {
            Err(EcsError::ScheduleError("intentional failure".into()))
        }
    }

    #[test]
    fn test_executor_runs_systems_in_order() {
        let mut world = World::new();
        let entity = world
            .spawn((LogComponent::default(),))
            .expect("spawn log entity");

        let schedule = Schedule::new()
            .with_system(Box::new(LoggingSystem { name: "first" }))
            .with_system(Box::new(LoggingSystem { name: "second" }))
            .build()
            .expect("build schedule");

        let mut executor = Executor::new(schedule);
        executor
            .execute_frame(&mut world)
            .expect("executor should run");

        let log = world
            .get_component::<LogComponent>(entity)
            .expect("log component exists");
        assert_eq!(log.entries, vec!["first", "second"]);

        let profile = executor.profile().expect("profiling data available");
        assert_eq!(profile.system_timings.len(), 2);
        assert_eq!(profile.system_timings[0].name, "first");
    }

    #[test]
    fn test_executor_propagates_errors_and_stops() {
        let mut world = World::new();
        let entity = world
            .spawn((LogComponent::default(),))
            .expect("spawn log entity");

        let schedule = Schedule::new()
            .with_system(Box::new(LoggingSystem { name: "first" }))
            .with_system(Box::new(FailingSystem))
            .with_system(Box::new(LoggingSystem { name: "second" }))
            .build()
            .expect("build schedule");

        let mut executor = Executor::new(schedule);
        let result = executor.execute_frame(&mut world);
        assert!(result.is_err(), "executor should propagate system error");

        let log = world
            .get_component::<LogComponent>(entity)
            .expect("log component exists");
        assert_eq!(log.entries, vec!["first"]);
    }
}
