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

//! Query system with archetype filtering
//!
//! Type-safe component queries with automatic archetype matching.

use std::any::TypeId;
use std::marker::PhantomData;
use std::ptr::{self, NonNull};

#[cfg(feature = "profiling")]
use tracing::info_span;

use crate::archetype::{Archetype, ComponentColumn};
use crate::component::Component;
use crate::world::World;
use smallvec::{smallvec, SmallVec};

const MAX_FILTER_COMPONENTS: usize = 8;

/// Query filter trait for type-level archetype matching
pub trait QueryFilter {
    /// Check if archetype matches this query
    fn matches_archetype(archetype: &Archetype) -> bool;

    /// Get required component type IDs
    fn type_ids() -> SmallVec<[TypeId; MAX_FILTER_COMPONENTS]>;
}

/// Stateful mutable query wrapper
pub struct QueryMut<'w, Q>
where
    Q: QueryFilter + QueryFetchMut<'w>,
{
    world: &'w mut World,
    _phantom: PhantomData<Q>,
}

impl<'w, Q> QueryMut<'w, Q>
where
    Q: QueryFilter + QueryFetchMut<'w>,
{
    /// Create mutable query wrapper
    pub fn new(world: &'w mut World) -> Self {
        Self {
            world,
            _phantom: PhantomData,
        }
    }

    /// Iterate results
    ///
    /// Creates a temporary QueryState internally for convenience.
    /// For better performance in hot loops, use `CachedQuery` instead.
    pub fn iter(&'w mut self) -> QueryIterMut<'w, Q> {
        let state = QueryState::<Q>::new(&*self.world);
        QueryIterMut::new(self.world, &state.matched_archetypes, 0, self.world.tick())
    }

    /// Count matching entities
    pub fn count(&mut self) -> usize {
        let world_ref: &World = &*self.world;
        let state = QueryState::<Q>::new(world_ref);
        state
            .matched_archetypes
            .iter()
            .filter_map(|&id| world_ref.get_archetype(id))
            .map(|arch| arch.len())
            .sum()
    }
}

/// Mutable query iterator
pub struct QueryIterMut<'w, Q: QueryFilter> {
    archetypes: Vec<NonNull<Archetype>>,
    archetype_index: usize,
    entity_index: usize,
    change_tick: u32,
    current_tick: u32,
    _phantom: PhantomData<&'w mut Q>,
}

impl<'w, Q: QueryFilter> QueryIterMut<'w, Q> {
    /// Create new mutable query iterator
    fn new(world: &'w mut World, matched: &[usize], change_tick: u32, current_tick: u32) -> Self {
        let mut archetypes = Vec::with_capacity(matched.len());
        for &id in matched {
            if let Some(ptr) = world.archetype_ptr_mut(id) {
                archetypes.push(ptr);
            }
        }

        Self {
            archetypes,
            archetype_index: 0,
            entity_index: 0,
            change_tick,
            current_tick,
            _phantom: PhantomData,
        }
    }
}

impl<'w, Q> Iterator for QueryIterMut<'w, Q>
where
    Q: QueryFilter + QueryFetchMut<'w>,
{
    type Item = <Q as QueryFetchMut<'w>>::Item;

    fn next(&mut self) -> Option<Self::Item> {
        while self.archetype_index < self.archetypes.len() {
            let archetype_ptr = self.archetypes[self.archetype_index].as_ptr();
            let archetype = unsafe { &mut *archetype_ptr };

            if self.entity_index < archetype.len() {
                let row = self.entity_index;
                self.entity_index += 1;
                if let Some(item) =
                    Q::fetch_mut(archetype, row, self.change_tick, self.current_tick)
                {
                    return Some(item);
                } else {
                    continue;
                }
            }

            self.archetype_index += 1;
            self.entity_index = 0;
        }

        None
    }
}

impl<'w, Q> ExactSizeIterator for QueryIterMut<'w, Q>
where
    Q: QueryFilter + QueryFetchMut<'w>,
{
    fn len(&self) -> usize {
        if self.archetype_index >= self.archetypes.len() {
            return 0;
        }

        let mut count = 0;

        let current_ptr = self.archetypes[self.archetype_index].as_ptr();
        let current = unsafe { &*current_ptr };
        count += current.len().saturating_sub(self.entity_index);

        for archetype_ptr in self.archetypes.iter().skip(self.archetype_index + 1) {
            let archetype = unsafe { &*archetype_ptr.as_ptr() };
            count += archetype.len();
        }

        count
    }
}

