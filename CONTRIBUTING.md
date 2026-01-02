# Contributing to Archetype ECS

## API Naming Conventions

This document outlines the naming conventions used throughout the Archetype ECS codebase to ensure consistency and predictability.

### Naming Pattern Rules

**Rule:** `{verb}_{noun}_{mutation}`

- **verb:** `spawn`, `create`, `add`, `remove`, `get`, `iter`, `query`, `execute`, `has`, `is`
- **noun:** `entity`, `component`, `system`, `plugin`, `archetype`, `resource`
- **mutation:** 
  - none = immutable operation
  - `mut` = mutable operation
  - `batch` = multiple items operation

### Pattern Categories

#### Pattern 1: Creation/Modification
```rust
// ✅ Consistent pattern
spawn_entity()           // Create single entity
spawn_batch()           // Create multiple entities
add_component()         // Add component to entity
remove_component()      // Remove component from entity
insert_resource()       // Add resource to world
clear()                  // Clear all entities
```

#### Pattern 2: Access
```rust
// ✅ Consistent pattern
get_component()         // Get component (immutable)
get_component_mut()     // Get component (mutable)
has_component()         // Check if component exists
get_entity_location()   // Get entity location
get_archetype()          // Get archetype (immutable)
get_archetype_mut()      // Get archetype (mutable)
resource()               // Get resource (immutable)
resource_mut()           // Get resource (mutable)
```

#### Pattern 3: Query/Iteration
```rust
// ✅ Consistent pattern
query()                  // Create immutable query
query_mut()              // Create mutable query
iter()                    // Iterate over items
iter_mut()               // Iterate over items mutably
iter_since()             // Iterate since specific tick
par_for_each()           // Parallel iteration
```

#### Pattern 4: State/Status
```rust
// ✅ Consistent pattern
is_alive()               // Check if entity exists
entity_exists()          // Check if entity exists
entity_count()           // Get number of entities
archetype_count()        // Get number of archetypes
memory_stats()           // Get memory usage statistics
```

### Examples of Good Naming

```rust
// ✅ Clear and consistent
world.spawn_entity((Position { x: 0.0, y: 0.0 }, Velocity { x: 1.0, y: 0.0 }));
world.add_component(entity, Health { current: 100, max: 100 });
world.get_component::<Position>(entity);
world.query_mut::<(&mut Position, &Velocity)>();

// ✅ Batch operations
world.spawn_batch(bundles);
world.flush_removals();

// ✅ Status checks
world.is_alive(entity);
world.entity_count();
```

### Migration Guidelines

When adding new methods, follow these patterns:

1. **Use verbs for actions:** `spawn`, `get`, `add`, `remove`, `query`, `iter`
2. **Specify the noun:** `entity`, `component`, `resource`, `archetype`
3. **Add mutation suffix:** `mut` for mutable, `batch` for multiple
4. **Use `is_` or `has_` for boolean checks**
5. **Use `get_` for access operations**

### Deprecated Patterns

These patterns should be avoided in new code:

```rust
// ❌ Inconsistent naming
allocate_row()           // Should be allocate_entity_row()
get_entity_location()    // ✅ This is correct
has_component()          // ✅ This is correct
iter()                    // ✅ This is correct
```

### Implementation Checklist

When implementing new methods:

- [ ] Follow the naming pattern: `{verb}_{noun}_{mutation}`
- [ ] Add comprehensive documentation
- [ ] Include examples in docstrings
- [ ] Add tests for the new functionality
- [ ] Update existing code that uses old patterns
- [ ] Add deprecation warnings for old methods when needed

### Code Review Guidelines

When reviewing code for naming consistency:

1. **Check method names** follow the established patterns
2. **Verify parameter names** are descriptive and consistent
3. **Ensure documentation** matches the naming convention
4. **Look for opportunities** to standardize existing inconsistent names

### Examples in the Codebase

The following files demonstrate these patterns:

- `src/world.rs` - World operations and entity management
- `src/query.rs` - Query operations and iteration
- `src/archetype.rs` - Archetype management
- `src/component.rs` - Component operations

By following these conventions, we maintain a consistent, predictable, and professional API that is easy for users to learn and use effectively.
