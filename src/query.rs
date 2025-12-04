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
use std::ptr::NonNull;

#[cfg(feature = "profiling")]
use tracing::info_span;

use crate::archetype::{Archetype, ComponentColumn};
use crate::component::Component;
use crate::entity::EntityId;
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

    /// Parallel iteration over chunks
    ///
    /// This method allows processing entities in parallel chunks using Rayon.
    /// Each chunk provides typed slice access to components for SIMD optimization.
    #[cfg(feature = "parallel")]
    pub fn par_for_each_chunk<F>(&mut self, func: F)
    where
        F: Fn(crate::archetype::ArchetypeChunkMut) + Send + Sync,
    {
        use rayon::prelude::*;

        // 1. Get matched archetypes
        let state = QueryState::<Q>::new(&*self.world);

        // 2. Prepare for parallel execution
        // We need to share the world pointer safely across threads
        let world_ptr = self.world as *mut World as usize;

        // 3. Iterate over archetypes in parallel
        state.matched_archetypes.par_iter().for_each(|&arch_id| {
            // SAFETY:
            // 1. We have exclusive access to world in QueryMut
            // 2. We are reading distinct archetypes (arch_id is unique)
            // 3. We are not mutating the archetype list
            // 4. The pointer is valid for the duration of this function
            let world = unsafe { &mut *(world_ptr as *mut World) };

            if let Some(archetype) = world.get_archetype_mut(arch_id) {
                // Iterate over chunks of this archetype
                // Note: We are processing chunks of the same archetype sequentially here,
                // but archetypes are processed in parallel.
                // To parallelize chunks within an archetype, we would need to collect them first.
                // Given typical ECS usage (many archetypes or few large ones),
                // we might want to flatten the parallelism.

                // For now, let's process chunks sequentially within an archetype
                // because `chunks_mut` returns an iterator (Vec::into_iter).

                // Optimization: If archetype is large, we could use par_bridge() if we implemented it,
                // but chunks_mut returns a Vec, so we CAN use par_iter() on it!

                let chunks = archetype.chunks_mut(crate::archetype::DEFAULT_CHUNK_SIZE);

                // Process chunks in parallel (nested parallelism)
                // Rayon handles this well with work stealing
                chunks.into_par_iter().for_each(|chunk| {
                    func(chunk);
                });
            }
        });
    }
}

impl<'w, Q> IntoIterator for QueryMut<'w, Q>
where
    Q: QueryFilter + QueryFetchMut<'w> + 'w,
{
    type Item = <Q as QueryFetchMut<'w>>::Item;
    type IntoIter = QueryIterMut<'w, Q>;

    fn into_iter(self) -> Self::IntoIter {
        let state = QueryState::<Q>::new(&*self.world);
        QueryIterMut::new(self.world, &state.matched_archetypes, 0, self.world.tick())
    }
}

/// Immutable query iterator
pub struct QueryIter<'w, Q: QueryFilter>
where
    Q: QueryFetch<'w>,
{
    archetypes: Vec<NonNull<Archetype>>,
    archetype_index: usize,
    entity_index: usize,
    change_tick: u32,
    state: Option<Q::State>,
    _phantom: PhantomData<&'w Q>,
}

impl<'w, Q: QueryFilter> QueryIter<'w, Q>
where
    Q: QueryFetch<'w>,
{
    /// Create new immutable query iterator
    fn new(world: &'w World, matched: &[usize], change_tick: u32) -> Self {
        let mut archetypes = Vec::with_capacity(matched.len());
        for &id in matched {
            if let Some(ptr) = world.archetype_ptr(id) {
                archetypes.push(ptr);
            }
        }

        Self {
            archetypes,
            archetype_index: 0,
            entity_index: 0,
            change_tick,
            state: None,
            _phantom: PhantomData,
        }
    }
}

