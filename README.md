# Archetype ECS

**A high-performance, strictly-typed, archetype-based Entity Component System for Rust.**

Archetype ECS is what happens when you take ECS seriously, strip out the "game engine" bloat, and focus on raw performance. It's the ECS equivalent of a sports car - fast, focused, and doesn't come with cup holders.

## What This Is (Probably)

Archetype ECS is a prototype ECS library for Rust game engines. It follows Unix philosophy (does one thing, hopefully well) and provides the ECS components you'd need to integrate into a larger engine.

**Fair Warning**: This is a library component, not a complete solution. You'll need to bring your own rendering, materials, lighting, and scene management. Think of it as an engine block for your car - it works, but you can't drive it alone.

## Core Concepts

### What It Does
✅ **High Performance**: Cache-friendly archetype storage with SoA (Structure of Arrays) layout
✅ **Parallel Execution**: Automatic multi-threaded system scheduling with dependency resolution
✅ **Reactive Queries**: Efficient `Changed<T>` and `Added<T>` filters for change detection
✅ **Hierarchy System**: First-class parent-child relationships with transform propagation
✅ **Serialization**: Built-in JSON serialization for entities, components, and worlds
✅ **Hot Reload**: System hot-reloading for development workflow (Code)
✅ **Profiling**: Integrated `tracing` instrumentation for performance analysis
✅ **Error Types**: Asset loading error types for your asset manager

### What It Doesn't Do
❌ Rendering (but works perfectly with ash_renderer)
❌ Physics (but integrates seamlessly with particle_accelerator)
❌ Asset Management (we intentionally separate data from logic)
❌ Materials/Lighting (that's your renderer's job)

---

## Integration Examples

### With particle_accelerator (Physics)
```rust
use archetype_ecs::prelude::*;
use particle_accelerator::{PhysicsWorld, RigidBody, Collider};

fn setup_physics_world() -> Result<()> {
    let mut world = World::new();
    let mut physics = PhysicsWorld::new();
    
    // Spawn physics entities
    let ball = world.spawn_entity((
        Position { x: 0.0, y: 10.0, z: 0.0 },
        Velocity { x: 5.0, y: 0.0, z: 0.0 },
        RigidBody::dynamic(),
        Collider::sphere(1.0),
    ));
    
    // Physics system integration
    physics.update(&mut world, 0.016)?;
    
    Ok(())
}
```

### With archetype_assets (Asset Management)
```rust
use archetype_ecs::prelude::*;
use archetype_assets::{AssetManager, Handle};

fn setup_assets() -> Result<()> {
    let mut world = World::new();
    let mut assets = AssetManager::new();
    
    // Load assets
    let texture_handle = assets.load::<Texture>("player.png")?;
    let mesh_handle = assets.load::<Mesh>("player.obj")?;
    
    // Store asset manager in world
    world.insert_resource(assets);
    
    // Spawn entity with asset handles
    let player = world.spawn_entity((
        Position::default(),
        MeshComponent { handle: mesh_handle },
        TextureComponent { handle: texture_handle },
    ));
    
    Ok(())
}
```

## Quick Start

### Installation
```toml
[dependencies]
archetype_ecs = "1.2.0"
glam = "0.30"            # Required for transform types
```

### 1. Basic ECS Usage (The Foundation)
```rust
use archetype_ecs::prelude::*;

fn main() -> Result<()> {
    let mut world = World::new();
    
    // Spawn some entities
    let player = world.spawn_entity((
        Position { x: 0.0, y: 0.0 },
        Velocity { x: 1.0, y: 0.0 },
    ));
    
    // Query and update
    for (pos, vel) in world.query_mut::<(&mut Position, &Velocity)>() {
        pos.x += vel.x;
        pos.y += vel.y;
    }
    
    Ok(())
}
```

### 2. Systems & Hot Reloading (The Real Deal)
We support hot-reloading of systems using the `hot_reload` module. This allows you to iterate on game logic without restarting the application.

```rust
use archetype_ecs::prelude::*;

struct MovementSystem;

impl System for MovementSystem {
    fn name(&self) -> &'static str { "Movement" }
    
    fn access(&self) -> SystemAccess {
        SystemAccess::new()
            .read::<Velocity>()
            .write::<Position>()
    }
    
    fn run(&mut self, world: &mut World) -> Result<()> {
        for (pos, vel) in world.query_mut::<(&mut Position, &Velocity)>() {
            pos.x += vel.x * 0.016; // 60 FPS dt
        }
        Ok(())
    }
}
```

## Performance

Archetype ECS is designed for speed because slow ECS libraries are sad ECS libraries.
- **Iteration**: Linear memory access pattern allows efficient prefetching. Your CPU will thank you.
- **Change Detection**: Bitset-based filtering makes reactive systems negligible in cost.
- **Fragmentation**: Archetype moves can be expensive; we recommend using distinct components for distinct states (e.g. `Walking` vs `Flying`).

*(Benchmarks running on Intel Core i5-11400f, 100k entities)*
- **Simple Iteration**: ~1-2ns / entity (very fast)
- **Composed Query**: ~2-3ns / entity (still fast)
- **Parallel Dispatch**: Scales linearly with cores for disjoint data

## Known Limitations (Being Honest)

### What We Don't Do (Because We're Focused)
- ❌ Rendering (use ash_renderer, wgpu, or your own solution)
- ❌ Materials/Lighting (that's your engine's job, not ours)
- ❌ Asset loading (we provide error types, you bring the loader; see `archetype_assets`)
- ❌ Code generation (we prefer explicit typed code over macro magic)

### Current Limitations
- **Archetype Moves**: Adding/removing components moves entities between arrays. This is O(N) for the components being moved.
- **No Built-in Networking**: We focus on single-machine performance.
- **Serialization**: JSON support is MVP. Binary serialization is planned.

## Troubleshooting

### "My entities aren't spawning!"
Check:
- Are you using `spawn_entity()` instead of the deprecated `spawn()`?
- Did you remember to import the prelude?
- Is your Bundle implementation correct?

### "My queries aren't finding anything!"
Check:
- Did you actually spawn entities with those components?
- Are you using the right query type (`query` vs `query_mut`)?
- Did you despawn the entities and forget to flush removals?

## Contributing

Contributions are welcome. I appreciate:
- Clean code (clippy is your friend)
- Performance justifications ("It's faster" -> Show me the benchmark)
- Honest PR descriptions

## License

Apache-2.0. Because nobody likes monsters.

## Acknowledgments

This library wouldn't exist without:
- The Rust gamedev community (for putting up with ECS experimentation)
- glam (for making math not painful)
- serde (for making serialization not a nightmare)
