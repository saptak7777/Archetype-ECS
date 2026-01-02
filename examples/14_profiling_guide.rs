//! Example 14: Comprehensive Profiling Guide
//! 
//! This example demonstrates real-world profiling usage patterns:
//! - Basic system profiling setup
//! - Advanced profiling with custom metrics
//! - Batch operation profiling
//! - Query performance analysis
//! - Production-ready profiling configuration

use archetype_ecs::{World, System, SystemAccess, Executor, Schedule};

#[derive(Clone)]
struct Position {
    x: f32,
    y: f32,
}

#[derive(Clone)]
struct Velocity {
    x: f32,
    y: f32,
}

#[derive(Clone)]
struct Health {
    current: f32,
    max: f32,
}

// Basic movement system with profiling
struct MovementSystem;

impl System for MovementSystem {
    fn access(&self) -> SystemAccess {
        let mut access = SystemAccess::empty();
        access.reads.push(std::any::TypeId::of::<Velocity>());
        access.writes.push(std::any::TypeId::of::<Position>());
        access
    }

    fn name(&self) -> &'static str {
        "MovementSystem"
    }

    fn run(&mut self, world: &mut World) -> Result<(), archetype_ecs::error::EcsError> {
        #[cfg(feature = "profiling")]
        let span = tracing::info_span!(
            "movement_system",
            entity_count = world.entity_count()
        );
        
        #[cfg(feature = "profiling")]
        let _guard = span.enter();
        
        let mut _moved_count = 0;
        for (pos, vel) in world.query_mut::<(&mut Position, &Velocity)>().iter() {
            pos.x += vel.x;
            pos.y += vel.y;
            _moved_count += 1;
        }
        
        #[cfg(feature = "profiling")]
        tracing::info!("Moved entities successfully");
        
        Ok(())
    }
}

// Health system with advanced profiling
struct HealthSystem {
    processed_count: std::sync::atomic::AtomicUsize,
}

impl HealthSystem {
    fn new() -> Self {
        Self {
            processed_count: std::sync::atomic::AtomicUsize::new(0),
        }
    }
}

impl System for HealthSystem {
    fn access(&self) -> SystemAccess {
        let mut access = SystemAccess::empty();
        access.writes.push(std::any::TypeId::of::<Health>());
        access
    }

    fn name(&self) -> &'static str {
        "HealthSystem"
    }

    fn run(&mut self, world: &mut World) -> Result<(), archetype_ecs::error::EcsError> {
        let _start = std::time::Instant::now();
        
        #[cfg(feature = "profiling")]
        let span = tracing::info_span!(
            "health_system",
            entity_count = world.entity_count()
        );
        
        #[cfg(feature = "profiling")]
        let _guard = span.enter();
        
        let mut processed = 0;
        for health in world.query_mut::<&mut Health>().iter() {
            if health.current < health.max {
                health.current = (health.current + 0.1).min(health.max);
            }
            processed += 1;
        }
        
        self.processed_count.store(processed, std::sync::atomic::Ordering::Relaxed);
        
        #[cfg(feature = "profiling")]
        tracing::info!(
            target: "performance_metrics",
            "Processed {} health entities",
            processed
        );
        
        Ok(())
    }
}

// Complex query system with profiling
struct QueryAnalysisSystem;

impl QueryAnalysisSystem {
    fn new(_world: &World) -> Self {
        Self
    }
}

impl System for QueryAnalysisSystem {
    fn access(&self) -> SystemAccess {
        let mut access = SystemAccess::empty();
        access.reads.push(std::any::TypeId::of::<Position>());
        access.reads.push(std::any::TypeId::of::<Velocity>());
        access
    }

