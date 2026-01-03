use archetype_ecs::world::World;
use std::any::TypeId;

#[test]
fn test_disjoint_mutable_borrows_via_cell() {
    let mut world = World::new();

    // Spawn some entities
    for i in 0..100 {
        world.spawn_entity((i as f32, i as i32));
    }

    let tick = world.tick();

    // Get unsafe cell
    let cell = unsafe { world.as_unsafe_world_cell() };

    // Simulating two threads accessing disjoint components
    // Thread A: access f32
    // Thread B: access i32

    let arch_count = unsafe { (*cell.world_ptr()).archetype_count() };

    for arch_id in 0..arch_count {
        unsafe {
            let col_f32 = cell.get_column_raw_mut(arch_id, TypeId::of::<f32>());
            let col_i32 = cell.get_column_raw_mut(arch_id, TypeId::of::<i32>());

            assert!(col_f32.is_some() || arch_id == 0); // arch 0 is empty
            assert!(col_i32.is_some() || arch_id == 0);

            if let Some(c) = col_f32 {
                let col = &mut *c;
                col.set_changed_tick(0, tick + 1);
            }

            if let Some(c) = col_i32 {
                let col = &mut *c;
                col.set_changed_tick(0, tick + 1);
            }
        }
    }
}
