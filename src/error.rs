// Copyright 2024 Saptak Santra
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Error types

use std::fmt;

/// ECS error type
#[derive(Debug, Clone)]
pub enum EcsError {
    /// Entity not found
    EntityNotFound,

    /// Component not found
    ComponentNotFound,

    /// Archetype not found
    ArchetypeNotFound,

    /// Invalid entity ID
    InvalidEntity,

    /// Command buffer error
    CommandError(String),

    /// System cycle detected (Phase 4)
    SystemCycleDetected,

    /// Schedule error (Phase 4)
    ScheduleError(String),

    /// System not found (Phase 4)
    SystemNotFound,
}

impl fmt::Display for EcsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EcsError::EntityNotFound => write!(f, "Entity not found"),
            EcsError::ComponentNotFound => write!(f, "Component not found"),
            EcsError::ArchetypeNotFound => write!(f, "Archetype not found"),
            EcsError::InvalidEntity => write!(f, "Invalid entity ID"),
            EcsError::CommandError(msg) => write!(f, "Command error: {msg}"),
            EcsError::SystemCycleDetected => write!(f, "System dependency cycle detected"),
            EcsError::ScheduleError(msg) => write!(f, "Schedule error: {msg}"),
            EcsError::SystemNotFound => write!(f, "System not found"),
        }
    }
}

impl std::error::Error for EcsError {}

/// Result type alias
pub type Result<T> = std::result::Result<T, EcsError>;
