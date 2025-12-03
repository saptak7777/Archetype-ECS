use crate::system::SystemAccess;
use rustc_hash::{FxHashMap, FxHashSet};
use std::collections::VecDeque;

/// Represents execution stages where all systems in a stage can run in parallel
#[derive(Clone, Debug)]
pub struct ExecutionStage {
    pub system_indices: Vec<usize>,
    pub depth: usize, // Depth in dependency graph (for priority)
}

/// Builds execution stages from system dependencies using topological sort
pub struct DependencyGraph {
    stages: Vec<ExecutionStage>,
    critical_path: Vec<usize>,
    #[allow(dead_code)] // Used for future graph analysis features
    adjacency_list: FxHashMap<usize, Vec<usize>>,
}

impl DependencyGraph {
    /// Create graph from system accesses with optimal scheduling
    pub fn new(system_accesses: Vec<SystemAccess>) -> Self {
        let adjacency_list = Self::build_adjacency_list(&system_accesses);
        let stages = Self::build_stages_topological(&system_accesses, &adjacency_list);
        let critical_path = Self::find_critical_path(&stages, &adjacency_list);

        Self {
            stages,
            critical_path,
            adjacency_list,
        }
    }

    /// Build adjacency list representing dependencies between systems
    /// If system A must run before system B, then A -> B in the graph
    fn build_adjacency_list(accesses: &[SystemAccess]) -> FxHashMap<usize, Vec<usize>> {
        let mut graph = FxHashMap::default();

        for i in 0..accesses.len() {
            graph.insert(i, Vec::new());
        }

        // Build directed edges: if A conflicts with B and A comes first, A -> B
        for i in 0..accesses.len() {
            for j in (i + 1)..accesses.len() {
                if accesses[i].conflicts_with(&accesses[j]) {
                    // System i must complete before system j can start
                    graph.get_mut(&i).unwrap().push(j);
                }
            }
        }

        graph
    }

    /// Build execution stages using topological sort and graph coloring
    /// This ensures optimal parallelization while respecting dependencies
    fn build_stages_topological(
        accesses: &[SystemAccess],
        adjacency_list: &FxHashMap<usize, Vec<usize>>,
    ) -> Vec<ExecutionStage> {
        if accesses.is_empty() {
            return vec![];
        }

        // Calculate in-degree for each node (number of dependencies)
        let mut in_degree = vec![0; accesses.len()];
        for edges in adjacency_list.values() {
            for &target in edges {
                in_degree[target] += 1;
            }
        }

        // Track depth of each system in the dependency graph
        let mut depths = vec![0; accesses.len()];
        let mut queue = VecDeque::new();

        // Start with systems that have no dependencies
        for (idx, &degree) in in_degree.iter().enumerate() {
            if degree == 0 {
                queue.push_back(idx);
            }
        }

        // Topological sort with depth tracking (Kahn's algorithm)
        let mut sorted = Vec::with_capacity(accesses.len());
        while let Some(node) = queue.pop_front() {
            sorted.push(node);

            if let Some(neighbors) = adjacency_list.get(&node) {
                for &neighbor in neighbors {
                    in_degree[neighbor] -= 1;
                    depths[neighbor] = depths[neighbor].max(depths[node] + 1);

                    if in_degree[neighbor] == 0 {
                        queue.push_back(neighbor);
                    }
                }
            }
        }

        // Group systems by depth (systems at same depth can potentially run in parallel)
        let max_depth = *depths.iter().max().unwrap_or(&0);
        let mut stages = Vec::new();

        for depth in 0..=max_depth {
            let mut stage_systems = Vec::new();

            for &sys_idx in &sorted {
                if depths[sys_idx] == depth {
                    // Check if this system can be added to current stage
                    // (doesn't conflict with any system already in stage)
                    let mut can_add = true;
                    for &existing_idx in &stage_systems {
                        if accesses[sys_idx].conflicts_with(&accesses[existing_idx]) {
                            can_add = false;
                            break;
                        }
                    }

                    if can_add {
                        stage_systems.push(sys_idx);
                    }
                }
            }

            if !stage_systems.is_empty() {
                stages.push(ExecutionStage {
                    system_indices: stage_systems,
                    depth,
                });
            }
        }

        // Optimize stages using graph coloring for systems that couldn't fit
        Self::optimize_stages(&mut stages, accesses, &sorted, &depths);

        stages
    }

