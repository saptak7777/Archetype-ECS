//! Phase 4 Executor, Sync, and Debugging
//! Combined to fit size constraints

// ============================================================================
// executor.rs
// ============================================================================

use crate::error::{EcsError, Result};
use crate::schedule::Schedule;
use crate::system::{System, SystemId};
use crate::World;
use rustc_hash::FxHashMap;
use std::time::{Duration, Instant};

/// System execution profiler
#[derive(Debug, Clone)]
pub struct SystemStats {
    pub min: Duration,
    pub max: Duration,
    pub avg: Duration,
    pub call_count: u64,
}

/// System profiler for collecting timing data
pub struct SystemProfiler {
    timings: FxHashMap<SystemId, Vec<Duration>>,
    call_counts: FxHashMap<SystemId, u64>,
}

impl SystemProfiler {
    pub fn new() -> Self {
        Self {
            timings: FxHashMap::default(),
            call_counts: FxHashMap::default(),
        }
    }

    pub fn record_execution(&mut self, id: SystemId, duration: Duration) {
        self.timings.entry(id).or_default().push(duration);
        self.call_counts
            .entry(id)
            .and_modify(|c| *c += 1)
            .or_insert(1);
    }

    pub fn get_stats(&self, id: SystemId) -> Option<SystemStats> {
        let timings = self.timings.get(&id)?;
        if timings.is_empty() {
            return None;
        }

        let min = *timings.iter().min().unwrap_or(&Duration::ZERO);
        let max = *timings.iter().max().unwrap_or(&Duration::ZERO);
        let avg = timings.iter().sum::<Duration>() / timings.len() as u32;

        Some(SystemStats {
            min,
            max,
            avg,
            call_count: *self.call_counts.get(&id).unwrap_or(&0),
        })
    }

    pub fn clear(&mut self) {
        self.timings.clear();
        self.call_counts.clear();
    }
}

impl Default for SystemProfiler {
    fn default() -> Self {
        Self::new()
    }
}

/// Per-system timing data for a single frame
#[derive(Debug, Clone)]
pub struct SystemTiming {
    pub name: String,
    pub duration: Duration,
}

/// Execution profile for a frame
#[derive(Debug, Clone)]
pub struct ExecutionProfile {
    pub total_frame_time: Duration,
    pub system_timings: Vec<SystemTiming>,
}

/// Frame executor
pub struct Executor {
    pub schedule: Schedule,
    pub profiler: SystemProfiler,
    last_profile: Option<ExecutionProfile>,
}

impl Executor {
    /// Create new executor
    pub fn new(schedule: Schedule) -> Self {
        Self {
            schedule,
            profiler: SystemProfiler::new(),
            last_profile: None,
        }
    }

    /// Execute one frame
    pub fn execute_frame(&mut self, world: &mut World) -> Result<()> {
        self.schedule.ensure_built()?;
        // Collect stage plan to avoid borrow checker issues
        let stage_plan: Vec<Vec<SystemId>> = self
            .schedule
            .stage_plan()
            .iter()
            .map(|stage| stage.to_vec())
            .collect();
        let frame_start = Instant::now();
        let mut system_timings = Vec::with_capacity(self.schedule.systems.len());

        for stage in stage_plan {
            for system_id in stage {
                let system = self
                    .schedule
                    .system_mut_by_id(system_id)
                    .ok_or(EcsError::SystemNotFound)?;
                let system_name = system.name();

                let start = Instant::now();
                system.run(world)?;
                let duration = start.elapsed();

                self.profiler.record_execution(system_id, duration);
                system_timings.push(SystemTiming {
                    name: system_name.to_string(),
                    duration,
                });
            }

            self.barrier(world)?;
        }

        let total_frame_time = frame_start.elapsed();
        self.last_profile = Some(ExecutionProfile {
            total_frame_time,
            system_timings,
        });

        Ok(())
    }

