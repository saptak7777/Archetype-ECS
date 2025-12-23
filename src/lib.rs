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

//! Archetype ECS - High-performance Entity Component System
//!
//! Production-ready ECS with parallel scheduler and advanced features.

pub mod app;
pub mod archetype;
pub mod command;
pub mod component;
pub mod debug;
pub mod dependency;
pub mod entity;
pub mod error;
pub mod event;
pub mod event_bus;
pub mod event_subscriber;
pub mod event_types;
pub mod executor;
pub mod hierarchy;
pub mod hierarchy_system;
pub mod observer;
pub mod parallel;
pub mod plugin;
pub mod prelude;
pub mod query;
pub mod reflection;
pub mod schedule;
pub mod system;
pub mod time;
pub mod transform;
pub mod world;

#[cfg(test)]
mod tests;

pub use app::*;
pub use archetype::*;
pub use command::*;
pub use component::*;
pub use dependency::*;
pub use entity::*;
pub use error::*;
pub use event::*;
pub use event_bus::*;
pub use event_subscriber::*;
pub use event_types::*;
pub use executor::*;
pub use hierarchy::*;
pub use hierarchy_system::*;
pub use observer::*;
pub use parallel::*;
pub use plugin::*;
pub use query::*;
pub use reflection::*;
pub use schedule::*;
pub use system::*;
pub use transform::*;
pub use world::*;

#[cfg(all(test, not(target_env = "msvc")))]
mod ecs_bench;