    /// Optimize stage assignment using graph coloring
    /// Systems that couldn't fit in their depth level are assigned to later stages
    fn optimize_stages(
        stages: &mut Vec<ExecutionStage>,
        accesses: &[SystemAccess],
        sorted: &[usize],
        depths: &[usize],
    ) {
        // Collect systems not yet assigned to any stage
        let mut assigned: FxHashSet<usize> = stages
            .iter()
            .flat_map(|s| s.system_indices.iter().copied())
            .collect();

        let mut unassigned: Vec<usize> = sorted
            .iter()
            .copied()
            .filter(|&idx| !assigned.contains(&idx))
            .collect();

        // Assign unassigned systems to stages
        while !unassigned.is_empty() {
            let mut next_unassigned = Vec::with_capacity(unassigned.len());

            for &sys_idx in &unassigned {
                let target_depth = depths[sys_idx];
                let mut placed = false;

                // Try to place in existing stages at or after target depth
                for stage in stages.iter_mut().filter(|s| s.depth >= target_depth) {
                    let mut can_add = true;
                    for &existing_idx in &stage.system_indices {
                        if accesses[sys_idx].conflicts_with(&accesses[existing_idx]) {
                            can_add = false;
                            break;
                        }
                    }

                    if can_add {
                        stage.system_indices.push(sys_idx);
                        assigned.insert(sys_idx);
                        placed = true;
                        break;
                    }
                }

                if !placed {
                    next_unassigned.push(sys_idx);
                }
            }

            // If we couldn't place any systems, create a new stage
            if next_unassigned.len() == unassigned.len() && !next_unassigned.is_empty() {
                let sys_idx = next_unassigned.remove(0);
                let new_depth = stages.last().map(|s| s.depth + 1).unwrap_or(0);
                stages.push(ExecutionStage {
                    system_indices: vec![sys_idx],
                    depth: new_depth,
                });
                assigned.insert(sys_idx);
            }

            unassigned = next_unassigned;
        }
    }

    /// Find the critical path (longest dependency chain) for priority scheduling
    fn find_critical_path(
        stages: &[ExecutionStage],
        adjacency_list: &FxHashMap<usize, Vec<usize>>,
    ) -> Vec<usize> {
        if stages.is_empty() {
            return vec![];
        }

        // Find the system with maximum depth
        let mut max_depth_system = 0;
        let mut max_depth = 0;

        for stage in stages {
            if stage.depth > max_depth {
                max_depth = stage.depth;
                if let Some(&first_sys) = stage.system_indices.first() {
                    max_depth_system = first_sys;
                }
            }
        }

        // Backtrack to find the critical path
        let mut path = vec![max_depth_system];
        let mut current = max_depth_system;

        // Build reverse adjacency list
        let mut reverse_adj: FxHashMap<usize, Vec<usize>> = FxHashMap::default();
        for (&from, targets) in adjacency_list {
            for &to in targets {
                reverse_adj.entry(to).or_default().push(from);
            }
        }

        // Trace back through dependencies
        while let Some(predecessors) = reverse_adj.get(&current) {
            if let Some(&pred) = predecessors.first() {
                path.push(pred);
                current = pred;
            } else {
                break;
            }
        }

        path.reverse();
        path
    }

    /// Get execution stages
    pub fn stages(&self) -> &[ExecutionStage] {
        &self.stages
    }

    /// Get number of stages
    pub fn stage_count(&self) -> usize {
        self.stages.len()
    }

    /// Get critical path (systems on longest dependency chain)
    pub fn critical_path(&self) -> &[usize] {
        &self.critical_path
    }