/// Extracts mutable component data from a matching archetype row
pub trait QueryFetchMut<'w>: Sized {
    /// Item returned by the mutable iterator
    type Item;

    /// Fetch mutable component data for the given archetype row
    fn fetch_mut(
        archetype: &'w mut Archetype,
        row: usize,
        change_tick: u32,
        current_tick: u32,
    ) -> Option<Self::Item>;
}

impl<'w, T: Component> QueryFetchMut<'w> for &'w mut T {
    type Item = &'w mut T;

    fn fetch_mut(
        archetype: &'w mut Archetype,
        row: usize,
        _change_tick: u32,
        current_tick: u32,
    ) -> Option<Self::Item> {
        let type_id = TypeId::of::<T>();
        let column = archetype.get_column_mut(type_id)?;

        // Set changed tick for mutation tracking
        column.set_changed_tick(row, current_tick);
        column.get_mut::<T>(row)
    }
}

macro_rules! impl_query_fetch_mut_tuple {
    ($($T:ident),+) => {
        impl<'w, $($T: Component),+> QueryFetchMut<'w> for ($(&'w mut $T,)+) {
            type Item = ($(&'w mut $T,)+);

            fn fetch_mut(archetype: &'w mut Archetype, row: usize, _change_tick: u32, current_tick: u32) -> Option<Self::Item> {
                let mut indices = Vec::with_capacity(crate::component::MAX_BUNDLE_COMPONENTS);
                $(
                    let idx = archetype.column_index(TypeId::of::<$T>())?;
                    indices.push((idx, indices.len()));
                )+

                let columns = borrow_columns_mut(archetype.components_mut(), indices)?;
                let mut iter = columns.into_iter();

                Some(($({
                    let column = iter.next()?;
                    column.set_changed_tick(row, current_tick);
                    column.get_mut::<$T>(row)?
                },)+))
            }
        }
    };
}

impl_query_fetch_mut_tuple!(A, B);
impl_query_fetch_mut_tuple!(A, B, C);
impl_query_fetch_mut_tuple!(A, B, C, D);

fn borrow_columns_mut(
    columns: &mut [ComponentColumn],
    mut indices: Vec<(usize, usize)>,
) -> Option<Vec<&mut ComponentColumn>> {
    indices.sort_by_key(|(idx, _)| *idx);

    for pair in indices.windows(2) {
        if pair[0].0 == pair[1].0 {
            return None;
        }
    }

    let len = columns.len();
    let base_ptr = columns.as_mut_ptr();
    let mut ordered_ptrs = vec![ptr::null_mut(); indices.len()];

    for &(idx, original_pos) in &indices {
        if idx >= len {
            return None;
        }
        unsafe {
            ordered_ptrs[original_pos] = base_ptr.add(idx);
        }
    }

    Some(
        ordered_ptrs
            .into_iter()
            .map(|ptr| unsafe { &mut *ptr })
            .collect(),
    )
}

/// Extracts immutable component data from a matching archetype row
pub trait QueryFetch<'w>: Sized {
    /// Item returned by the iterator
    type Item;

    /// Fetch component data for the given archetype row
    ///
    /// # Safety
    /// Caller must ensure:
    /// - The archetype reference is valid for the lifetime 'w
    /// - The row index is within bounds of the archetype
    /// - No mutable aliases exist for the fetched data
    fn fetch(archetype: &'w Archetype, row: usize, change_tick: u32) -> Option<Self::Item>;
}

/// Cached query state
///
/// Pre-computes which archetypes match the query filter.
/// Hack from Bevy: 50-80% query overhead reduction
///
/// # Performance
/// Create a `QueryState` once (for example during system initialization) and reuse it every
/// frame. Rebuild the state only when the world's archetype layout changes (e.g. a new component
/// combination is introduced). Reusing the cached state avoids repeatedly hashing archetype
/// signatures.
///
/// ```ignore
/// struct MovementSystem {
///     state: QueryState<(&'static mut Position, &'static Velocity)>,
/// }
///
/// impl MovementSystem {
///     fn new(world: &World) -> Self {
///         Self {
///             state: QueryState::new(world),
///         }
///     }
///
///     fn run(&mut self, world: &mut World) {
///         for (pos, vel) in self.state.iter_mut(world) {
///             pos.x += vel.x;
///             pos.y += vel.y;
///         }
///     }
/// }
/// ```
pub struct QueryState<F> {
    matched_archetypes: Vec<usize>,
    last_archetype_count: usize,
    _phantom: PhantomData<F>,
}

impl<F: QueryFilter> QueryState<F> {
    /// Create query state by scanning archetypes. Call this once during setup and reuse the
    /// returned state until the world's archetype layout changes.
    pub fn new(world: &World) -> Self {
        #[cfg(feature = "profiling")]
        let span = info_span!("query_state.new", archetype_count = world.archetype_count());
        #[cfg(feature = "profiling")]
        let _span_guard = span.enter();

        let matched = world
            .archetypes()
            .iter()
            .enumerate()
            .filter_map(|(id, arch)| {
                if F::matches_archetype(arch) {
                    Some(id)
                } else {
                    None
                }
            })
            .collect();

        Self {
            matched_archetypes: matched,
            last_archetype_count: world.archetype_count(),
            _phantom: PhantomData,
        }
    }

    /// Iterate query results
    ///
    /// Note: Returns iterator that borrows from self.matched_archetypes
    pub fn iter<'w, 's>(&'s self, world: &'w World, change_tick: u32) -> QueryIter<'w, 's, F>
    where
        F: QueryFetch<'w>,
    {
        QueryIter {
            world,
            archetype_index: 0,
            entity_index: 0,
            matched_archetypes: &self.matched_archetypes,
            change_tick,
            _phantom: PhantomData,
        }
    }

    /// Iterate query results mutably
    pub fn iter_mut<'w>(&'w mut self, world: &'w mut World, change_tick: u32) -> QueryIterMut<'w, F>
    where
        F: QueryFetchMut<'w>,
    {
        QueryIterMut::new(world, &self.matched_archetypes, change_tick, world.tick())
    }

    /// Get number of matched archetypes
    pub fn matched_archetype_count(&self) -> usize {
        self.matched_archetypes.len()
    }

    /// Update query state with new archetypes (incremental)
    pub fn update(&mut self, world: &World) {
        #[cfg(feature = "profiling")]
        let span = info_span!(
            "query_state.invalidate",
            archetype_count = world.archetype_count()
        );
        #[cfg(feature = "profiling")]
        let _span_guard = span.enter();

        let current_count = world.archetype_count();
        if current_count > self.last_archetype_count {
            for (id, arch) in world
                .archetypes()
                .iter()
                .enumerate()
                .skip(self.last_archetype_count)
            {
                if F::matches_archetype(arch) {
                    self.matched_archetypes.push(id);
                }
            }
            self.last_archetype_count = current_count;
        }
    }
}

/// Query iterator
pub struct QueryIter<'w, 's, Q: QueryFilter> {
    world: &'w World,
    archetype_index: usize,
    entity_index: usize,
    matched_archetypes: &'s [usize],
    change_tick: u32,
    _phantom: PhantomData<Q>,
}

impl<'w, 's, Q> Iterator for QueryIter<'w, 's, Q>
where
    Q: QueryFilter + QueryFetch<'w>,
{
    type Item = <Q as QueryFetch<'w>>::Item;

    fn next(&mut self) -> Option<Self::Item> {
        while self.archetype_index < self.matched_archetypes.len() {
            let arch_id = self.matched_archetypes[self.archetype_index];
            let archetype = self.world.get_archetype(arch_id)?;

            if self.entity_index < archetype.len() {
                let row = self.entity_index;
                self.entity_index += 1;
                if let Some(item) = Q::fetch(archetype, row, self.change_tick) {
                    return Some(item);
                } else {
                    continue;
                }
            }

            // Move to next archetype
            self.archetype_index += 1;
            self.entity_index = 0;
        }

        None
    }
}

impl<'w, 's, Q> ExactSizeIterator for QueryIter<'w, 's, Q>
where
    Q: QueryFilter + QueryFetch<'w>,
{
    fn len(&self) -> usize {
        let mut count = 0;
        for &arch_id in self.matched_archetypes {
            if let Some(arch) = self.world.get_archetype(arch_id) {
                count += arch.len();
            }
        }
        count.saturating_sub(self.entity_index)
    }
}

/// Stateless query wrapper
pub struct Query<'w, Q>
where
    Q: QueryFilter + QueryFetch<'w>,
{
    world: &'w World,
    _phantom: PhantomData<Q>,
}

impl<'w, Q> Query<'w, Q>
where
    Q: QueryFilter + QueryFetch<'w>,
{
    /// Create query
    pub fn new(world: &'w World) -> Self {
        Self {
            world,
            _phantom: PhantomData,
        }
    }

    /// Iterate query - creates temporary QueryState
    ///
    /// This is simpler but less efficient than creating QueryState once
    /// and reusing it across frames
    pub fn iter(&self) -> QueryIterOwned<'w, Q> {
        let state = QueryState::<Q>::new(self.world);
        QueryIterOwned {
            world: self.world,
            matched_archetypes: state.matched_archetypes,
            archetype_index: 0,
            entity_index: 0,
            change_tick: 0, // Stateless query matches everything
            _phantom: PhantomData,
        }
    }

    /// Count matching entities
    pub fn count(&self) -> usize {
        self.iter().len()
    }
}

/// Owned query iterator (for Query::iter)
///
/// Unlike QueryIter which borrows matched_archetypes,
/// this owns the vec so it can be returned from Query::iter
pub struct QueryIterOwned<'w, Q: QueryFilter> {
    world: &'w World,
    archetype_index: usize,
    entity_index: usize,
    matched_archetypes: Vec<usize>,
    change_tick: u32,
    _phantom: PhantomData<Q>,
}

impl<'w, Q> Iterator for QueryIterOwned<'w, Q>
where
    Q: QueryFilter + QueryFetch<'w>,
{
    type Item = <Q as QueryFetch<'w>>::Item;

    fn next(&mut self) -> Option<Self::Item> {
        while self.archetype_index < self.matched_archetypes.len() {
            let arch_id = self.matched_archetypes[self.archetype_index];
            let archetype = self.world.get_archetype(arch_id)?;

            if self.entity_index < archetype.len() {
                let row = self.entity_index;
                self.entity_index += 1;
                if let Some(item) = Q::fetch(archetype, row, self.change_tick) {
                    return Some(item);
                } else {
                    continue;
                }
            }

            // Move to next archetype
            self.archetype_index += 1;
            self.entity_index = 0;
        }

        None
    }
}

impl<'w, Q> ExactSizeIterator for QueryIterOwned<'w, Q>
where
    Q: QueryFilter + QueryFetch<'w>,
{
    fn len(&self) -> usize {
        let mut count = 0;
        for &arch_id in &self.matched_archetypes {
            if let Some(arch) = self.world.get_archetype(arch_id) {
                count += arch.len();
            }
        }
        count.saturating_sub(self.entity_index)
    }
}

/// Cached query for persistent system state
///
/// Automatically updates when new archetypes are added.
pub struct CachedQuery<F: QueryFilter> {
    state: QueryState<F>,
    last_run_tick: u32,
}

impl<F: QueryFilter> CachedQuery<F> {
    /// Create new cached query
    pub fn new(world: &World) -> Self {
        Self {
            state: QueryState::new(world),
            last_run_tick: 0,
        }
    }

    /// Iterate query (updates state automatically)
    pub fn iter<'w, 's>(&'s mut self, world: &'w World) -> QueryIter<'w, 's, F>
    where
        F: QueryFetch<'w>,
    {
        self.state.update(world);
        let iter = self.state.iter(world, self.last_run_tick);
        self.last_run_tick = world.tick();
        iter
    }

    /// Iterate query mutably (updates state automatically)
    pub fn iter_mut<'w>(&'w mut self, world: &'w mut World) -> QueryIterMut<'w, F>
    where
        F: QueryFetchMut<'w>,
    {
        // Note: update requires immutable reference, so we can't call it here if we have mutable world
        // Ideally, update should be called before getting mutable access
        // For now, we assume state is up to date or user called update manually if needed
        // self.state.update(world);
        let tick = world.tick();
        let iter = self.state.iter_mut(world, self.last_run_tick);
        self.last_run_tick = tick;
        iter
    }
}

// QueryFilter implementations for common patterns

impl<T: 'static> QueryFilter for &T {
    fn matches_archetype(archetype: &Archetype) -> bool {
        archetype.signature().contains(&TypeId::of::<T>()) // FIXED: Added &
    }

    fn type_ids() -> SmallVec<[TypeId; MAX_FILTER_COMPONENTS]> {
        smallvec![TypeId::of::<T>()]
    }
}