    /// Execute systems in parallel where possible
    ///
    /// Uses the dependency graph to determine which systems can run concurrently.
    /// See `ParallelExecutor::execute_stage` for detailed safety documentation.
    pub fn execute_frame_parallel(&mut self, world: &mut World) -> Result<()> {
        use crate::dependency::DependencyGraph;
        use crate::system::System;
        use rayon::prelude::*;
        // Get system accesses
        let accesses = self.schedule.get_accesses();
        let graph = DependencyGraph::new(accesses);

        // Clone stages to avoid borrowing issues
        let stages = graph.stages().to_vec();

        for stage in stages {
            // Parallel execution logic inline (similar to ParallelExecutor)
            let systems_ptr = self.schedule.systems.as_mut_ptr() as usize;
            let systems_len = self.schedule.systems.len();
            let world_ptr = world as *mut World as usize;

            let results: Vec<Result<()>> = stage
                .system_indices
                .par_iter()
                .map(move |&sys_idx| {
                    if sys_idx >= systems_len {
                        return Err(EcsError::SystemNotFound);
                    }

                    // SAFETY: See ParallelExecutor::execute_stage for full safety documentation.
                    // In summary:
                    // 1. sys_idx is guaranteed valid by dependency graph
                    // 2. Systems in same stage have non-conflicting access
                    // 3. Each thread accesses a unique system index
                    // 4. World access is disjoint (different components/archetypes)
                    let system =
                        unsafe { &mut *(systems_ptr as *mut Box<dyn System>).add(sys_idx) };
                    let world = unsafe { &mut *(world_ptr as *mut World) };
                    system.run(world)
                })
                .collect();

            for result in results {
                result?;
            }

            self.barrier(world)?;
        }

        Ok(())
    }

    /// Execute with automatic parallel detection
    pub fn execute_frame_auto(&mut self, world: &mut World) -> Result<()> {
        // Use parallel if multiple systems, sequential if one
        if self.schedule.systems.len() > 1 {
            self.execute_frame_parallel(world)
        } else {
            self.execute_frame(world)
        }
    }

    /// Execute systems and process observer events
    pub fn execute_frame_with_events(&mut self, world: &mut World) -> Result<()> {
        // Execute systems
        for system in &mut self.schedule.systems {
            system.run(world)?;
        }

        // Process queued events
        world.process_events()?;

        Ok(())
    }

    /// Execute with events and profiling
    pub fn execute_frame_full(&mut self, world: &mut World) -> Result<()> {
        #[cfg(feature = "profiling")]
        let _span = info_span!("execute_frame_full");

        // Execute systems
        for system in &mut self.schedule.systems {
            system.run(world)?;
        }

        // Process all events
        world.process_events()?;

        Ok(())
    }

    /// Execute with hierarchy system
    pub fn execute_with_hierarchy(&mut self, world: &mut World) -> Result<()> {
        use crate::hierarchy_system::HierarchyUpdateSystem;

        // Run hierarchy update first (transforms)
        let mut hierarchy_system = HierarchyUpdateSystem::new();
        hierarchy_system.run(world)?;

        // Then run user systems
        for system in &mut self.schedule.systems {
            system.run(world)?;
        }

        // Process events if Phase 3 is enabled
        world.process_events()?;

        Ok(())
    }

    /// Execute with everything (hierarchy + systems + events)
    pub fn execute_full(&mut self, world: &mut World) -> Result<()> {
        use crate::hierarchy_system::HierarchyUpdateSystem;

        // Execute hierarchy system
        let mut hierarchy_system = HierarchyUpdateSystem::new();
        hierarchy_system.run(world)?;

        // Execute user systems
        for system in &mut self.schedule.systems {
            system.run(world)?;
        }

        // Process events
        world.process_events()?;

        Ok(())
    }

    /// Execute with global event processing (Phase 6)
    pub fn execute_with_global_events(&mut self, world: &mut World) -> Result<()> {
        // Execute systems
        for system in &mut self.schedule.systems {
            system.run(world)?;
        }

        // Process global events published by systems
        world.process_global_events()?;

        Ok(())
    }

    /// Execute complete frame (hierarchy + systems + global events + entity events)
    pub fn execute_complete_frame(&mut self, world: &mut World) -> Result<()> {
        use crate::hierarchy_system::HierarchyUpdateSystem;

        // 1. Update hierarchy transforms
        let mut hierarchy_system = HierarchyUpdateSystem::new();
        hierarchy_system.run(world)?;

        // 2. Execute systems
        for system in &mut self.schedule.systems {
            system.run(world)?;
        }

        // 3. Process global events (Phase 6)
        world.process_global_events()?;

        // 4. Process entity lifecycle events (Phase 3)
        world.process_events()?;

        Ok(())
    }

