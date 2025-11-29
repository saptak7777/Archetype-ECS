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

//! Command buffer with struct variants

use crate::entity::EntityId;
use crate::error::Result;
use crate::world::World;

/// Deferred command for world mutations  
#[derive(Debug)]
pub enum Command {
    /// Spawn entity with closure
    Spawn {
        bundle_fn: fn(&mut World) -> Result<()>,
    },

    /// Despawn entity
    Despawn(EntityId),

    /// Add component to entity
    AddComponent(EntityId),

    /// Remove component from entity
    RemoveComponent(EntityId),
}

/// Command buffer for deferred operations
pub struct CommandBuffer {
    commands: Vec<Command>,
}

impl CommandBuffer {
    /// Create new command buffer
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
        }
    }

    /// Create with capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            commands: Vec::with_capacity(capacity),
        }
    }

    /// Queue spawn command
    pub fn spawn(&mut self, bundle_fn: fn(&mut World) -> Result<()>) {
        self.commands.push(Command::Spawn { bundle_fn });
    }

    /// Queue despawn command
    pub fn despawn(&mut self, entity: EntityId) {
        self.commands.push(Command::Despawn(entity));
    }

    /// Get commands
    pub fn commands(&self) -> &[Command] {
        &self.commands
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }

    /// Get length
    pub fn len(&self) -> usize {
        self.commands.len()
    }

    /// Clear buffer
    pub fn clear(&mut self) {
        self.commands.clear();
    }

    /// Drain commands
    pub fn drain(&mut self) -> std::vec::Drain<'_, Command> {
        self.commands.drain(..)
    }

    /// Iterate commands
    pub fn iter(&self) -> std::slice::Iter<'_, Command> {
        self.commands.iter()
    }
}

impl Default for CommandBuffer {
    fn default() -> Self {
        Self::new()
    }
}

impl IntoIterator for CommandBuffer {
    type Item = Command;
    type IntoIter = std::vec::IntoIter<Command>;

    fn into_iter(self) -> Self::IntoIter {
        self.commands.into_iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use slotmap::Key;

    #[test]
    fn test_command_buffer() {
        let mut buffer = CommandBuffer::new();
        assert!(buffer.is_empty());
        assert_eq!(buffer.len(), 0);

        let entity = EntityId::null();
        buffer.despawn(entity);

        assert!(!buffer.is_empty());
        assert_eq!(buffer.len(), 1);
    }

    #[test]
    fn test_command_buffer_clear() {
        let mut buffer = CommandBuffer::new();
        let entity = EntityId::null();
        buffer.despawn(entity);
        buffer.clear();
        assert_eq!(buffer.len(), 0);
    }
}