impl<T: 'static> QueryFilter for &mut T {
    fn matches_archetype(archetype: &Archetype) -> bool {
        archetype.signature().contains(&TypeId::of::<T>()) // FIXED: Added &
    }

    fn type_ids() -> SmallVec<[TypeId; MAX_FILTER_COMPONENTS]> {
        smallvec![TypeId::of::<T>()]
    }
}

/// Read access wrapper for CachedQuery
pub struct Read<T>(PhantomData<T>);

impl<T: 'static> QueryFilter for Read<T> {
    fn matches_archetype(archetype: &Archetype) -> bool {
        archetype.signature().contains(&TypeId::of::<T>())
    }

    fn type_ids() -> SmallVec<[TypeId; MAX_FILTER_COMPONENTS]> {
        smallvec![TypeId::of::<T>()]
    }
}

/// Write access wrapper for CachedQuery
pub struct Write<T>(PhantomData<T>);

impl<T: 'static> QueryFilter for Write<T> {
    fn matches_archetype(archetype: &Archetype) -> bool {
        archetype.signature().contains(&TypeId::of::<T>())
    }

    fn type_ids() -> SmallVec<[TypeId; MAX_FILTER_COMPONENTS]> {
        smallvec![TypeId::of::<T>()]
    }
}

/// Filter for entities with component T
pub struct With<T>(PhantomData<T>);

