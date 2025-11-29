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

//! AAA ECS - High-performance Entity Component System
//!
//! Production-ready ECS with parallel scheduler.

pub mod archetype;
pub mod command;
pub mod component;
pub mod entity;
pub mod error;
pub mod query;
pub mod utils;
pub mod world;

// Phase 4: Parallel Scheduler
pub mod executor;
pub mod schedule;
pub mod system;

// Re-exports for convenience
pub use archetype::Archetype;
pub use command::CommandBuffer;
pub use component::{Bundle, Component};
pub use entity::EntityId;
pub use error::{EcsError, Result};
pub use query::{Query, QueryFetchMut, QueryFilter, QueryMut, QueryState};
pub use world::World;

// Phase 4 exports
pub use executor::{Executor, SystemProfiler};
pub use schedule::{Schedule, Stage, SystemGraph};
pub use system::{BoxedSystem, System, SystemAccess, SystemId};

#[cfg(test)]
mod tests;

#[cfg(all(test, not(target_env = "msvc")))]
mod ecs_bench;
