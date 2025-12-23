//! Convenient re-exports of commonly used types.
//!
//! The prelude can be imported with:
//! ```
//! use archetype_ecs::prelude::*;
//! ```

pub use crate::app::App;
pub use crate::component::Component;
pub use crate::debug::{Diagnostics, WorldInspector};
pub use crate::entity::EntityId;
pub use crate::error::Result;
pub use crate::hierarchy::{Children, Parent};
pub use crate::plugin::Plugin;
pub use crate::query::{Entity, Query, QueryMut, QueryState};
pub use crate::reflection::{Reflect, TypeRegistry};
pub use crate::time::{FixedTime, Time};
pub use crate::transform::{GlobalTransform, LocalTransform, Quat, Vec3};
pub use crate::world::World;
