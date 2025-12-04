# Archetype ECS

A high-performance Entity Component System (ECS) library for Rust.

[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)

## Installation

```toml
[dependencies]
archetype_ecs = "1.1.3"
```

## Quick Start

```rust
use archetype_ecs::prelude::*;

// Components are plain structs
struct Position { x: f32, y: f32 }
struct Velocity { dx: f32, dy: f32 }

fn main() -> Result<()> {
    let mut world = World::new();
    
    // Spawn entities
    for i in 0..1000 {
        world.spawn((
            Position { x: i as f32, y: 0.0 },
            Velocity { dx: 1.0, dy: 0.0 },
        ))?;
    }
    
    // Query and update components
    for (pos, vel) in world.query_mut::<(&mut Position, &Velocity)>() {
        pos.x += vel.dx;
        pos.y += vel.dy;
    }
    
    Ok(())
}
```

## Querying Entities

### Basic Queries

```rust
// Mutable query - use .iter() or IntoIterator
let mut query = world.query_mut::<(&mut Position, &Velocity)>();
for (pos, vel) in query.iter() {
    pos.x += vel.dx;
}

// Or directly (IntoIterator)
for (pos, vel) in world.query_mut::<(&mut Position, &Velocity)>() {
    pos.x += vel.dx;
}
```

### Getting Entity IDs

Use the `Entity` marker to get entity IDs during iteration:

```rust
use archetype_ecs::prelude::*;

let mut to_delete = Vec::new();

for (entity, health) in world.query_mut::<(Entity, &Health)>() {
    if health.current <= 0.0 {
        to_delete.push(entity);  // Track entity for deletion
    }
}

// Delete entities after iteration
for entity in to_delete {
    world.despawn(entity)?;
}
```

### Mixed Mutability

Read some components while writing others:

```rust
// Read Position, write Velocity
for (pos, vel) in world.query_mut::<(&Position, &mut Velocity)>() {
    vel.dx = -pos.x * 0.1;  // Use pos to calculate new velocity
}
```

### Direct Component Access

Access components on a specific entity:

```rust
// Immutable access
if let Some(pos) = world.get_component::<Position>(entity) {
    println!("Position: ({}, {})", pos.x, pos.y);
}

// Mutable access
if let Some(pos) = world.get_component_mut::<Position>(entity) {
    pos.x += 10.0;
}
```

## Resources (Global State)

Resources are typed singletons for global state:

```rust
struct GameTime { delta: f32, elapsed: f32 }

// Insert resource
world.insert_resource(GameTime { delta: 0.016, elapsed: 0.0 });

// Read resource
if let Some(time) = world.resource::<GameTime>() {
    println!("Elapsed: {}", time.elapsed);
}

// Mutate resource
if let Some(time) = world.resource_mut::<GameTime>() {
    time.elapsed += time.delta;
}
```

## Query Filters

Filter entities by component presence:

```rust
use archetype_ecs::query::{With, Without, Changed};

// Only entities WITH a Visible component
let query = Query::<&Position, With<Visible>>::new(&world);
for pos in query.iter() { /* ... */ }

// Only entities WITHOUT a Dead component
let query = Query::<&Position, Without<Dead>>::new(&world);

// Only entities where Position changed this frame
let query = Query::<&Position, Changed<Position>>::new(&world);
```

## Systems & Scheduling

```rust
use archetype_ecs::{System, SystemAccess};
use std::any::TypeId;

struct PhysicsSystem;

impl System for PhysicsSystem {
    fn name(&self) -> &'static str { "PhysicsSystem" }
    
    fn access(&self) -> SystemAccess {
        let mut access = SystemAccess::empty();
        access.writes.push(TypeId::of::<Position>());
        access.reads.push(TypeId::of::<Velocity>());
        access
    }
    
    fn run(&mut self, world: &mut World) -> Result<()> {
        for (pos, vel) in world.query_mut::<(&mut Position, &Velocity)>() {
            pos.x += vel.dx;
            pos.y += vel.dy;
        }
        Ok(())
    }
}
```

### Parallel Execution

```rust
use archetype_ecs::parallel::ParallelExecutor;

let systems: Vec<Box<dyn System>> = vec![
    Box::new(PhysicsSystem),
    Box::new(RenderSystem),
];

let mut executor = ParallelExecutor::new(systems);
executor.execute_parallel(&mut world)?;
```

## SIMD & Chunk Processing

Process entities in parallel with SIMD-friendly access:

```rust
let mut query = world.query_mut::<&mut Position>();
query.par_for_each_chunk(|mut chunk| {
    if let Some(positions) = chunk.get_slice_mut::<Position>() {
        for pos in positions.iter_mut() {
            pos.x += 1.0;
            pos.y += 1.0;
        }
    }
});
```

## Batch Operations

```rust
// Spawn many entities efficiently
let entities = world.spawn_batch((0..1000).map(|i| {
    (Position { x: i as f32, y: 0.0 }, Velocity { dx: 1.0, dy: 0.0 })
}))?;
```

## Performance

| Operation | Time | Scale |
|-----------|------|-------|
| Query Iteration | 11.1 µs | 10,000 entities |
| Entity Spawn | 42.3 µs | 1,000 entities |
| Parallel Execution | 3.1 ms | Multi-core |

## License

Copyright 2024 Saptak Santra. Licensed under Apache-2.0.