impl<'w, Q> Iterator for QueryIter<'w, Q>
where
    Q: QueryFilter + QueryFetch<'w>,
{
    type Item = <Q as QueryFetch<'w>>::Item;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.state.is_none() {
                if self.archetype_index >= self.archetypes.len() {
                    return None;
                }

                let archetype_ptr = self.archetypes[self.archetype_index].as_ptr();
                // SAFETY: Valid pointer from world, tied to 'w lifetime
                let archetype = unsafe { &*archetype_ptr };

                self.state = Q::prepare(archetype, self.change_tick);
                self.entity_index = 0;

                if self.state.is_none() {
                    self.archetype_index += 1;
                    continue;
                }
            }

            let archetype_ptr = self.archetypes[self.archetype_index].as_ptr();
            let archetype = unsafe { &*archetype_ptr };

            if self.entity_index < archetype.len() {
                let row = self.entity_index;
                self.entity_index += 1;

                // SAFETY: We checked bounds above. State is valid for this archetype.
                if let Some(item) = unsafe { Q::fetch(self.state.as_ref().unwrap(), row) } {
                    return Some(item);
                } else {
                    continue;
                }
            } else {
                self.state = None;
                self.archetype_index += 1;
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }
}

impl<'w, Q> ExactSizeIterator for QueryIter<'w, Q>
where
    Q: QueryFilter + QueryFetch<'w>,
{
    fn len(&self) -> usize {
        if self.archetype_index >= self.archetypes.len() {
            return 0;
        }

        let mut count = 0;

        let current_ptr = self.archetypes[self.archetype_index].as_ptr();
        // SAFETY: Pointer is valid for 'w lifetime and comes from world.archetype_ptr()
        let current = unsafe { &*current_ptr };
        count += current.len().saturating_sub(self.entity_index);

        for archetype_ptr in self.archetypes.iter().skip(self.archetype_index + 1) {
            // SAFETY: All pointers in self.archetypes are valid for the query lifetime
            let archetype = unsafe { &*archetype_ptr.as_ptr() };
            count += archetype.len();
        }

        count
    }
}

/// Mutable query iterator
pub struct QueryIterMut<'w, Q: QueryFilter>
where
    Q: QueryFetchMut<'w>,
{
    archetypes: Vec<NonNull<Archetype>>,
    archetype_index: usize,
    entity_index: usize,
    #[allow(dead_code)] // Reserved for future change detection features
    change_tick: u32,
    current_tick: u32,
    state: Option<Q::State>,
    _phantom: PhantomData<&'w mut Q>,
}

impl<'w, Q: QueryFilter> QueryIterMut<'w, Q>
where
    Q: QueryFetchMut<'w>,
{
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
            state: None,
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
        loop {
            // Check if we have a valid state for current archetype
            if self.state.is_none() {
                if self.archetype_index >= self.archetypes.len() {
                    return None;
                }

                let archetype_ptr = self.archetypes[self.archetype_index].as_ptr();
                // SAFETY: This is safe because:
                // 1. archetype_ptr comes from world.archetype_ptr_mut() which returns valid pointers
                // 2. The pointer is valid for the lifetime 'w (tied to world borrow)
                // 3. No other code is mutating the archetype list during iteration
                // 4. NonNull guarantees the pointer is non-null
                // 5. The archetype exists for the duration of the query
                let archetype = unsafe { &mut *archetype_ptr };

                self.state = Q::prepare(archetype, self.current_tick);
                self.entity_index = 0;

                if self.state.is_none() {
                    self.archetype_index += 1;
                    continue;
                }
            }

            // We have a state, try to fetch
            let archetype_ptr = self.archetypes[self.archetype_index].as_ptr();
            // SAFETY: Same as above
            let archetype = unsafe { &*archetype_ptr };

            if self.entity_index < archetype.len() {
                let row = self.entity_index;
                self.entity_index += 1;

                // SAFETY: We checked bounds above. State is valid for this archetype.
                // SAFETY: We checked bounds above. State is valid for this archetype.
                if let Some(item) = unsafe { Q::fetch(self.state.as_mut().unwrap(), row) } {
                    return Some(item);
                } else {
                    continue;
                }
            } else {
                // Finished this archetype
                self.state = None;
                self.archetype_index += 1;
            }
        }
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
        // SAFETY: Pointer is valid for 'w lifetime and comes from world.archetype_ptr_mut()
        let current = unsafe { &*current_ptr };
        count += current.len().saturating_sub(self.entity_index);

        for archetype_ptr in self.archetypes.iter().skip(self.archetype_index + 1) {
            // SAFETY: All pointers in self.archetypes are valid for the query lifetime
            let archetype = unsafe { &*archetype_ptr.as_ptr() };
            count += archetype.len();
        }

        count
    }
}

