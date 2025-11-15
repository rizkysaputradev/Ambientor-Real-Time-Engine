//! Realtime synthesis graph core.
//!
//! This module defines the minimal `Generator` trait and a lightweight `Engine<G>`
//! wrapper that owns a generator (scene/voice), tracks sample rate and time, and
//! produces one **mono** sample at a time with zero heap work per sample.
//!
//! Design goals
//! - No dynamic allocations in the audio thread
//! - SR changes handled lazily (if the host reconfigures), with cheap branching
//! - Generic over the scene type, so scenes can be swapped without trait objects

/// Anything that can generate one sample at a time.
pub trait Generator {
    /// Called when the engine is (re)initialized or when the sample rate changes.
    fn reset(&mut self, sr: f32);

    /// Generate the next mono sample. Implementations should assume the sample
    /// rate has been communicated via `reset`.
    fn next(&mut self) -> f32;
}

/// Lightweight realtime engine that owns a generator.
///
/// The audio callback should call `next(sr)` for every output sample. If the
/// `sr` reported by the host changes, the engine will call `reset(sr)` on the
/// inner generator once and continue.
pub struct Engine<G: Generator> {
    sr: f32,
    t: f32,
    gen: G,
}

impl<G: Generator> Engine<G> {
    /// Construct with an already-configured generator. We immediately `reset`
    /// the generator to communicate the sample rate.
    #[inline]
    pub fn new(mut gen: G) -> Self {
        // `sr` will be set by the first `next(sr)` call, but we can initialize to sane defaults.
        let sr = 48_000.0;
        gen.reset(sr);
        Self { sr, t: 0.0, gen }
    }

    /// Produce **one** mono sample at the given sample rate.
    ///
    /// - If `sr` differs from the current engine `sr`, we update and call `reset(sr)`.
    /// - We track `t` (seconds) incrementally, in case scenes want to expose it later.
    #[inline]
    pub fn next(&mut self, sr: f32) -> f32 {
        if sr != self.sr {
            self.sr = sr;
            self.gen.reset(sr);
        }
        // maintain a running time accumulator (not currently exposed)
        self.t += 1.0 / self.sr;
        self.gen.next()
    }

    /// Return the engine’s current sample rate.
    #[inline] pub fn sample_rate(&self) -> f32 { self.sr }

    /// Return elapsed time (seconds) since this engine was created.
    #[inline] pub fn time(&self) -> f32 { self.t }

    /// Replace the inner generator (scene) in a zero-allocation manner.
    /// We call `reset(sr)` on the new scene.
    #[inline]
    pub fn swap_scene(&mut self, mut new_scene: G) {
        new_scene.reset(self.sr);
        self.gen = new_scene;
    }

    /// Get a mutable reference to the inner generator for live parameter tweaks.
    #[inline]
    pub fn scene_mut(&mut self) -> &mut G { &mut self.gen }
}
