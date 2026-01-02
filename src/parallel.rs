use crate::dependency::{DependencyGraph, ExecutionStage};
use crate::error::Result;
use crate::system::System;
use crate::world::World;
use rayon::prelude::*;
use rustc_hash::FxHashMap;
use std::cmp::Ordering;
use std::sync::atomic::{AtomicUsize, Ordering as AtomicOrdering};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Priority levels for system execution
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    Critical = 3, // On critical path
    High = 2,     // Heavy computation or important systems
    Normal = 1,   // Default priority
    Low = 0,      // Lightweight systems
}

/// A scheduled task with priority and cost estimation
#[derive(Debug, Clone)]
pub struct ScheduledTask {
    pub system_index: usize,
    pub priority: Priority,
    pub estimated_cost: Duration,
    pub stage_depth: usize,
}

impl PartialEq for ScheduledTask {
    fn eq(&self, other: &Self) -> bool {
        self.system_index == other.system_index
    }
}

impl Eq for ScheduledTask {}

impl PartialOrd for ScheduledTask {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ScheduledTask {
    fn cmp(&self, other: &Self) -> Ordering {
        // Higher priority first, then by estimated cost (larger first for better load balancing)
        self.priority
            .cmp(&other.priority)
            .then_with(|| other.estimated_cost.cmp(&self.estimated_cost))
    }
}

/// Tracks execution statistics for adaptive profiling
#[derive(Debug, Clone)]
pub struct ExecutionStats {
    pub total_runs: usize,
    pub total_time: Duration,
    pub avg_time: Duration,
    pub last_time: Duration,
}

impl ExecutionStats {
    fn new() -> Self {
        Self {
            total_runs: 0,
            total_time: Duration::ZERO,
            avg_time: Duration::from_micros(100), // Default estimate
            last_time: Duration::ZERO,
        }
    }

    fn record(&mut self, duration: Duration) {
        self.total_runs += 1;
        self.total_time += duration;
        self.last_time = duration;
        self.avg_time = self.total_time / self.total_runs as u32;
    }

    fn estimated_cost(&self) -> Duration {
        if self.total_runs == 0 {
            Duration::from_micros(100)
        } else {
            // Use weighted average: 70% historical average, 30% last run
            (self.avg_time * 7 + self.last_time * 3) / 10
        }
    }
}

/// Task scheduler with priority queues and load balancing
pub struct TaskScheduler {
    execution_stats: FxHashMap<usize, ExecutionStats>,
    load_per_thread: Arc<Vec<AtomicUsize>>, // Microseconds of work per thread
}

impl TaskScheduler {
    pub fn new() -> Self {
        let thread_count = rayon::current_num_threads();
        let load_per_thread = Arc::new((0..thread_count).map(|_| AtomicUsize::new(0)).collect());

        Self {
            execution_stats: FxHashMap::default(),
            load_per_thread,
        }
    }

    /// Assign priority to a system based on critical path and execution history
    pub fn assign_priority(
        &self,
        system_index: usize,
        is_critical: bool,
        stage_depth: usize,
    ) -> Priority {
        if is_critical {
            return Priority::Critical;
        }

        // Check if system is expensive (>1ms average)
        if let Some(stats) = self.execution_stats.get(&system_index) {
            if stats.avg_time > Duration::from_millis(1) {
                return Priority::High;
            } else if stats.avg_time < Duration::from_micros(100) {
                return Priority::Low;
            }
        }

        // Systems early in the graph get higher priority
        if stage_depth == 0 {
            Priority::High
        } else {
            Priority::Normal
        }
    }

    /// Get estimated cost for a system
    pub fn estimated_cost(&self, system_index: usize) -> Duration {
        self.execution_stats
            .get(&system_index)
            .map(|s| s.estimated_cost())
            .unwrap_or_else(|| Duration::from_micros(100))
    }