    fn barrier(&mut self, _world: &mut World) -> Result<()> {
        // Flush command buffers
        // Compact archetypes (optional)
        Ok(())
    }

    /// Get the most recent execution profile
    pub fn profile(&self) -> Option<&ExecutionProfile> {
        self.last_profile.as_ref()
    }

    /// Print profiling information for the last frame
    pub fn print_profile(&self) {
        if let Some(profile) = &self.last_profile {
            println!(
                "Frame time: {:.3?} ({} systems)",
                profile.total_frame_time,
                profile.system_timings.len()
            );
            for (index, timing) in profile.system_timings.iter().enumerate() {
                println!("  {:02}: {:<24} {:?}", index, timing.name, timing.duration);
            }
        } else {
            println!("No profiling data collected yet.");
        }
    }
}

// ============================================================================
// world_sync.rs
// ============================================================================

use crate::command::CommandBuffer;
use crate::entity::EntityId;

/// Synchronization point between stages
pub struct SyncPoint {
    pub command_buffers: Vec<CommandBuffer>,
    pub despawn_queue: Vec<EntityId>,
}

impl SyncPoint {
    /// Create new sync point
    pub fn new() -> Self {
        Self {
            command_buffers: Vec::new(),
            despawn_queue: Vec::new(),
        }
    }

    /// Add command buffer to flush
    pub fn add_command_buffer(&mut self, buffer: CommandBuffer) {
        self.command_buffers.push(buffer);
    }

    /// Queue entity for despawn
    pub fn queue_despawn(&mut self, entity: EntityId) {
        self.despawn_queue.push(entity);
    }

    /// Flush all commands to world
    pub fn flush(&mut self, world: &mut World) -> Result<()> {
        // Despawn entities (LIFO to maintain indices)
        for &entity in self.despawn_queue.iter().rev() {
            world.despawn(entity).ok();
        }
        self.despawn_queue.clear();

        // Flush command buffers
        for buffer in self.command_buffers.drain(..) {
            world.flush_commands(buffer)?;
        }

        Ok(())
    }
}

impl Default for SyncPoint {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// scheduler_debug.rs
// ============================================================================

use std::fs::File;
use std::io::Write;

/// Debug information about scheduling
#[derive(Debug, Clone)]
pub struct ScheduleDebugInfo {
    pub stage_count: usize,
    pub total_systems: usize,
    pub systems_per_stage: Vec<usize>,
}

impl ScheduleDebugInfo {
    /// Create from schedule
    pub fn from_schedule(schedule: &Schedule) -> Self {
        let stage_count = schedule.stage_count();
        let total_systems = schedule.system_count();
        let systems_per_stage = (0..stage_count)
            .map(|i| schedule.stage_system_count(i))
            .collect();

        Self {
            stage_count,
            total_systems,
            systems_per_stage,
        }
    }

    /// Print debug info
    pub fn print_debug(&self) {
        println!("Schedule Debug Info:");
        println!("  Total systems: {}", self.total_systems);
        println!("  Stages: {}", self.stage_count);
        for (i, &count) in self.systems_per_stage.iter().enumerate() {
            println!("    Stage {i}: {count} systems");
        }
    }

    /// Export as JSON (simplified)
    pub fn export_json(&self, filename: &str) -> std::io::Result<()> {
        let mut file = File::create(filename)?;
        write!(file, "{{")?;
        write!(file, "\"stage_count\":{},", self.stage_count)?;
        write!(file, "\"total_systems\":{},", self.total_systems)?;
        write!(file, "\"systems_per_stage\":[")?;
        for (i, &count) in self.systems_per_stage.iter().enumerate() {
            if i > 0 {
                write!(file, ",")?;
            }
            write!(file, "{count}")?;
        }
        write!(file, "]")?;
        write!(file, "}}")?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_point_creation() {
        let sp = SyncPoint::new();
        assert!(sp.command_buffers.is_empty());
        assert!(sp.despawn_queue.is_empty());
    }

    #[test]
    fn test_profiler_creation() {
        let profiler = SystemProfiler::new();
        assert!(profiler.timings.is_empty());
    }
}
