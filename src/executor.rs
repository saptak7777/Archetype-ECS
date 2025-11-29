//! Phase 4 Executor, Sync, and Debugging
//! Combined to fit size constraints

// ============================================================================
// executor.rs
// ============================================================================

use crate::error::Result;
use crate::schedule::Schedule;
use crate::system::SystemId;
use crate::World;
use std::collections::HashMap;

/// System execution profiler
#[derive(Debug, Clone)]
pub struct SystemStats {
    pub min: std::time::Duration,
    pub max: std::time::Duration,
    pub avg: std::time::Duration,
    pub call_count: u64,
}

/// System profiler for collecting timing data
pub struct SystemProfiler {
    timings: HashMap<SystemId, Vec<std::time::Duration>>,
    call_counts: HashMap<SystemId, u64>,
}

impl SystemProfiler {
    pub fn new() -> Self {
        Self {
            timings: HashMap::new(),
            call_counts: HashMap::new(),
        }
    }

    pub fn record_execution(&mut self, id: SystemId, duration: std::time::Duration) {
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

        let min = *timings.iter().min().unwrap_or(&std::time::Duration::ZERO);
        let max = *timings.iter().max().unwrap_or(&std::time::Duration::ZERO);
        let avg = timings.iter().sum::<std::time::Duration>() / timings.len() as u32;

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

/// Frame executor
pub struct Executor {
    pub schedule: Schedule,
    pub profiler: SystemProfiler,
}

impl Executor {
    /// Create new executor
    pub fn new(schedule: Schedule) -> Self {
        Self {
            schedule,
            profiler: SystemProfiler::new(),
        }
    }

    /// Execute one frame
    pub fn execute_frame(&mut self, world: &mut World) -> Result<()> {
        // FIXED: Clone stages to avoid borrow conflict
        let stage_count = self.schedule.stages.len();

        for stage_idx in 0..stage_count {
            // Execute systems in stage
            for &_system_id in &self.schedule.stages[stage_idx].systems {
                // In Phase 4, we store system references
                // Full implementation in actual executor
                // For now: placeholder
            }

            // Barrier: sync point between stages
            self.barrier(world)?;
        }

        Ok(())
    }

    fn barrier(&mut self, _world: &mut World) -> Result<()> {
        // Flush command buffers
        // Compact archetypes (optional)
        Ok(())
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
        let total_systems = schedule.graph.nodes.len();
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