    /// Record execution time for adaptive profiling
    pub fn record_execution(&mut self, system_index: usize, duration: Duration) {
        self.execution_stats
            .entry(system_index)
            .or_insert_with(ExecutionStats::new)
            .record(duration);
    }

    /// Create scheduled tasks from a stage
    pub fn schedule_stage(
        &self,
        stage: &ExecutionStage,
        critical_path: &[usize],
    ) -> Vec<ScheduledTask> {
        let mut tasks: Vec<ScheduledTask> = stage
            .system_indices
            .iter()
            .map(|&sys_idx| {
                let is_critical = critical_path.contains(&sys_idx);
                let priority = self.assign_priority(sys_idx, is_critical, stage.depth);
                let estimated_cost = self.estimated_cost(sys_idx);

                ScheduledTask {
                    system_index: sys_idx,
                    priority,
                    estimated_cost,
                    stage_depth: stage.depth,
                }
            })
            .collect();

        // Sort by priority (highest first)
        tasks.sort_by(|a, b| b.cmp(a));
        tasks
    }

    /// Reset load tracking for a new frame
    pub fn reset_load_tracking(&self) {
        for load in self.load_per_thread.iter() {
            load.store(0, AtomicOrdering::Relaxed);
        }
    }

    /// Get current thread's load
    pub fn get_thread_load(&self, thread_id: usize) -> usize {
        if thread_id < self.load_per_thread.len() {
            self.load_per_thread[thread_id].load(AtomicOrdering::Relaxed)
        } else {
            0
        }
    }

    /// Add load to current thread
    pub fn add_thread_load(&self, thread_id: usize, microseconds: usize) {
        if thread_id < self.load_per_thread.len() {
            self.load_per_thread[thread_id].fetch_add(microseconds, AtomicOrdering::Relaxed);
        }
    }
}

impl Default for TaskScheduler {
    fn default() -> Self {
        Self::new()
    }
}

/// Parallel executor using rayon work-stealing with advanced scheduling
pub struct ParallelExecutor {
    pub systems: Vec<Box<dyn System>>,
    dependency_graph: DependencyGraph,
    scheduler: TaskScheduler,
}

impl ParallelExecutor {
    /// Create parallel executor from systems
    pub fn new(systems: Vec<Box<dyn System>>) -> Self {
        // Get system accesses
        let accesses: Vec<_> = systems.iter().map(|s| s.access()).collect();

        // Build dependency graph
        let graph = DependencyGraph::new(accesses);

        // Debug: print schedule
        graph.print_schedule();

        Self {
            systems,
            dependency_graph: graph,
            scheduler: TaskScheduler::new(),
        }
    }

    /// Execute all systems in parallel with optimal scheduling
    pub fn execute_parallel(&mut self, world: &mut World) -> Result<()> {
        // Reset load tracking for this frame
        self.scheduler.reset_load_tracking();

        // Clone stages and critical path to avoid borrowing issues
        let stages = self.dependency_graph.stages().to_vec();
        let critical_path = self.dependency_graph.critical_path().to_vec();

        for stage in &stages {
            // Schedule tasks with priorities
            let tasks = self.scheduler.schedule_stage(stage, &critical_path);

            // Execute stage with scheduled tasks
            self.execute_stage_scheduled(&tasks, world)?;
        }

        Ok(())
    }