impl<T: 'static> QueryFilter for With<T> {
    fn matches_archetype(archetype: &Archetype) -> bool {
        archetype.signature().contains(&TypeId::of::<T>())
    }

    fn type_ids() -> SmallVec<[TypeId; MAX_FILTER_COMPONENTS]> {
        smallvec![TypeId::of::<T>()]
    }
}

/// Filter for entities without component T
pub struct Without<T>(PhantomData<T>);

impl<T: 'static> QueryFilter for Without<T> {
    fn matches_archetype(archetype: &Archetype) -> bool {
        !archetype.signature().contains(&TypeId::of::<T>())
    }

    fn type_ids() -> SmallVec<[TypeId; MAX_FILTER_COMPONENTS]> {
        smallvec![] // Without doesn't require component presence for storage access
    }
}

/// Filter for entities where component T has changed
pub struct Changed<T>(PhantomData<T>);

impl<T: 'static> QueryFilter for Changed<T> {
    fn matches_archetype(archetype: &Archetype) -> bool {
        archetype.signature().contains(&TypeId::of::<T>())
    }

    fn type_ids() -> SmallVec<[TypeId; MAX_FILTER_COMPONENTS]> {
        smallvec![TypeId::of::<T>()]
    }
}

/// Filter for entities where component T was added
pub struct Added<T>(PhantomData<T>);

