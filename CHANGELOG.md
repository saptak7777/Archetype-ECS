# Changelog

All notable changes to Archetype ECS will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.1.1] - 2024-12-04

### Added
- **Entity ID Access in Queries**: New `Entity` marker type allows getting entity IDs during query iteration
  ```rust
  for (entity, health) in world.query_mut::<(Entity, &Health)>() {
      if health.is_dead() {
          to_delete.push(entity);
      }
  }
  ```
- **IntoIterator for QueryMut**: Queries can now be used directly in for loops without calling `.iter()`
  ```rust
  for (pos, vel) in world.query_mut::<(&mut Position, &Velocity)>() { }
  ```
- **Mixed Mutability Tuple Queries**: Support for reading and writing different components in the same query
  ```rust
  for (pos, vel) in world.query_mut::<(&Position, &mut Velocity)>() { }
  ```
- **Simple Resource API**: Global singleton state management
  ```rust
  world.insert_resource(Time { delta: 0.016 });
  let time = world.resource::<Time>();
  ```
- Exported `Entity`, `Result`, `QueryMut`, `QueryState` in prelude for convenience

### Changed
- Updated README with clearer examples and better organization
- Improved documentation for all new features

### Performance
- Query iteration (cached): **10% faster** (10.1 µs, was 11.2 µs)
- All changes are zero-cost abstractions with no runtime overhead

## [1.1.0] - 2024-12-03

### Added
- SIMD & chunk-based iteration support
- Parallel query processing with `par_for_each_chunk`
- System ordering constraints (`add_system_before`, `add_system_after`)
- Change detection with `Changed<T>` and `Added<T>` filters
- Comprehensive benchmark suite

### Performance
- Significant improvements in spawn operations (-20.7%)
- Parallel execution improvements (-11.6%)
- Serialization improvements (-15.5%)

## [1.0.0] - 2024-11-15

### Added
- Initial release
- Archetype-based ECS implementation
- Parallel system execution
- Query system with type-safe component access
- Command buffer for deferred operations
- Event system
- Hierarchy support
- Serialization/deserialization
- Resource management