    fn name(&self) -> &'static str {
        "QueryAnalysisSystem"
    }

    fn run(&mut self, world: &mut World) -> Result<(), archetype_ecs::error::EcsError> {
        #[cfg(feature = "profiling")]
        let span = tracing::info_span!(
            "query_analysis",
            query_type = "position_velocity"
        );
        
        #[cfg(feature = "profiling")]
        let _guard = span.enter();
        
        let mut _count = 0;
        let mut _total_distance = 0.0;
        
        for (pos, vel) in world.query_mut::<(&Position, &Velocity)>().iter() {
            let distance = (pos.x * pos.x + pos.y * pos.y).sqrt();
            let speed = (vel.x * vel.x + vel.y * vel.y).sqrt();
            
            if distance > 50.0 && speed > 0.5 {
                _total_distance += distance;
                _count += 1;
            }
        }
        
        #[cfg(feature = "profiling")]
        tracing::info!(
            "Analyzed entities successfully"
        );
        
        Ok(())
    }
}

fn setup_profiling_subscriber() {
    #[cfg(feature = "profiling")]
    {
        use tracing_subscriber::Registry;
        use tracing_subscriber::prelude::*;
        use tracing_subscriber::fmt;
        
        let subscriber = Registry::default()
            .with(fmt::layer().with_target(false));
        
        tracing::subscriber::set_global_default(subscriber).unwrap();
    }
}

fn spawn_entities_batch(world: &mut World, count: usize) {
    #[cfg(feature = "profiling")]
    let span = tracing::info_span!(
        "spawn_batch",
        entity_count = count,
        archetype_count = world.archetype_count()
    );
    
    #[cfg(feature = "profiling")]
    let _guard = span.enter();
    
    let bundles: Vec<_> = (0..count)
        .map(|i| (
            Position { x: i as f32, y: 0.0 },
            Velocity { x: 0.1, y: 0.0 },
            Health { current: 50.0, max: 100.0 }
        ))
        .collect();
    
    match world.spawn_batch(bundles) {
        Ok(_entities) => {
            #[cfg(feature = "profiling")]
            tracing::info!("Spawned entities successfully");
        }
        Err(_e) => {
            #[cfg(feature = "profiling")]
            tracing::error!("Batch spawn failed");
        }
    }
}

fn main() {
    println!("=== Comprehensive Profiling Guide ===\n");
    
    // Setup profiling subscriber
    setup_profiling_subscriber();
    
    // Create world and executor
    let mut world = World::new();
    let mut schedule = Schedule::new();
    
    println!("1. Setting up profiling-enabled systems...");
    
    // Add systems with profiling
    schedule.add_system(Box::new(MovementSystem));
    schedule.add_system(Box::new(HealthSystem::new()));
    let query_system = QueryAnalysisSystem::new(&world);
    schedule.add_system(Box::new(query_system));
    
    let mut executor = Executor::new(&mut schedule);
    
    println!("2. Spawning entities with batch profiling...");
    spawn_entities_batch(&mut world, 1000);
    
    println!("3. Running profiling-enabled frame...");
    
    // Execute frame with profiling
    if let Err(_e) = executor.execute_frame(&mut world) {
        println!("Error occurred during execution");
    }
    
    println!("\n4. Profiling Results:");
    
    // Print profiling information
    executor.print_profile();
    
    println!("\n5. Enhanced Profiling Summary:");
    executor.print_profiling_summary(&world);
    
    println!("\n=== Profiling Best Practices ===");
    println!("✅ Enable profiling feature: --features profiling");
    println!("✅ Use RUST_LOG=debug for detailed tracing output");
    println!("✅ Profile in release mode for accurate performance metrics");
    println!("✅ Use spans to group related operations");
    println!("✅ Add custom metrics for domain-specific performance data");
    println!("✅ Export profiling data for analysis with CSV export");
    
    println!("\n=== Performance Tips ===");
    println!("🚀 Profile individual systems to identify bottlenecks");
    println!("🚀 Use query state caching for repeated queries");
    println!("🚀 Batch operations for better cache locality");
    println!("🚀 Consider component layout for optimal access patterns");
    println!("🚀 Use parallel execution for independent systems");
    
    println!("\n=== Production Profiling Setup ===");
    println!("```toml");
    println!("[dependencies]");
    println!("archetype-ecs = {{ version = \"1.1\", features = [\"profiling\"] }}");
    println!("tracing = \"0.1\"");
    println!("tracing-subscriber = {{ version = \"0.3\", features = [\"env-filter\"] }}");
    println!("```");
    
    println!("\n=== Example Complete ===");
}
