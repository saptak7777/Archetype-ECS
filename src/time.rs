//! Time management and fixed timestep support.
//!
//! This module provides:
//! - [`Time`] - Frame timing and delta time tracking
//! - [`FixedTime`] - Fixed timestep for deterministic updates
//!
//! # Examples
//!
//! ```
//! use archetype_ecs::time::{Time, FixedTime};
//!
//! let mut time = Time::new();
//! let mut fixed = FixedTime::new(60); // 60 Hz
//!
//! // In your game loop:
//! time.update();
//! for _ in 0..fixed.tick(time.delta()) {
//!     // Run physics at fixed 60 Hz
//! }
//! ```

use std::time::Duration;

/// Time resource for tracking frame timing
#[derive(Clone, Debug)]
pub struct Time {
    /// Time since last frame
    delta: Duration,
    /// Total elapsed time since start
    elapsed: Duration,
    /// Frame counter
    frame_count: u64,
    /// Time scale multiplier (1.0 = normal speed)
    time_scale: f32,
    /// Time at start of current frame
    startup_time: std::time::Instant,
    /// Time of last frame
    last_update: std::time::Instant,
}

impl Time {
    /// Create new Time resource
    pub fn new() -> Self {
        let now = std::time::Instant::now();
        Self {
            delta: Duration::ZERO,
            elapsed: Duration::ZERO,
            frame_count: 0,
            time_scale: 1.0,
            startup_time: now,
            last_update: now,
        }
    }

    /// Update time (call once per frame)
    pub fn update(&mut self) {
        let now = std::time::Instant::now();
        self.delta = now.duration_since(self.last_update);
        self.elapsed = now.duration_since(self.startup_time);
        self.last_update = now;
        self.frame_count += 1;
    }

    /// Get delta time (time since last frame)
    pub fn delta(&self) -> Duration {
        self.delta
    }

    /// Get scaled delta time
    pub fn delta_seconds(&self) -> f32 {
        self.delta.as_secs_f32() * self.time_scale
    }

    /// Get total elapsed time
    pub fn elapsed(&self) -> Duration {
        self.elapsed
    }

    /// Get elapsed time in seconds
    pub fn elapsed_seconds(&self) -> f32 {
        self.elapsed.as_secs_f32()
    }

    /// Get current frame count
    pub fn frame_count(&self) -> u64 {
        self.frame_count
    }

    /// Set time scale (1.0 = normal, 0.5 = half speed, 2.0 = double speed)
    pub fn set_time_scale(&mut self, scale: f32) {
        self.time_scale = scale.max(0.0);
    }

    /// Get time scale
    pub fn time_scale(&self) -> f32 {
        self.time_scale
    }

    /// Pause time (set scale to 0)
    pub fn pause(&mut self) {
        self.time_scale = 0.0;
    }

    /// Resume time (set scale to 1)
    pub fn resume(&mut self) {
        self.time_scale = 1.0;
    }

    /// Check if time is paused
    pub fn is_paused(&self) -> bool {
        self.time_scale == 0.0
    }
}

impl Default for Time {
    fn default() -> Self {
        Self::new()
    }
}

/// Fixed timestep for deterministic updates
#[derive(Clone, Debug)]
pub struct FixedTime {
    /// Fixed timestep duration
    timestep: Duration,
    /// Accumulated time from variable frame rate
    accumulator: Duration,
    /// Overstep from last frame (for interpolation)
    overstep: Duration,
}

impl FixedTime {
    /// Create new FixedTime with given frequency (Hz)
    pub fn new(hz: u32) -> Self {
        let timestep = Duration::from_secs_f32(1.0 / hz as f32);
        Self {
            timestep,
            accumulator: Duration::ZERO,
            overstep: Duration::ZERO,
        }
    }

    /// Create with explicit timestep duration
    pub fn from_duration(timestep: Duration) -> Self {
        Self {
            timestep,
            accumulator: Duration::ZERO,
            overstep: Duration::ZERO,
        }
    }

    /// Update accumulator and return number of fixed steps to run
    pub fn tick(&mut self, delta: Duration) -> usize {
        self.accumulator += delta;

        let mut steps = 0;
        while self.accumulator >= self.timestep {
            self.accumulator -= self.timestep;
            steps += 1;
        }

        self.overstep = self.accumulator;
        steps
    }

    /// Get fixed timestep duration
    pub fn timestep(&self) -> Duration {
        self.timestep
    }

    /// Get timestep in seconds
    pub fn timestep_seconds(&self) -> f32 {
        self.timestep.as_secs_f32()
    }

    /// Get overstep (for interpolation)
    pub fn overstep(&self) -> Duration {
        self.overstep
    }

    /// Get overstep as fraction of timestep (0.0 to 1.0)
    pub fn overstep_fraction(&self) -> f32 {
        if self.timestep.as_secs_f32() > 0.0 {
            self.overstep.as_secs_f32() / self.timestep.as_secs_f32()
        } else {
            0.0
        }
    }
}

impl Default for FixedTime {
    fn default() -> Self {
        Self::new(60) // 60 Hz default
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_creation() {
        let time = Time::new();
        assert_eq!(time.frame_count(), 0);
        assert_eq!(time.time_scale(), 1.0);
    }

    #[test]
    fn test_time_pause() {
        let mut time = Time::new();
        time.pause();
        assert!(time.is_paused());
        time.resume();
        assert!(!time.is_paused());
    }

    #[test]
    fn test_fixed_time_60hz() {
        let mut fixed = FixedTime::new(60);

        // 16.67ms frame (60 FPS)
        let steps = fixed.tick(Duration::from_millis(16));
        assert_eq!(steps, 0); // Not quite a full step yet

        // Another frame
        let steps = fixed.tick(Duration::from_millis(17));
        assert_eq!(steps, 1); // Now we have enough
    }

    #[test]
    fn test_fixed_time_slow_frame() {
        let mut fixed = FixedTime::new(60);

        // 33ms frame (30 FPS) - should run 2 fixed steps
        let steps = fixed.tick(Duration::from_millis(33));
        assert_eq!(steps, 1); // First step
    }

    #[test]
    fn test_overstep_fraction() {
        let mut fixed = FixedTime::new(60);
        fixed.tick(Duration::from_millis(8));

        let fraction = fixed.overstep_fraction();
        assert!(fraction > 0.0 && fraction < 1.0);
    }
}
