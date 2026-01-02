# Archetype ECS

**A high-performance, strictly-typed, archetype-based Entity Component System for Rust.**

Archetype ECS differs from other Rust ECS libraries by focusing on a "pure" data-oriented design with zero "game engine" bloat. It provides a robust kernel for building complex simulations and game engines, offering industry-standard features like parallel iteration, reactive queries, and hierarchical transforms out of the box.

## features

- **🚀 High Performance**: Cache-friendly archetype storage with SoA (Structure of Arrays) layout.
- **⚡ Parallel Execution**: Automatic multi-threaded system scheduling with dependency resolution.
- **🔍 Reactive Queries**: Efficient `Changed<T>`, `Added<T>`, and `Removed<T>` filters.
- **🌳 Hierarchy System**: First-class parent-child relationship management with efficient transform propagation using `glam`.
- **💾 Serialization**: Built-in JSON serialization for entities, components, and entire worlds.
- **📦 Asset Management**: Typed asset handles, async-ready loaders, and hot-reloading support.
- **🧩 Modularity**: Zero "engine" assumptions. Use it for rendering, physics, or data processing.

## Quick Start

Add to `Cargo.toml`:

```toml
[dependencies]
archetype_ecs = { git = "https://github.com/saptak7777/archetype_ecs" }
glam = "0.25" # Recommended for math types
```

### Basic Example

```rust
use archetype_ecs::prelude::*;
use glam::{Vec3, Quat};

// 1. Define Components
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
struct Velocity {
    pub value: Vec3,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
struct Player {
    pub name: &'static str,
}

// 2. Define a System
struct MovementSystem;

impl System for MovementSystem {
    fn name(&self) -> &'static str { "Movement" }

    fn access(&self) -> SystemAccess {
        SystemAccess::new()
            .read::<Velocity>()
            .write::<LocalTransform>()
    }

    fn run(&mut self, world: &mut World) -> Result<()> {
        // Query for entities with Velocity and LocalTransform
        // We use query_mut to modify the transform
        for (vel, transform) in world.query_mut::<(&Velocity, &mut LocalTransform)>() {
            transform.position += vel.value * 0.016; // Assume 60 FPS dt
        }
        Ok(())
    }
}

fn main() -> Result<()> {
    let mut world = World::new();

    // 3. Spawn Entities
    // We use standard glam types for positions
    let player_id = world.spawn((
        Player { name: "Hero" },
        LocalTransform::with_position(Vec3::new(0.0, 1.0, 0.0)),
        GlobalTransform::identity(), // Required for hierarchy participation
        Velocity { value: Vec3::new(1.0, 0.0, 1.0) },
    ));

    // 4. Run Systems
    let mut movement = MovementSystem;
    movement.run(&mut world)?;

    // 5. Verify Result
    let player_pos = world.get_component::<GlobalTransform>(player_id).unwrap().position;
    println!("Player is now at: {}", player_pos);

    Ok(())
}
```

## Core Concepts

### World & Archetypes
Data is stored in **Archetypes**, grouping entities with the precise same set of components together. This guarantees contiguous memory for iteration, minimizing cache misses.

```rust
// Entities are just IDs.
let entity = world.spawn((ComponentA, ComponentB));
```

### Queries & Filters
Queries are cached for O(1) access after the first run.

```rust
// Basic iteration
for (pos, vel) in world.query::<(&Position, &Velocity)>() { ... }

// Mutable iteration
for (pos, vel) in world.query_mut::<(&mut Position, &Velocity)>() { ... }

// Change detection (Reactive)
for pos in world.query::<&Position, Changed<Position>>() {
    println!("Position changed: {:?}", pos);
}
```

### Hierarchy & Transforms
The `hierarchy` module provides optimized component-based scene graphs.

```rust
let parent = world.spawn((LocalTransform::identity(), GlobalTransform::identity()));
let child = world.spawn((LocalTransform::identity(), GlobalTransform::identity()));

// Attach child to parent
world.attach(parent, child)?;

// HierarchyUpdateSystem will automatically propagate transforms
```

### Resources
Resources are typed singletons for global state (e.g., time, config, window handles).

```rust
// Insert resource
world.insert_resource(Time { delta: 0.016 });

// Get resource
let time = world.resource::<Time>().unwrap();

// Lazy initialization (inserts if missing)
let time = world.get_or_insert_with(|| Time::default());

// Init-only (errors if exists - prevents accidental overwrites)
world.init_resource(Config::new())?;

// Systems can declare resource dependencies
impl System for MySystem {
    fn access(&self) -> SystemAccess {
        SystemAccess::new()
            .read::<Velocity>()
            .resource::<Time>()  // Tracks resource access
    }
}
```

### Parallel Systems
The `ParallelExecutor` distributes systems across a thread pool, ensuring thread safety via runtime borrow checking.

```rust
let mut executor = ParallelExecutor::new(vec![
    Box::new(PhysicsSystem),
    Box::new(RenderSystem),
    Box::new(AudioSystem),
]);

// Automatically runs independent systems in parallel
executor.execute_parallel(&mut world)?;
```

## Performance

Archetype ECS is designed for speed.
- **Iteration**: Linear memory access pattern allows efficient prefetching.
- **Change Detection**: Bitset-based filtering makes reactive systems negligible in cost.
- **Fragmentation**: Archetype moves are somewhat expensive, so distinct "States" (like `Walking` vs `Flying` components) should be used judiciously.

*(Benchmarks running on Intel Core i5-11400f, 100k entities)*
- **Simple Iteration**: ~0.5ns / entity
- **Composed Query**: ~1.2ns / entity
- **Parallel Dispatch**: Scales linearly with cores for disjoint data.

## Standard Compliance

- **Math**: Uses [`glam`](https://crates.io/crates/glam) for SIMD-accelerated linear algebra.
- **Serialization**: Uses [`serde`](https://crates.io/crates/serde) for universal IO.
- **Async**: Compatible with standard async runtimes for non-ECS tasks (like asset loading).

## License

Apache-2.0.
