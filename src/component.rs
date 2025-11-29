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

//! Component and Bundle traits
//!
//! Components are data attached to entities.
//! Bundles group multiple components for spawning.

use std::any::TypeId;

use smallvec::{smallvec, SmallVec};

use crate::archetype::Archetype;

/// Maximum number of components supported by Bundle implementations
pub const MAX_BUNDLE_COMPONENTS: usize = 8;

/// Marker trait for components
///
/// Components must be 'static (no borrowed data)
pub trait Component: 'static + Send + Sync {}

/// Automatically implement Component for all valid types
impl<T: 'static + Send + Sync> Component for T {}

/// Bundle of components
///
/// Allows spawning entities with multiple components at once.
pub trait Bundle: Send + Sync + 'static {
    /// Get type IDs of all components in bundle
    fn type_ids() -> SmallVec<[TypeId; MAX_BUNDLE_COMPONENTS]>
    where
        Self: Sized;

    /// Ensure component columns exist in an archetype
    fn register_components(archetype: &mut Archetype)
    where
        Self: Sized;

    /// Write components to raw pointers
    ///
    /// # Safety
    /// Caller must ensure pointers are valid and properly aligned
    unsafe fn write_components(self, ptrs: &[*mut u8]);
}

// DO NOT implement Bundle for T: Component
// This conflicts with tuple implementations
// Instead, implement only for tuples

// Macro for tuple Bundle implementations
macro_rules! impl_bundle {
    ($($T:ident),*) => {
        impl<$($T: Component),*> Bundle for ($($T,)*) {
            fn type_ids() -> SmallVec<[TypeId; MAX_BUNDLE_COMPONENTS]> {
                smallvec![$(TypeId::of::<$T>()),*]
            }

            fn register_components(archetype: &mut Archetype) {
                $(archetype.register_component::<$T>();)*
            }

            #[allow(non_snake_case)]
            unsafe fn write_components(self, ptrs: &[*mut u8]) {
                let ($($T,)*) = self;
                let mut i = 0;
                $(
                    std::ptr::write(ptrs[i] as *mut $T, $T);
                    i += 1;
                )*
                let _ = i; // Suppress unused warning
            }
        }
    };
}

// Implement for tuples of 1-8 components
impl_bundle!(A);
impl_bundle!(A, B);
impl_bundle!(A, B, C);
impl_bundle!(A, B, C, D);
impl_bundle!(A, B, C, D, E);
impl_bundle!(A, B, C, D, E, F);
impl_bundle!(A, B, C, D, E, F, G);
impl_bundle!(A, B, C, D, E, F, G, H);

#[cfg(test)]
mod tests {
    #![allow(dead_code)]
    use super::*;

    #[test]
    fn test_single_component() {
        #[derive(Debug, Clone, Copy)]
        struct Position {
            x: f32,
            y: f32,
        }

        let type_ids = <(Position,)>::type_ids();
        assert_eq!(type_ids.len(), 1);
        assert_eq!(type_ids[0], TypeId::of::<Position>());
    }

    #[test]
    fn test_multiple_components() {
        #[derive(Debug, Clone, Copy)]
        struct Position {
            x: f32,
        }

        #[derive(Debug, Clone, Copy)]
        struct Velocity {
            x: f32,
        }

        let type_ids = <(Position, Velocity)>::type_ids();
        assert_eq!(type_ids.len(), 2);
    }
}
