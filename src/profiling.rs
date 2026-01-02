//! # Profiling Guide
//!
//! The Archetype ECS includes built-in profiling support to measure system performance.
//!
//! ## Basic Usage
//!
//! Enable the `profiling` feature in your Cargo.toml:
//!
//! ```toml
//! [dependencies]
//! archetype-ecs = { version = "1.1", features = ["profiling"] }
//! ```
//!
//! Then use profiling in your systems:
//!
//! ```ignore
//! use tracing::{info_span, instrument};
//!
//! pub fn movement_system(mut world: World) {
//!     #[cfg(feature = "profiling")]
//!     let span = info_span!("movement_system", entity_count = ?world.entity_count());
//!     
//!     #[cfg(feature = "profiling")]
//!     let _guard = span.enter();
//!     
//!     // System logic here
//! }
//! ```
//!
//! ## Collecting Metrics
//!
//! ```ignore
//! use tracing_subscriber::Registry;
//! use tracing_subscriber::prelude::*;
//!
//! let subscriber = Registry::default()
//!     .with(tracing_subscriber::fmt::layer())
//!     .with(tracing_subscriber::EnvFilter::from_default_env());
//!
//! tracing::subscriber::set_global_default(subscriber).unwrap();
//!
//! // Now all profiling data is collected and printed
//! ```
//!
//! ## Performance Tips
//!
//! 1. Profile in release mode for accurate metrics
//! 2. Use `RUST_LOG=debug` to see all spans

pub mod profiling_examples {
    /// Example implementations of profiling patterns
    /// Basic system profiling
    /// ```ignore
    /// use tracing::{info_span, Level};
    /// use archetype_ecs::World;
    /// 
    /// let mut world = World::new();
    /// 
    /// // Profile a system execution
    /// let _span = info_span!("movement_system").entered();
    /// 
    /// // System logic here
    /// for (mut pos, vel) in world.query_mut::<(&mut Position, &Velocity)>().iter() {
    ///     pos.x += vel.x;
    ///     pos.y += vel.y;
    ///     fn run(&mut self, world: &mut World) -> Result<(), archetype_ecs::error::EcsError> {
    ///         #[cfg(feature = "profiling")]
    ///         let span = info_span!(
    ///             "movement_system",
    ///             entity_count = world.entity_count()
    ///         );
    ///         
    ///         #[cfg(feature = "profiling")]
    ///         let _guard = span.enter();
    ///         
    ///         // System logic here
    ///         for (pos, vel) in world.query_mut::<(&mut Position, &Velocity)>().iter() {
    ///             pos.x += vel.x;
    ///             pos.y += vel.y;
    ///         }
    ///         
    ///         Ok(())
    ///     }
    /// }
    /// ```
    pub fn basic_system_profiling() {
    }
    
    /// Advanced profiling with custom metrics
    /// ```ignore
    /// use tracing::{info_span, Level};
    /// use std::time::Instant;
    ///
    /// struct ComplexSystem {
    ///     processed_count: std::cell::Cell<usize>,
    /// }
    ///
    /// impl System for ComplexSystem {
    ///     fn run(&mut self, world: &mut World) -> Result<(), archetype_ecs::error::EcsError> {
    ///         let start = Instant::now();
    ///         
    ///         #[cfg(feature = "profiling")]
    ///         let span = info_span!(
    ///             "complex_system",
    ///             entity_count = world.entity_count(),
    ///             component_count = world.component_count()
    ///         );
    ///         
    ///         #[cfg(feature = "profiling")]
    ///         let _guard = span.enter();
    ///         
    ///         // Record custom metrics
    ///         let mut processed = 0;
    ///         for (entity, (pos, vel, health)) in world.query_mut::<(archetype_ecs::Entity, &mut Position, &mut Velocity, &mut Health)>().iter() {
    ///             // Complex processing
    ///             pos.x += vel.x * 0.016;
    ///             pos.y += vel.y * 0.016;
    ///             
    ///             if health.current < health.max {
    ///                 health.current = (health.current + 1.0).min(health.max);
    ///             }
    ///             
    ///             processed += 1;
    ///         }
    ///         
    ///         self.processed_count.set(processed);
    ///         
    ///         #[cfg(feature = "profiling")]
    ///         tracing::info!(
    ///             target: "performance_metrics",
    ///             "Processed {} entities in {:?}",
    ///             processed,
    ///             start.elapsed()
    ///         );
    ///         
    ///         Ok(())
    ///     }
    /// }
    /// ```
    pub fn advanced_profiling() {
    }
    
    /// Batch operation profiling
    /// ```ignore
    /// use archetype_ecs::World;
    /// use tracing::info_span;
    ///
    /// fn spawn_entities_batch(world: &mut World, count: usize) {
    ///     #[cfg(feature = "profiling")]
    ///     let span = info_span!(
    ///         "spawn_batch",
    ///         entity_count = count,
    ///         archetype_count = world.archetype_count()
    ///     );
    ///     
    ///     #[cfg(feature = "profiling")]
    ///     let _guard = span.enter();
    ///     
    ///     let bundles: Vec<_> = (0..count)
    ///         .map(|i| (Position { x: i as f32, y: 0.0 }, Velocity { x: 0.1, y: 0.0 }))
    ///         .collect();
    ///     
    ///     match world.spawn_batch(bundles) {
    ///         Ok(entities) => {
    ///             #[cfg(feature = "profiling")]
    ///             tracing::info!("Spawned {} entities successfully", entities.len());
    ///         }
    ///         Err(e) => {
    ///             #[cfg(feature = "profiling")]
    ///             tracing::error!("Batch spawn failed: {}", e);
    ///         }
    ///     }
    /// }
    /// ```
    pub fn batch_operation_profiling() {
    }
    
    /// Query performance profiling
    /// ```ignore
    /// use archetype_ecs::{World, QueryState};
    /// use tracing::info_span;
    ///
    /// struct QueryBenchmarkSystem {
    ///     query_state: QueryState<(&Position, &Velocity)>,
    /// }
    ///
    /// impl QueryBenchmarkSystem {
    ///     fn new(world: &World) -> Self {
    ///         Self {
    ///             query_state: QueryState::new(world),
    ///         }
    ///     }
    /// }
    ///
    /// impl System for QueryBenchmarkSystem {
    ///     fn run(&mut self, world: &mut World) -> Result<(), archetype_ecs::error::EcsError> {
    ///         #[cfg(feature = "profiling")]
    ///         let span = info_span!(
    ///             "query_benchmark",
    ///             matched_archetypes = self.query_state.matches.len()
    ///         );
    ///         
    ///         #[cfg(feature = "profiling")]
    ///         let _guard = span.enter();
    ///         
    ///         let mut count = 0;
    ///         for (pos, vel) in self.query_state.query(world).iter() {
    ///             // Query processing
    ///             let distance = (pos.x * pos.x + pos.y * pos.y).sqrt();
    ///             if distance > 100.0 {
    ///                 // Some processing
    ///             }
    ///             count += 1;
    ///         }
    ///         
    ///         #[cfg(feature = "profiling")]
    ///         tracing::info!("Queried {} entities", count);
    ///         
    ///         Ok(())
    ///     }
    /// }
    /// ```
    pub fn query_performance_profiling() {
    }
}