/// Trait for fetching component data (immutable)
///
/// # Safety
/// Implementations must ensure that `fetch` is safe to call with the state returned by `prepare`.
pub unsafe trait QueryFetch<'w>: QueryFilter {
    /// The type of data returned by the query
    type Item;
    /// State used to fetch data (e.g. column pointers)
    type State;

    /// Prepare to fetch from an archetype
    fn prepare(archetype: &'w Archetype, change_tick: u32) -> Option<Self::State>;

    /// Fetch data for a specific entity
    ///
    /// # Safety
    /// - `row` must be valid for the archetype used in `prepare`
    unsafe fn fetch(state: &Self::State, row: usize) -> Option<Self::Item>;
}

// QueryFetch implementations for immutable component access
unsafe impl<'w, T: Component> QueryFetch<'w> for &'w T {
    type Item = &'w T;
    type State = &'w ComponentColumn;

    fn prepare(archetype: &'w Archetype, _change_tick: u32) -> Option<Self::State> {
        let type_id = TypeId::of::<T>();
        archetype.get_column(type_id)
    }

    unsafe fn fetch(state: &Self::State, row: usize) -> Option<Self::Item> {
        state.get::<T>(row)
    }
}

/// Trait for fetching component data (mutable)
///
/// # Safety
/// Implementations must ensure that `fetch` is safe to call with the state returned by `prepare`.
pub unsafe trait QueryFetchMut<'w>: QueryFilter {
    /// The type of data returned by the query
    type Item;
    /// State used to fetch data (e.g. column pointers)
    type State;

    /// Prepare to fetch from an archetype
    fn prepare(archetype: &'w mut Archetype, current_tick: u32) -> Option<Self::State>;

    /// Fetch data for a specific entity
    ///
    /// # Safety
    /// - `row` must be valid for the archetype used in `prepare`
    /// - Must not be called multiple times for the same row (aliasing)
    unsafe fn fetch(state: &mut Self::State, row: usize) -> Option<Self::Item>;
}

unsafe impl<'w, T: Component> QueryFetchMut<'w> for &'w mut T {
    type Item = &'w mut T;
    type State = (*mut ComponentColumn, u32);

    fn prepare(archetype: &'w mut Archetype, current_tick: u32) -> Option<Self::State> {
        let type_id = TypeId::of::<T>();
        let column = archetype.get_column_mut(type_id)?;
        Some((column as *mut ComponentColumn, current_tick))
    }

    unsafe fn fetch(state: &mut Self::State, row: usize) -> Option<Self::Item> {
        let (column_ptr, current_tick) = state;
        // SAFETY: The column pointer is valid for the lifetime 'w and points to a valid ComponentColumn.
        // The caller ensures that row is a valid index within the column.
        let column = unsafe { &mut **column_ptr };
        column.set_changed_tick(row, *current_tick);
        column.get_mut::<T>(row)
    }
}

/// QueryFetchMut for immutable reference - allows mixed mutability tuples
/// Example: `world.query_mut::<(&Position, &mut Velocity)>()`
unsafe impl<'w, T: Component> QueryFetchMut<'w> for &'w T {
    type Item = &'w T;
    type State = *const ComponentColumn;

    fn prepare(archetype: &'w mut Archetype, _current_tick: u32) -> Option<Self::State> {
        let type_id = TypeId::of::<T>();
        archetype
            .get_column(type_id)
            .map(|col| col as *const ComponentColumn)
    }

    unsafe fn fetch(state: &mut Self::State, row: usize) -> Option<Self::Item> {
        // SAFETY: The pointer is valid for the lifetime 'w
        let column = unsafe { &**state };
        column.get::<T>(row)
    }
}

// Generic tuple implementations for QueryFetchMut
// These use QueryFetchMut bounds, allowing mixed types like (Entity, &mut T), (&T, &mut U), etc.

unsafe impl<'w, A: QueryFetchMut<'w>, B: QueryFetchMut<'w>> QueryFetchMut<'w> for (A, B)
where
    A: QueryFilter,
    B: QueryFilter,
{
    type Item = (A::Item, B::Item);
    type State = (A::State, B::State);

    fn prepare(archetype: &'w mut Archetype, current_tick: u32) -> Option<Self::State> {
        // SAFETY: We're getting non-overlapping mutable borrows through prepare
        // Each component type gets its own column pointer
        let ptr = archetype as *mut Archetype;
        let state_a = A::prepare(unsafe { &mut *ptr }, current_tick)?;
        let state_b = B::prepare(unsafe { &mut *ptr }, current_tick)?;
        Some((state_a, state_b))
    }

    unsafe fn fetch(state: &mut Self::State, row: usize) -> Option<Self::Item> {
        Some((A::fetch(&mut state.0, row)?, B::fetch(&mut state.1, row)?))
    }
}