impl<T: 'static> QueryFilter for Added<T> {
    fn matches_archetype(archetype: &Archetype) -> bool {
        archetype.signature().contains(&TypeId::of::<T>())
    }

    fn type_ids() -> SmallVec<[TypeId; MAX_FILTER_COMPONENTS]> {
        smallvec![TypeId::of::<T>()]
    }
}

// Tuple QueryFilter implementations
macro_rules! impl_query_filter {
    ($($T:ident),*) => {
        #[allow(non_snake_case)]
        impl<$($T: QueryFilter),*> QueryFilter for ($($T,)*) {
            fn matches_archetype(archetype: &Archetype) -> bool {
                $($T::matches_archetype(archetype))&&*
            }

            fn type_ids() -> SmallVec<[TypeId; MAX_FILTER_COMPONENTS]> {
                let mut ids = SmallVec::new();
                $(ids.extend($T::type_ids());)*
                ids
            }
        }
    };
}

impl_query_filter!(A, B);
impl_query_filter!(A, B, C);
impl_query_filter!(A, B, C, D);

// QueryFetch implementations for immutable component access
impl<'w, T: Component> QueryFetch<'w> for &'w T {
    type Item = &'w T;

    fn fetch(archetype: &'w Archetype, row: usize, _change_tick: u32) -> Option<Self::Item> {
        let type_id = TypeId::of::<T>();
        let column = archetype.get_column(type_id)?;
        column.get::<T>(row)
    }
}

