use archetype_ecs::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq)]
struct A(i32);
#[derive(Debug, Clone, Copy, PartialEq)]
struct B(i32);

struct SpawnerSystem;
impl archetype_ecs::System for SpawnerSystem {
    fn run(&mut self, _world: &mut World, commands: &mut CommandBuffer) -> Result<()> {
        commands.spawn(|world| {
            world.spawn_entity((A(1), B(2)));
            Ok(())
        });
        Ok(())
    }
    fn name(&self) -> &'static str {
        "SpawnerSystem"
    }
    fn accesses(&self) -> archetype_ecs::system::SystemAccess {
        archetype_ecs::system::SystemAccess::new()
    }
}

struct MutatorSystem;
impl archetype_ecs::System for MutatorSystem {
    fn run(&mut self, world: &mut World, commands: &mut CommandBuffer) -> Result<()> {
        // Use Entity marker to get EntityId in query
        for (entity, _a) in world.query::<(Entity, &A)>().iter() {
            commands.add_component(entity, B(10));
        }
        Ok(())
    }
    fn name(&self) -> &'static str {
        "MutatorSystem"
    }
    fn accesses(&self) -> archetype_ecs::system::SystemAccess {
        archetype_ecs::system::SystemAccess::new().read::<A>()
    }
}

#[test]
fn test_deferred_spawn_and_apply() {
    let mut world = World::new();
    let mut schedule = Schedule::new();
    schedule.add_system(Box::new(SpawnerSystem));

    let mut executor = Executor::new(&mut schedule);

    // After 1 frame, the entity should be spawned
    executor.execute_frame(&mut world).unwrap();

    let count = world.query::<(&A, &B)>().iter().count();
    assert_eq!(count, 1);
}

#[test]
fn test_deferred_mutation_sequential() {
    let mut world = World::new();
    let entity = world.spawn_entity((A(1),));

    let mut schedule = Schedule::new();
    schedule.add_system(Box::new(MutatorSystem));

    let mut executor = Executor::new(&mut schedule);

    // MutatorSystem adds B(10) via command buffer
    executor.execute_frame(&mut world).unwrap();

    let b = world.get_component::<B>(entity);
    assert!(b.is_some());
    assert_eq!(b.unwrap().0, 10);
}

#[test]
fn test_deferred_mutation_parallel_stages() {
    let mut world = World::new();

    let mut schedule = Schedule::new();
    schedule.add_stage("Stage1").unwrap();
    schedule.add_stage("Stage2").unwrap();
    schedule.add_stage_dependency("Stage2", "Stage1").unwrap();

    schedule
        .add_system_to_stage("Stage1", Box::new(SpawnerSystem))
        .unwrap();
    schedule
        .add_system_to_stage("Stage2", Box::new(MutatorSystem))
        .unwrap();

    let mut executor = Executor::new(&mut schedule);

    // execute_frame applies commands after each stage
    executor.execute_frame(&mut world).unwrap();

    // Verify Stage1 spawned AND Stage2 mutated it
    let count = world.query::<(&A, &B)>().iter().count();
    assert_eq!(count, 1);

    let (_entity, b) = world.query::<(Entity, &B)>().iter().next().unwrap();
    assert_eq!(b.0, 10);
}

struct DespawnerSystem;
impl archetype_ecs::System for DespawnerSystem {
    fn run(&mut self, world: &mut World, commands: &mut CommandBuffer) -> Result<()> {
        for (entity, _) in world.query::<(Entity, &A)>().iter() {
            commands.despawn(entity);
        }
        Ok(())
    }
    fn name(&self) -> &'static str {
        "DespawnerSystem"
    }
    fn accesses(&self) -> archetype_ecs::system::SystemAccess {
        archetype_ecs::system::SystemAccess::new().read::<A>()
    }
}

#[test]
fn test_deferred_despawn() {
    let mut world = World::new();
    world.spawn_entity((A(1),));

    let mut schedule = Schedule::new();
    schedule.add_system(Box::new(DespawnerSystem));

    let mut executor = Executor::new(&mut schedule);
    executor.execute_frame(&mut world).unwrap();

    assert_eq!(world.entity_count(), 0);
}
