# AAA ECS

A high-performance Entity Component System (ECS) written in Rust, designed for game development and performance-critical applications.

## Features

- **Archetype-based storage** for optimal cache locality
- **Type-safe queries** with compile-time guarantees
- **Zero-cost abstractions** with minimal runtime overhead
- **Mutable and immutable queries** for flexible data access
- **Efficient entity management** with O(1) lookups
- **Component bundles** for convenient entity spawning

## Performance

AAA ECS demonstrates significant performance advantages over established libraries like hecs:

| Operation | AAA ECS | HECS | Improvement |
|-----------|---------|------|-------------|
| Entity Spawn (100k) | 2.84ms | 4.41ms | **35% faster** |
| Entity Lookup (100k) | 197µs | 1.51ms | **7.6x faster** |
| Entity Despawn (1k) | 7.25µs | 17.08µs | **2.3x faster** |
| Query Creation (10k) | 43ns | 2.90µs | **67x faster** |

## Quick Start

```rust
use aaa_ecs::World;

#[derive(Debug, Clone)]
struct Position { x: f32, y: f32 }

#[derive(Debug, Clone)]
struct Velocity { x: f32, y: f32 }

let mut world = World::new();

// Spawn entities
let entity = world.spawn((
    Position { x: 0.0, y: 0.0 },
    Velocity { x: 1.0, y: 2.0 }
)).unwrap();

// Query components
for (pos, vel) in world.query::<(&Position, &Velocity)>().iter() {
    println!("Position: ({}, {}), Velocity: ({}, {})", pos.x, pos.y, vel.x, vel.y);
}

// Mutable queries
for (pos, vel) in world.query_mut::<(&mut Position, &mut Velocity)>().iter() {
    pos.x += vel.x;
    pos.y += vel.y;
}
```

## Architecture

AAA ECS uses an archetype-based architecture where entities with the same component types are stored together in memory. This provides excellent cache locality during iteration and enables efficient query processing.

### Key Components

- **World**: Central storage for entities and archetypes
- **Archetype**: Container for entities with identical component types
- **Query**: Type-safe iterator over matching entities
- **Component**: Data attached to entities

## License

Copyright 2024 Saptak Santra

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