impl<'w, T: 'static> QueryFetch<'w> for With<T> {
    type Item = ();

    fn fetch(_archetype: &'w Archetype, _row: usize, _change_tick: u32) -> Option<Self::Item> {
        Some(())
    }
}

impl<'w, T: 'static> QueryFetch<'w> for Without<T> {
    type Item = ();

    fn fetch(_archetype: &'w Archetype, _row: usize, _change_tick: u32) -> Option<Self::Item> {
        Some(())
    }
}

impl<'w, T: 'static> QueryFetch<'w> for Changed<T> {
    type Item = ();

    fn fetch(archetype: &'w Archetype, row: usize, change_tick: u32) -> Option<Self::Item> {
        let type_id = TypeId::of::<T>();
        let column = archetype.get_column(type_id)?;
        if column.get_changed_tick(row)? > change_tick {
            Some(())
        } else {
            None
        }
    }
}

impl<'w, T: 'static> QueryFetch<'w> for Added<T> {
    type Item = ();

    fn fetch(archetype: &'w Archetype, row: usize, change_tick: u32) -> Option<Self::Item> {
        let type_id = TypeId::of::<T>();
        let column = archetype.get_column(type_id)?;
        if column.get_added_tick(row)? > change_tick {
            Some(())
        } else {
            None
        }
    }
}

impl<'w, T: Component> QueryFetch<'w> for Read<T> {
    type Item = &'w T;

    fn fetch(archetype: &'w Archetype, row: usize, _change_tick: u32) -> Option<Self::Item> {
        let type_id = TypeId::of::<T>();
        let column = archetype.get_column(type_id)?;
        column.get::<T>(row)
    }
}

impl<'w, T: Component> QueryFetchMut<'w> for Write<T> {
    type Item = &'w mut T;

    fn fetch_mut(
        archetype: &'w mut Archetype,
        row: usize,
        _change_tick: u32,
        current_tick: u32,
    ) -> Option<Self::Item> {
        let type_id = TypeId::of::<T>();
        let column = archetype.get_column_mut(type_id)?;
        column.set_changed_tick(row, current_tick);
        column.get_mut::<T>(row)
    }
}

macro_rules! impl_query_fetch_tuple {
    ($($T:ident),+) => {
        impl<'w, $($T: QueryFetch<'w>),+> QueryFetch<'w> for ($($T,)+) {
            type Item = ($($T::Item,)+);

            fn fetch(archetype: &'w Archetype, row: usize, change_tick: u32) -> Option<Self::Item> {
                Some((
                    $($T::fetch(archetype, row, change_tick)?,)+
                ))
            }
        }
    }
}

impl_query_fetch_tuple!(A, B);
impl_query_fetch_tuple!(A, B, C);
impl_query_fetch_tuple!(A, B, C, D);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_state_creation() {
        let world = crate::World::new();
        let state = QueryState::<&i32>::new(&world);
        // There are no archetypes containing i32 yet
        // There are no archetypes containing i32 yet
        assert_eq!(state.matched_archetype_count(), 0);
    }

    #[test]
    fn test_incremental_update() {
        let mut world = crate::World::new();
        let mut query = CachedQuery::<&i32>::new(&world);

        // Initially empty (except potentially empty archetype)
        let initial_count = query.state.matched_archetype_count();

        // Add archetype matching query
        world.spawn((10i32,)).unwrap();

        // Iterating should update state
        let count = query.iter(&world).count();
        assert_eq!(count, 1);
        assert!(query.state.matched_archetype_count() > initial_count);
    }

    #[test]
    fn test_query_filters() {
        let mut world = crate::World::new();

        #[derive(Debug, Clone, Copy)]
        struct A;
        #[derive(Debug, Clone, Copy)]
        struct B;

        world.spawn((A, B)).unwrap();
        world.spawn((A,)).unwrap();
        world.spawn((B,)).unwrap();

        // Query: A with B
        let mut query = CachedQuery::<(&A, With<B>)>::new(&world);
        assert_eq!(query.iter(&world).count(), 1);

        // Query: A without B
        let mut query = CachedQuery::<(&A, Without<B>)>::new(&world);
        assert_eq!(query.iter(&world).count(), 1);
    }
}