unsafe impl<'w, A: QueryFetchMut<'w>, B: QueryFetchMut<'w>, C: QueryFetchMut<'w>> QueryFetchMut<'w>
    for (A, B, C)
where
    A: QueryFilter,
    B: QueryFilter,
    C: QueryFilter,
{
    type Item = (A::Item, B::Item, C::Item);
    type State = (A::State, B::State, C::State);

    fn prepare(archetype: &'w mut Archetype, current_tick: u32) -> Option<Self::State> {
        let ptr = archetype as *mut Archetype;
        let state_a = A::prepare(unsafe { &mut *ptr }, current_tick)?;
        let state_b = B::prepare(unsafe { &mut *ptr }, current_tick)?;
        let state_c = C::prepare(unsafe { &mut *ptr }, current_tick)?;
        Some((state_a, state_b, state_c))
    }

    unsafe fn fetch(state: &mut Self::State, row: usize) -> Option<Self::Item> {
        Some((
            A::fetch(&mut state.0, row)?,
            B::fetch(&mut state.1, row)?,
            C::fetch(&mut state.2, row)?,
        ))
    }
}

unsafe impl<
        'w,
        A: QueryFetchMut<'w>,
        B: QueryFetchMut<'w>,
        C: QueryFetchMut<'w>,
        D: QueryFetchMut<'w>,
    > QueryFetchMut<'w> for (A, B, C, D)
where
    A: QueryFilter,
    B: QueryFilter,
    C: QueryFilter,
    D: QueryFilter,
{
    type Item = (A::Item, B::Item, C::Item, D::Item);
    type State = (A::State, B::State, C::State, D::State);

    fn prepare(archetype: &'w mut Archetype, current_tick: u32) -> Option<Self::State> {
        let ptr = archetype as *mut Archetype;
        let state_a = A::prepare(unsafe { &mut *ptr }, current_tick)?;
        let state_b = B::prepare(unsafe { &mut *ptr }, current_tick)?;
        let state_c = C::prepare(unsafe { &mut *ptr }, current_tick)?;
        let state_d = D::prepare(unsafe { &mut *ptr }, current_tick)?;
        Some((state_a, state_b, state_c, state_d))
    }

    unsafe fn fetch(state: &mut Self::State, row: usize) -> Option<Self::Item> {
        Some((
            A::fetch(&mut state.0, row)?,
            B::fetch(&mut state.1, row)?,
            C::fetch(&mut state.2, row)?,
            D::fetch(&mut state.3, row)?,
        ))
    }
}

// Manual implementations for tuple QueryFetch (immutable) to avoid macro complexity

unsafe impl<'w, A: QueryFetch<'w>, B: QueryFetch<'w>> QueryFetch<'w> for (A, B) {
    type Item = (A::Item, B::Item);
    type State = (A::State, B::State);

    fn prepare(archetype: &'w Archetype, change_tick: u32) -> Option<Self::State> {
        Some((
            A::prepare(archetype, change_tick)?,
            B::prepare(archetype, change_tick)?,
        ))
    }

    unsafe fn fetch(state: &Self::State, row: usize) -> Option<Self::Item> {
        Some((A::fetch(&state.0, row)?, B::fetch(&state.1, row)?))
    }
}

unsafe impl<'w, A: QueryFetch<'w>, B: QueryFetch<'w>, C: QueryFetch<'w>> QueryFetch<'w>
    for (A, B, C)
{
    type Item = (A::Item, B::Item, C::Item);
    type State = (A::State, B::State, C::State);

    fn prepare(archetype: &'w Archetype, change_tick: u32) -> Option<Self::State> {
        Some((
            A::prepare(archetype, change_tick)?,
            B::prepare(archetype, change_tick)?,
            C::prepare(archetype, change_tick)?,
        ))
    }

    unsafe fn fetch(state: &Self::State, row: usize) -> Option<Self::Item> {
        Some((
            A::fetch(&state.0, row)?,
            B::fetch(&state.1, row)?,
            C::fetch(&state.2, row)?,
        ))
    }
}

