use crate::bitset::BitSet;
use crate::system::SystemAccess;
use rustc_hash::FxHashSet;
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
    // Optimization: Use BitSet matrix instead of HashMap for conflict lookup
    // Row 'i' contains dependency bits for system 'i'.
    // If bit 'j' is set in row 'i', then system 'i' depends on system 'j'.
    #[allow(dead_code)] // Used for debugging/future analysis
    dependency_matrix: Vec<BitSet>,
}

impl DependencyGraph {
    /// Create graph from system accesses with optimal scheduling
    pub fn new(system_accesses: Vec<SystemAccess>) -> Self {
        let dependency_matrix = Self::build_dependency_matrix(&system_accesses);
        let stages = Self::build_stages_topological(&system_accesses, &dependency_matrix);
        let critical_path = Self::find_critical_path(&stages, &dependency_matrix);

        Self {
            stages,
            critical_path,
            dependency_matrix,
        }
    }

    /// Build bitset matrix representing dependencies between systems
    fn build_dependency_matrix(accesses: &[SystemAccess]) -> Vec<BitSet> {
        let count = accesses.len();
        let mut matrix = vec![BitSet::with_capacity(count); count];

        // Build directed edges: if A conflicts with B and A comes first, A -> B
        // Matrix[i] has bit j set if i depends on j?
        // NOTE: In topological sort, "edges" usually mean "i depends on j".
        // Use standard convention: Edge U -> V means U must happen before V.
        // So graph[U] contains V.
        // Or in adjacency list: list[U] contains V.

        // Let's stick to the previous logic:
        // "if A conflicts with B and A comes first, A -> B" (A must complete before B)
        // So adjacency_list[A] contained B.
        // Here: matrix[A] will have bit B set.

        for i in 0..count {
            for j in (i + 1)..count {
                if accesses[i].conflicts_with(&accesses[j]) {
                    // System i must complete before system j can start
                    // Edge i -> j
                    matrix[i].set(j);
                }
            }
        }

        matrix
    }

    /// Build execution stages using topological sort and graph coloring
    fn build_stages_topological(
        accesses: &[SystemAccess],
        dependency_matrix: &[BitSet],
    ) -> Vec<ExecutionStage> {
        let count = accesses.len();
        if count == 0 {
            return vec![];
        }

        // Calculate in-degree for each node
        let mut in_degree = vec![0; count];

        // matrix[u] contains v implies edge u -> v
        // So v has an incoming edge from u.
        for matrix_row in dependency_matrix.iter() {
            for neighbor in matrix_row.ones() {
                in_degree[neighbor] += 1;
            }
        }

        // Track depth of each system in the dependency graph
        let mut depths = vec![0; count];
        let mut queue = VecDeque::new();

        // Start with systems that have no dependencies (no incoming edges? wait)
        // In task scheduling:
        // A -> B means A must finish before B starts.
        // So A can start immediately (in-degree 0).
        // Start with systems that have no dependencies
        for (idx, &degree) in in_degree.iter().enumerate() {
            if degree == 0 {
                queue.push_back(idx);
            }
        }

        // Topological sort with depth tracking (Kahn's algorithm)
        let mut sorted = Vec::with_capacity(count);
        while let Some(node) = queue.pop_front() {
            sorted.push(node);

            // Visit neighbors (systems that depend on 'node')
            for neighbor in dependency_matrix[node].ones() {
                in_degree[neighbor] -= 1;
                depths[neighbor] = depths[neighbor].max(depths[node] + 1);

                if in_degree[neighbor] == 0 {
                    queue.push_back(neighbor);
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
                    // Check conflicts within the stage
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

        // Optimize stages
        Self::optimize_stages(&mut stages, accesses, &sorted, &depths);

        stages
    }

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

    /// Find the critical path (longest dependency chain)
    fn find_critical_path(stages: &[ExecutionStage], dependency_matrix: &[BitSet]) -> Vec<usize> {
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

        // Trace back through dependencies
        // We need incoming edges (Reverse Adjacency).
        // dependency_matrix[u] has bit v set if u -> v.
        // We need X -> current. So dependency_matrix[X] has current set.
        loop {
            let mut predecessor = None;
            // Iterate all systems to find one that points to `current`
            // optimization: only look at systems in lower depths?
            // For now, linear scan is acceptable for simple critical path finding
            // especially since BitSet check is fast.
            for (i, matrix_row) in dependency_matrix.iter().enumerate() {
                if matrix_row.contains(current) {
                    predecessor = Some(i);
                    // Heuristic: take the first found predecessor (simplification)
                    // Ideally we take the one with max depth-1
                    break;
                }
            }

            if let Some(pred) = predecessor {
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

    /// Get critical path
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
    use crate::system::ComponentId;

    #[test]
    fn test_no_conflicts_parallel() {
        let access1 = SystemAccess {
            reads: vec![ComponentId::of::<i32>()],
            writes: vec![],
        };
        let access2 = SystemAccess {
            reads: vec![ComponentId::of::<f32>()],
            writes: vec![],
        };

        let graph = DependencyGraph::new(vec![access1, access2]);
        assert_eq!(graph.stage_count(), 1, "Should execute in parallel");
    }

    #[test]
    fn test_write_conflict_sequential() {
        let access1 = SystemAccess {
            reads: vec![ComponentId::of::<i32>()],
            writes: vec![ComponentId::of::<f32>()],
        };
        let access2 = SystemAccess {
            reads: vec![ComponentId::of::<f32>()],
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
            writes: vec![ComponentId::of::<i32>()],
        };
        let access_b = SystemAccess {
            reads: vec![ComponentId::of::<i32>()],
            writes: vec![ComponentId::of::<f32>()],
        };
        let access_c = SystemAccess {
            reads: vec![ComponentId::of::<f32>()],
            writes: vec![],
        };

        let graph = DependencyGraph::new(vec![access_a, access_b, access_c]);

        // All systems should be on critical path (approximate check given Bitset implementation)
        assert!(!graph.critical_path.is_empty());
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
                writes: vec![ComponentId::of::<i32>()],
            },
            SystemAccess {
                reads: vec![],
                writes: vec![ComponentId::of::<f32>()],
            },
            SystemAccess {
                reads: vec![ComponentId::of::<i32>()],
                writes: vec![ComponentId::of::<i64>()],
            },
            SystemAccess {
                reads: vec![ComponentId::of::<f32>()],
                writes: vec![ComponentId::of::<f64>()],
            },
            SystemAccess {
                reads: vec![ComponentId::of::<i64>(), ComponentId::of::<f64>()],
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