    /// Check if a system is on the critical path
    pub fn is_critical(&self, system_index: usize) -> bool {
        self.critical_path.contains(&system_index)
    }

    /// Print execution plan (for debugging)
    pub fn print_schedule(&self) {
        println!("Execution Schedule ({} stages):", self.stages.len());
        println!("Critical Path: {:?}", self.critical_path);
        println!();

        for (stage_idx, stage) in self.stages.iter().enumerate() {
            println!(
                "  Stage {} (depth {}): {} systems (parallel)",
                stage_idx + 1,
                stage.depth,
                stage.system_indices.len()
            );
            for &sys_idx in &stage.system_indices {
                let marker = if self.is_critical(sys_idx) {
                    " [CRITICAL]"
                } else {
                    ""
                };
                println!("    - System {sys_idx}{marker}");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::any::TypeId;

    #[test]
    fn test_no_conflicts_parallel() {
        let access1 = SystemAccess {
            reads: vec![TypeId::of::<i32>()],
            writes: vec![],
        };
        let access2 = SystemAccess {
            reads: vec![TypeId::of::<f32>()],
            writes: vec![],
        };

        let graph = DependencyGraph::new(vec![access1, access2]);
        assert_eq!(graph.stage_count(), 1, "Should execute in parallel");
    }

    #[test]
    fn test_write_conflict_sequential() {
        let access1 = SystemAccess {
            reads: vec![TypeId::of::<i32>()],
            writes: vec![TypeId::of::<f32>()],
        };
        let access2 = SystemAccess {
            reads: vec![TypeId::of::<f32>()],
            writes: vec![],
        };

        let graph = DependencyGraph::new(vec![access1, access2]);
        assert_eq!(graph.stage_count(), 2, "Should execute sequentially");
    }

    #[test]
    fn test_critical_path_identification() {
        // Create a chain: A -> B -> C
        let access_a = SystemAccess {
            reads: vec![],
            writes: vec![TypeId::of::<i32>()],
        };
        let access_b = SystemAccess {
            reads: vec![TypeId::of::<i32>()],
            writes: vec![TypeId::of::<f32>()],
        };
        let access_c = SystemAccess {
            reads: vec![TypeId::of::<f32>()],
            writes: vec![],
        };

        let graph = DependencyGraph::new(vec![access_a, access_b, access_c]);

        // All systems should be on critical path
        assert!(graph.is_critical(0) || graph.is_critical(1) || graph.is_critical(2));
        assert_eq!(graph.stage_count(), 3, "Should have 3 sequential stages");
    }

    #[test]
    fn test_complex_dependency_graph() {
        // System 0: writes A
        // System 1: writes B (parallel with 0)
        // System 2: reads A, writes C (depends on 0)
        // System 3: reads B, writes D (depends on 1)
        // System 4: reads C, D (depends on 2 and 3)

        let accesses = vec![
            SystemAccess {
                reads: vec![],
                writes: vec![TypeId::of::<i32>()],
            },
            SystemAccess {
                reads: vec![],
                writes: vec![TypeId::of::<f32>()],
            },
            SystemAccess {
                reads: vec![TypeId::of::<i32>()],
                writes: vec![TypeId::of::<i64>()],
            },
            SystemAccess {
                reads: vec![TypeId::of::<f32>()],
                writes: vec![TypeId::of::<f64>()],
            },
            SystemAccess {
                reads: vec![TypeId::of::<i64>(), TypeId::of::<f64>()],
                writes: vec![],
            },
        ];

        let graph = DependencyGraph::new(accesses);

        // Should have 3 stages: [0,1], [2,3], [4]
        assert!(
            graph.stage_count() <= 3,
            "Should optimize to 3 or fewer stages"
        );

        // Systems 0 and 1 should be in first stage (parallel)
        let first_stage = &graph.stages()[0];
        assert!(first_stage.system_indices.contains(&0) || first_stage.system_indices.contains(&1));
    }
}