unsafe impl<'w, A: QueryFetch<'w>, B: QueryFetch<'w>, C: QueryFetch<'w>, D: QueryFetch<'w>>
    QueryFetch<'w> for (A, B, C, D)
{
    type Item = (A::Item, B::Item, C::Item, D::Item);
    type State = (A::State, B::State, C::State, D::State);

    fn prepare(archetype: &'w Archetype, change_tick: u32) -> Option<Self::State> {
        Some((
            A::prepare(archetype, change_tick)?,
            B::prepare(archetype, change_tick)?,
            C::prepare(archetype, change_tick)?,
            D::prepare(archetype, change_tick)?,
        ))
    }

    unsafe fn fetch(state: &Self::State, row: usize) -> Option<Self::Item> {
        Some((
            A::fetch(&state.0, row)?,
            B::fetch(&state.1, row)?,
            C::fetch(&state.2, row)?,
            D::fetch(&state.3, row)?,
        ))
    }
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
    pub fn iter<'w, 's>(&'s self, world: &'w World, change_tick: u32) -> QueryIter<'w, F>
    where
        F: QueryFetch<'w>,
    {
        QueryIter::new(world, &self.matched_archetypes, change_tick)
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
            state: None,
            _phantom: PhantomData,
        }
    }

    /// Count matching entities
    pub fn count(&self) -> usize {
        self.iter().len()
    }
}

/// Owned query iterator (holds its own state)
pub struct QueryIterOwned<'w, Q: QueryFilter>
where
    Q: QueryFetch<'w>,
{
    world: &'w World,
    matched_archetypes: Vec<usize>,
    archetype_index: usize,
    entity_index: usize,
    change_tick: u32,
    state: Option<Q::State>,
    _phantom: PhantomData<Q>,
}

impl<'w, Q> Iterator for QueryIterOwned<'w, Q>
where
    Q: QueryFilter + QueryFetch<'w>,
{
    type Item = <Q as QueryFetch<'w>>::Item;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.state.is_none() {
                if self.archetype_index >= self.matched_archetypes.len() {
                    return None;
                }

                let arch_id = self.matched_archetypes[self.archetype_index];
                let archetype = self.world.get_archetype(arch_id)?;

                self.state = Q::prepare(archetype, self.change_tick);
                self.entity_index = 0;

                if self.state.is_none() {
                    self.archetype_index += 1;
                    continue;
                }
            }

            let arch_id = self.matched_archetypes[self.archetype_index];
            let archetype = self.world.get_archetype(arch_id)?;

            if self.entity_index < archetype.len() {
                let row = self.entity_index;
                self.entity_index += 1;

                // SAFETY: We checked bounds above. State is valid for this archetype.
                if let Some(item) = unsafe { Q::fetch(self.state.as_ref().unwrap(), row) } {
                    return Some(item);
                } else {
                    continue;
                }
            } else {
                self.state = None;
                self.archetype_index += 1;
            }
        }
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

// QueryFetch implementations for filter types

unsafe impl<'w, T: 'static> QueryFetch<'w> for With<T> {
    type Item = ();
    type State = ();

    fn prepare(_archetype: &'w Archetype, _change_tick: u32) -> Option<Self::State> {
        Some(())
    }

    unsafe fn fetch(_state: &Self::State, _row: usize) -> Option<Self::Item> {
        Some(())
    }
}

unsafe impl<'w, T: 'static> QueryFetch<'w> for Without<T> {
    type Item = ();
    type State = ();

    fn prepare(_archetype: &'w Archetype, _change_tick: u32) -> Option<Self::State> {
        Some(())
    }

    unsafe fn fetch(_state: &Self::State, _row: usize) -> Option<Self::Item> {
        Some(())
    }
}

