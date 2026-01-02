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

    /// Event queue overflow (Phase 6)
    EventQueueOverflow,

    /// Serialization error (Phase 7)
    SerializationError(String),

    /// Deserialization error (Phase 7)
    DeserializationError(String),

    /// Resource not found (Phase 8)
    ResourceNotFound(String),

    /// Resource load error (Phase 8)
    ResourceLoadError(String),

    /// Resource memory overflow (Phase 8)
    ResourceMemoryOverflow(String),

    /// Resource deallocation error (Phase 8)
    ResourceDeallocError(String),

    /// Asset load error
    AssetLoadError(String),

    /// Asset not found
    AssetNotFound(String),

    /// Batch size too large (possible DoS attack)
    BatchTooLarge,

    /// Hierarchy operation error (cycle, self-attach, etc.)
    HierarchyError(String),

    /// Resource already exists (init_resource failed)
    ResourceAlreadyExists(std::any::TypeId),

    /// IO error (file operations, etc.)
    IoError(String),

    /// Spawn error with detailed context
    SpawnError(SpawnError),
}

/// Detailed spawn error types
#[derive(Debug, Clone)]
pub enum SpawnError {
    /// Entity capacity exhausted
    EntityCapacityExhausted {
        attempted: usize,
        capacity: usize,
    },
    /// Component registration failed
    ComponentRegistrationFailed(String),
    /// Archetype creation failed
    ArchetypeCreationFailed {
        component_count: usize,
        reason: String,
    },
}

impl fmt::Display for SpawnError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SpawnError::EntityCapacityExhausted { attempted, capacity } => {
                write!(f, "Entity capacity exhausted: attempted to spawn {attempted}, max is {capacity}")
            }
            SpawnError::ComponentRegistrationFailed(reason) => {
                write!(f, "Failed to register component: {reason}")
            }
            SpawnError::ArchetypeCreationFailed { component_count, reason } => {
                write!(f, "Failed to create archetype for {component_count} components: {reason}")
            }
        }
    }
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
            EcsError::EventQueueOverflow => write!(f, "Event queue overflow"),
            EcsError::SerializationError(msg) => write!(f, "Serialization error: {msg}"),
            EcsError::DeserializationError(msg) => write!(f, "Deserialization error: {msg}"),
            EcsError::ResourceNotFound(msg) => write!(f, "Resource not found: {msg}"),
            EcsError::ResourceLoadError(msg) => write!(f, "Resource load error: {msg}"),
            EcsError::ResourceMemoryOverflow(msg) => write!(f, "Resource memory overflow: {msg}"),
            EcsError::ResourceDeallocError(msg) => write!(f, "Resource deallocation error: {msg}"),
            EcsError::AssetLoadError(msg) => write!(f, "Asset load error: {msg}"),
            EcsError::AssetNotFound(msg) => write!(f, "Asset not found: {msg}"),
            EcsError::BatchTooLarge => write!(f, "Batch size too large (max 10,000,000)"),
            EcsError::HierarchyError(msg) => write!(f, "Hierarchy error: {msg}"),
            EcsError::ResourceAlreadyExists(type_id) => write!(f, "Resource already exists: {type_id:?}"),
            EcsError::IoError(msg) => write!(f, "IO error: {msg}"),
            EcsError::SpawnError(spawn_err) => write!(f, "Spawn error: {spawn_err}"),
        }
    }
}

impl std::error::Error for EcsError {}

impl From<std::io::Error> for EcsError {
    fn from(err: std::io::Error) -> Self {
        EcsError::IoError(err.to_string())
    }
}

impl From<SpawnError> for EcsError {
    fn from(err: SpawnError) -> Self {
        EcsError::SpawnError(err)
    }
}

/// Result type alias
pub type Result<T> = std::result::Result<T, EcsError>;