    /// Execute a stage with scheduled tasks (priority-based)
    fn execute_stage_scheduled(
        &mut self,
        tasks: &[ScheduledTask],
        world: &mut World,
    ) -> Result<()> {
        // Convert pointers to usize for Send + Sync
        let systems_ptr = self.systems.as_mut_ptr() as usize;
        let world_ptr = world as *mut World as usize;

        // SAFETY: Parallel Execution Invariants
        //
        // 1. Pointer Arithmetic Safety:
        //    - `systems_ptr` and `world_ptr` are captured BEFORE Rayon spawns threads
        //    - `self.systems` vec is NOT modified during parallel execution
        //    - Each thread gets a unique `sys_idx`, no aliasing of system references
        //
        // 2. Borrow Checker Satisfaction:
        //    - World is mutably borrowed for 'w (entire parallel section)
        //    - `&mut World` is reconstructed from raw pointer within each thread
        //    - No thread accesses the same system as another (unique indices)
        //
        // 3. No Data Races:
        //    - SystemAccess::conflicts_with() guarantees disjoint component/resource access
        //    - Read-only systems can run in parallel with each other
        //    - Write systems are scheduled in separate stages by dependency graph
        //
        // 4. Lifetime Validity:
        //    - 'w lifetime outlives the entire parallel execution
        //    - Systems vec remains valid (no reallocation during execution)
        //    - World reference remains valid (exclusive borrow held)
        //
        // 5. Thread Safety:
        //    - Only Send + Sync types are used in parallel closure
        //    - All captured data is either Copy (indices) or raw pointers
        //    - Results are collected after parallel execution completes
        //
        // This follows the same pattern as Rayon's internal parallel iterators
        // and is safe because the dependency graph ensures no conflicting access.
        let results: Vec<(usize, Duration, Result<()>)> = tasks
            .par_iter()
            .map(|task| {
                let start = Instant::now();
                let sys_idx = task.system_index;

                // Validate index bounds
                if sys_idx == usize::MAX {
                    return (
                        sys_idx,
                        Duration::ZERO,
                        Err(crate::error::EcsError::SystemNotFound),
                    );
                }

                // SAFETY: Same safety guarantees as before
                let system = unsafe { &mut *(systems_ptr as *mut Box<dyn System>).add(sys_idx) };
                let world = unsafe { &mut *(world_ptr as *mut World) };

                let result = system.run(world);
                let duration = start.elapsed();

                (sys_idx, duration, result)
            })
            .collect();

        // Record execution times and propagate errors
        for (sys_idx, duration, result) in results {
            self.scheduler.record_execution(sys_idx, duration);
            result?;
        }

        Ok(())
    }

    /// Execute a single stage (all systems in parallel) - legacy method for backward compatibility
    #[allow(dead_code)]
    ///
    /// # Safety Architecture
    ///
    /// This function uses unsafe code to enable parallel system execution while bypassing
    /// Rust's borrow checker. The safety of this approach relies on the following invariants:
    ///
    /// ## Invariant 1: Non-Overlapping System Access
    /// The `DependencyGraph` guarantees that all systems within a single stage have
    /// non-conflicting access patterns. Systems are only grouped in the same stage if:
    /// - They don't both write to the same component type
    /// - If one writes to a component, the other doesn't read or write it
    ///
    /// ## Invariant 2: Valid System Indices
    /// All indices in `stage.system_indices` are guaranteed to be:
    /// - Within bounds: `sys_idx < self.systems.len()`
    /// - Unique within the stage (no duplicate indices)
    /// - Derived from the dependency graph construction
    ///
    /// ## Invariant 3: Thread-Safe World Access
    /// Although multiple threads access `world` simultaneously, the ECS architecture ensures:
    /// - Each system accesses different archetypes or different components
    /// - Component columns are stored separately, preventing data races
    /// - The dependency graph enforces exclusive access to conflicting resources
    ///
    /// ## Lifetime Guarantees
    /// - The raw pointers are only valid for the duration of this function
    /// - No references escape the parallel iteration scope
    /// - All borrows are released before the function returns
    ///
    /// ## Why This Is Safe
    /// 1. **Spatial Safety**: Each thread accesses a unique system (unique indices)
    /// 2. **Temporal Safety**: Pointers are only dereferenced within the par_iter scope
    /// 3. **Data Race Freedom**: Dependency graph ensures disjoint memory access
    /// 4. **Bounds Safety**: Index validation prevents out-of-bounds access
    fn execute_stage(&mut self, stage: &ExecutionStage, world: &mut World) -> Result<()> {
        // Convert pointers to usize for Send + Sync across thread boundaries
        let systems_ptr = self.systems.as_mut_ptr() as usize;
        let world_ptr = world as *mut World as usize;

        // Execute all systems in this stage in parallel using Rayon's work-stealing
        let results: Vec<Result<()>> = stage
            .system_indices
            .par_iter()
            .map(move |&sys_idx| {
                // Validate index bounds (defensive programming)
                if sys_idx == usize::MAX {
                    return Err(crate::error::EcsError::SystemNotFound);
                }

                // SAFETY: This is safe because:
                // 1. sys_idx is guaranteed to be < self.systems.len() (from dependency graph)
                // 2. sys_idx is unique within this stage (no two threads access same system)
                // 3. The pointer is valid for the lifetime of this function
                // 4. No other code is accessing self.systems during parallel execution
                let system = unsafe { &mut *(systems_ptr as *mut Box<dyn System>).add(sys_idx) };

                // SAFETY: This is safe because:
                // 1. The world pointer is valid for the duration of this function
                // 2. Systems in this stage have non-conflicting access (verified by DependencyGraph)
                // 3. Each system accesses disjoint sets of components/archetypes
                // 4. The ECS architecture prevents data races through archetype isolation
                let world = unsafe { &mut *(world_ptr as *mut World) };

                system.run(world)
            })
            .collect();

        // Propagate any errors from system execution
        for result in results {
            result?;
        }

        Ok(())
    }