unsafe impl<'w, T: Component> QueryFetch<'w> for Changed<T> {
    type Item = ();
    type State = (&'w ComponentColumn, u32);

    fn prepare(archetype: &'w Archetype, change_tick: u32) -> Option<Self::State> {
        let type_id = TypeId::of::<T>();
        archetype
            .get_column(type_id)
            .map(|column| (column, change_tick))
    }

    unsafe fn fetch(state: &Self::State, row: usize) -> Option<Self::Item> {
        let (column, change_tick) = state;
        // Check if this specific row has changed since the given tick
        if row < column.changed_ticks.len() && column.changed_ticks[row] > *change_tick {
            Some(())
        } else {
            None
        }
    }
}

// With query filter
unsafe impl<'w, T: 'static> QueryFetch<'w> for Added<T> {
    type Item = ();
    type State = (&'w ComponentColumn, u32);

    fn prepare(archetype: &'w Archetype, change_tick: u32) -> Option<Self::State> {
        let type_id = TypeId::of::<T>();
        archetype
            .get_column(type_id)
            .map(|column| (column, change_tick))
    }

    unsafe fn fetch(state: &Self::State, row: usize) -> Option<Self::Item> {
        let (column, change_tick) = state;
        // SAFETY: `row` is assumed to be a valid index for the column.
        if column.get_added_tick(row)? > *change_tick {
            Some(())
        } else {
            None
        }
    }
}

unsafe impl<'w, T: Component> QueryFetch<'w> for Read<T> {
    type Item = &'w T;
    type State = &'w ComponentColumn;

    fn prepare(archetype: &'w Archetype, _change_tick: u32) -> Option<Self::State> {
        let type_id = TypeId::of::<T>();
        archetype.get_column(type_id)
    }

    unsafe fn fetch(state: &Self::State, row: usize) -> Option<Self::Item> {
        // SAFETY: `row` is assumed to be a valid index for the column.
        state.get::<T>(row)
    }
}

unsafe impl<'w, T: Component> QueryFetchMut<'w> for Write<T> {
    type Item = &'w mut T;
    type State = (*mut ComponentColumn, u32);

    fn prepare(archetype: &'w mut Archetype, current_tick: u32) -> Option<Self::State> {
        let type_id = TypeId::of::<T>();
        archetype
            .get_column_mut(type_id)
            .map(|column| (column as *mut ComponentColumn, current_tick))
    }

    unsafe fn fetch(state: &mut Self::State, row: usize) -> Option<Self::Item> {
        let (column_ptr, current_tick) = state;
        // SAFETY: The column pointer is valid for the lifetime 'w and points to a valid ComponentColumn.
        // The caller ensures that row is a valid index within the column.
        let column = &mut **column_ptr;
        column.set_changed_tick(row, *current_tick);
        column.get_mut::<T>(row)
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
    pub fn iter<'w>(&mut self, world: &'w World) -> QueryIter<'w, F>
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

/// Marker type for fetching EntityId in queries
///
/// Use this to access the entity ID during query iteration:
/// ```ignore
/// for (entity, health) in world.query_mut::<(Entity, &mut Health)>().iter() {
///     if health.is_dead() {
///         to_delete.push(entity);
///     }
/// }
/// ```
pub struct Entity;

impl QueryFilter for Entity {
    fn matches_archetype(_archetype: &Archetype) -> bool {
        true // Entity always matches - all archetypes have entities
    }

    fn type_ids() -> SmallVec<[TypeId; MAX_FILTER_COMPONENTS]> {
        smallvec![] // Entity doesn't require specific components
    }
}

unsafe impl<'w> QueryFetch<'w> for Entity {
    type Item = EntityId;
    type State = &'w [EntityId];

    fn prepare(archetype: &'w Archetype, _change_tick: u32) -> Option<Self::State> {
        Some(archetype.entities())
    }

    unsafe fn fetch(state: &Self::State, row: usize) -> Option<Self::Item> {
        state.get(row).copied()
    }
}

unsafe impl<'w> QueryFetchMut<'w> for Entity {
    type Item = EntityId;
    type State = *const [EntityId];

    fn prepare(archetype: &'w mut Archetype, _current_tick: u32) -> Option<Self::State> {
        Some(archetype.entities() as *const [EntityId])
    }

    unsafe fn fetch(state: &mut Self::State, row: usize) -> Option<Self::Item> {
        // SAFETY: The pointer is valid for the lifetime 'w
        let slice = unsafe { &**state };
        slice.get(row).copied()
    }
}

/// Query filter for components that changed since last system run
///
/// Usage: `Query<&Position, Changed<Position>>` - only entities where Position changed
pub struct Changed<T: Component>(PhantomData<T>);

impl<T: Component> QueryFilter for Changed<T> {
    fn matches_archetype(archetype: &Archetype) -> bool {
        // Check if archetype has this component type
        archetype.get_column(TypeId::of::<T>()).is_some()
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
