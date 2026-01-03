use archetype_ecs::prelude::*;
use archetype_ecs::{Executor, Schedule, System, SystemAccess};
use std::sync::{Arc, Mutex};

#[derive(Default, Clone)]
struct Counter(Arc<Mutex<Vec<&'static str>>>);

struct StageTester {
    name: &'static str,
    counter: Counter,
}

impl System for StageTester {
    fn run(&mut self, _world: &mut World, _commands: &mut CommandBuffer) -> Result<()> {
        let mut vec = self.counter.0.lock().unwrap();
        vec.push(self.name);
        Ok(())
    }

    fn name(&self) -> &'static str {
        self.name
    }

    fn accesses(&self) -> SystemAccess {
        SystemAccess::new()
    }
}

#[test]
fn test_named_stages_ordering() {
    let mut world = World::new();
    let counter = Counter::default();

    let mut schedule = Schedule::new();
    schedule.add_stage("First").unwrap();
    schedule.add_stage("Second").unwrap();
    schedule.add_stage_dependency("Second", "First").unwrap();

    schedule
        .add_system_to_stage(
            "Second",
            Box::new(StageTester {
                name: "Stage2_System",
                counter: counter.clone(),
            }),
        )
        .unwrap();

    schedule
        .add_system_to_stage(
            "First",
            Box::new(StageTester {
                name: "Stage1_System",
                counter: counter.clone(),
            }),
        )
        .unwrap();

    let mut executor = Executor::new(&mut schedule);
    executor.execute_frame(&mut world).unwrap();

    let results = counter.0.lock().unwrap();
    assert_eq!(results[0], "Stage1_System");
    assert_eq!(results[1], "Stage2_System");
}

#[test]
fn test_stage_parallel_execution() {
    let mut world = World::new();
    let mut schedule = Schedule::new();

    // Add two non-conflicting systems to the same stage
    schedule.add_stage("Main").unwrap();

    // We'll use a component to verify they ran
    #[derive(Debug, Clone, Copy, PartialEq)]
    struct A(i32);
    #[derive(Debug, Clone, Copy, PartialEq)]
    struct B(i32);

    struct SystemA;
    impl System for SystemA {
        fn run(
            &mut self,
            world: &mut World,
            _commands: &mut archetype_ecs::command::CommandBuffer,
        ) -> Result<()> {
            world.query_mut::<&mut A>().iter().for_each(|a| {
                a.0 += 1;
            });
            Ok(())
        }
        fn name(&self) -> &'static str {
            "SystemA"
        }
        fn accesses(&self) -> SystemAccess {
            SystemAccess::new().write::<A>()
        }
    }

    struct SystemB;
    impl System for SystemB {
        fn run(
            &mut self,
            world: &mut World,
            _commands: &mut archetype_ecs::command::CommandBuffer,
        ) -> Result<()> {
            world.query_mut::<&mut B>().iter().for_each(|b| {
                b.0 += 1;
            });
            Ok(())
        }
        fn name(&self) -> &'static str {
            "SystemB"
        }
        fn accesses(&self) -> SystemAccess {
            SystemAccess::new().write::<B>()
        }
    }

    schedule
        .add_system_to_stage("Main", Box::new(SystemA))
        .unwrap();
    schedule
        .add_system_to_stage("Main", Box::new(SystemB))
        .unwrap();

    world.spawn_entity((A(10), B(20)));

    let mut executor = Executor::new(&mut schedule);
    executor.execute_frame_parallel(&mut world).unwrap();

    let mut count = 0;
    world.query::<(&A, &B)>().iter().for_each(|(a, b)| {
        assert_eq!(a.0, 11);
        assert_eq!(b.0, 21);
        count += 1;
    });
    assert_eq!(count, 1);
}
