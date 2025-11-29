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

//! Entity identifiers and location metadata.

use slotmap::new_key_type;

new_key_type! {
    /// Unique entity identifier backed by slotmap's generational keys.
    pub struct EntityId;
}

/// Entity location in archetype (archetype_id, row)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EntityLocation {
    pub archetype_id: usize,
    pub archetype_row: usize,
}