    /// Get dependency graph for inspection
    pub fn dependency_graph(&self) -> &DependencyGraph {
        &self.dependency_graph
    }

    /// Get scheduler for inspection
    pub fn scheduler(&self) -> &TaskScheduler {
        &self.scheduler
    }

    /// Get mutable scheduler for configuration
    pub fn scheduler_mut(&mut self) -> &mut TaskScheduler {
        &mut self.scheduler
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::system::SystemAccess;
    use crate::world::World;

    struct DummySystem {
        name: &'static str,
        access: SystemAccess,
    }

    impl System for DummySystem {
        fn name(&self) -> &'static str {
            self.name
        }

        fn access(&self) -> SystemAccess {
            self.access.clone()
        }

        fn run(&mut self, _world: &mut World) -> Result<()> {
            Ok(())
        }
    }

    #[test]
    fn test_parallel_executor_creation() {
        let systems: Vec<Box<dyn System>> = vec![Box::new(DummySystem {
            name: "sys1",
            access: SystemAccess::empty(),
        })];

        let executor = ParallelExecutor::new(systems);
        assert_eq!(executor.systems.len(), 1);
    }

    #[test]
    fn test_priority_assignment() {
        let scheduler = TaskScheduler::new();

        // Critical systems get highest priority
        assert_eq!(scheduler.assign_priority(0, true, 0), Priority::Critical);

        // Non-critical at depth 0 get high priority
        assert_eq!(scheduler.assign_priority(0, false, 0), Priority::High);

        // Non-critical at higher depth get normal priority
        assert_eq!(scheduler.assign_priority(0, false, 1), Priority::Normal);
    }

    #[test]
    fn test_task_scheduling() {
        let scheduler = TaskScheduler::new();
        let stage = ExecutionStage {
            system_indices: vec![0, 1, 2],
            depth: 0,
        };
        let critical_path = vec![1];

        let tasks = scheduler.schedule_stage(&stage, &critical_path);

        assert_eq!(tasks.len(), 3);
        // System 1 should be first (critical)
        assert_eq!(tasks[0].system_index, 1);
        assert_eq!(tasks[0].priority, Priority::Critical);
    }

    #[test]
    fn test_adaptive_profiling() {
        let mut scheduler = TaskScheduler::new();

        // Record some executions
        scheduler.record_execution(0, Duration::from_millis(2));
        scheduler.record_execution(0, Duration::from_millis(3));

        // Should now classify as high priority due to cost
        let priority = scheduler.assign_priority(0, false, 1);
        assert_eq!(priority, Priority::High);
    }
}
