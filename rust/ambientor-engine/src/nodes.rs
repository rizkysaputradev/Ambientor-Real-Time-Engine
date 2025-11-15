//! Building blocks (nodes) for Ambientor scenes.
//!
//! These are zero-allocation, per-sample components designed for realtime use.
//! Everything here is `Copy` or small and cheap to move; no locks, no heap.
//!
//! Contents:
//! - `Wave`, `Osc`     : basic oscillators (Sine/Tri/Saw) with stable phase wrap
//! - `Lfo`             : low-frequency oscillator (same core as `Osc`), for modulation
//! - `NoiseMod`        : ultra-low-rate random modulator with slewed steps
//! - `OnePoleSmoother` : parameter smoothing
//! - `Mix2`            : lightweight stereo/mono mixer helpers
//! - `PanLaw`          : constant-power panning helper
//!
//! Notes:
//! - Frequency is **Hz**; methods expect the current **sample rate** when stepping.
//! - These nodes are deliberately simple—higher-level scenes wire them together.

use ambientor_core::dsp::{TAU};
use ambientor_core::filters::{OnePoleLP};
use core::fmt::Debug;

/// Oscillator waveform.
#[derive(Copy, Clone, Debug)]
pub enum Wave { Sine, Tri, Saw }

/// Simple bandlimited-ish triangle (cheap) and naive saw (good enough for ambient).
#[inline]
fn osc_sample(phase01: f32, wave: Wave) -> f32 {
    match wave {
        Wave::Sine => (TAU * phase01).sin(), // we can swap to dsp::fast_sin if `fast-math` globally
        Wave::Tri  => 4.0 * (phase01 - 0.5).abs() - 1.0,
        Wave::Saw  => 2.0 * phase01 - 1.0,
    }
}

/// Free-running oscillator. Not poly/anti-aliased; fine for ambient drones and LFO duties.
#[derive(Copy, Clone, Debug)]
pub struct Osc {
    phase: f32,   // [0,1)
    freq:  f32,   // Hz
    wave:  Wave,
    gain:  f32,   // output gain (0..1)
}

impl Osc {
    #[inline] pub fn new(freq_hz: f32, wave: Wave) -> Self { Self { phase: 0.0, freq: freq_hz, wave, gain: 1.0 } }
    #[inline] pub fn set_freq(&mut self, hz: f32) { self.freq = hz.max(0.0); }
    #[inline] pub fn set_gain(&mut self, g: f32) { self.gain = g.max(0.0); }
    #[inline] pub fn set_wave(&mut self, w: Wave) { self.wave = w; }

    /// Advance one sample and return the oscillator sample.
    #[inline]
    pub fn next(&mut self, sr: f32) -> f32 {
        self.phase = (self.phase + self.freq / sr) % 1.0;
        let s = osc_sample(self.phase, self.wave);
        s * self.gain
    }

    /// Hard-set phase in [0,1).
    #[inline] pub fn set_phase01(&mut self, p: f32) { self.phase = if p >= 1.0 { p - (p as i32 as f32) } else if p < 0.0 { 0.0 } else { p }; }
}

/// Low-frequency oscillator; identical to `Osc` but with convenience constructor.
#[derive(Copy, Clone, Debug)]
pub struct Lfo(Osc);
impl Lfo {
    #[inline] pub fn sine(rate_hz: f32) -> Self { Self(Osc::new(rate_hz, Wave::Sine)) }
    #[inline] pub fn tri(rate_hz: f32)  -> Self { Self(Osc::new(rate_hz, Wave::Tri))  }
    #[inline] pub fn saw(rate_hz: f32)  -> Self { Self(Osc::new(rate_hz, Wave::Saw))  }

    /// Next LFO value in **[-1,1]**.
    #[inline] pub fn next_norm(&mut self, sr: f32) -> f32 { self.0.next(sr) }

    /// Next LFO value remapped to **[0,1]**.
    #[inline] pub fn next01(&mut self, sr: f32) -> f32 { 0.5 * (self.0.next(sr) + 1.0) }

    #[inline] pub fn set_rate(&mut self, hz: f32) { self.0.set_freq(hz); }
    #[inline] pub fn set_phase01(&mut self, p: f32) { self.0.set_phase01(p); }
}

/// Slowly changing random modulator (great for ambient drift).
///
/// Every `period_s` seconds we choose a new random target in [low, high] and
/// slew towards it with a simple one-pole low-pass.
#[derive(Copy, Clone, Debug)]
pub struct NoiseMod {
    low: f32,
    high: f32,
    period_s: f32,
    t: f32,           // seconds since last target pick
    target: f32,      // current target
    lp: OnePoleLP,    // slewer over targets
}

impl NoiseMod {
    /// `period_s`: how often to pick a new target (e.g., 3–20 seconds for very slow drift)
    /// `cut_hz`  : smoothing/slew cutoff for the interpolator (smaller = slower)
    #[inline]
    pub fn new(low: f32, high: f32, period_s: f32, cut_hz: f32, sr: f32) -> Self {
        let mut s = Self {
            low, high, period_s: period_s.max(0.1),
            t: 0.0,
            target: 0.0,
            lp: OnePoleLP::new(cut_hz.max(0.01), sr),
        };
        s.pick_target();
        s
    }

    #[inline] pub fn reset_sr(&mut self, sr: f32) { self.lp.set_sample_rate(sr); }

    #[inline]
    fn pick_target(&mut self) {
        // Simple LCG-ish RNG without pulling `rand` to the audio thread
        let u = pseudo_rand01(self.t);
        self.target = self.low + (self.high - self.low) * u;
        self.t = 0.0;
    }

    /// Next value, updated once per sample. Returns a smoothed value in [low, high].
    #[inline]
    pub fn next(&mut self, sr: f32) -> f32 {
        self.t += 1.0 / sr;
        if self.t >= self.period_s {
            self.pick_target();
        }
        self.lp.process(self.target)
    }
}

/// Deterministic pseudo-random function mapping (time) → [0,1]. Cheap and stable.
#[inline]
fn pseudo_rand01(x: f32) -> f32 {
    // hash-like float noise; totally sufficient for slow drift targets
    let n = (x * 12_345.6789).sin() * 43758.5453;
    (n.fract() + 1.0).fract()
}

/// One-pole parameter smoother: y += (x - y) * (1 - a), with `a = exp(-1/(tau*sr))`.
#[derive(Copy, Clone, Debug)]
pub struct OnePoleSmoother {
    a: f32, // alpha (closer to 1 → slower)
    y: f32,
}
impl OnePoleSmoother {
    #[inline] pub fn new_ms(t_ms: f32, sr: f32) -> Self { Self { a: ambientor_core::dsp::one_pole_coeff_ms(t_ms, sr), y: 0.0 } }
    #[inline] pub fn reset(&mut self, y0: f32) { self.y = y0; }
    #[inline] pub fn set_time_ms(&mut self, t_ms: f32, sr: f32) { self.a = ambientor_core::dsp::one_pole_coeff_ms(t_ms, sr); }
    #[inline] pub fn process(&mut self, x: f32) -> f32 { self.y += (x - self.y) * (1.0 - self.a); self.y }
    #[inline] pub fn value(&self) -> f32 { self.y }
}

/// Two-input mix utility with per-input gains (mono for now).
#[derive(Copy, Clone, Debug)]
pub struct Mix2 {
    g1: f32,
    g2: f32,
}
impl Mix2 {
    #[inline] pub fn new(g1: f32, g2: f32) -> Self { Self { g1, g2 } }
    #[inline] pub fn set(&mut self, g1: f32, g2: f32) { self.g1 = g1; self.g2 = g2; }
    #[inline] pub fn run(&self, a: f32, b: f32) -> f32 { a * self.g1 + b * self.g2 }
}

/// Constant-power panner helper.
#[derive(Copy, Clone, Debug)]
pub struct PanLaw;
impl PanLaw {
    /// Return (left, right) gains given `pan` in [-1..1], where -1 = hard left, +1 = hard right.
    #[inline]
    pub fn gains(pan: f32) -> (f32, f32) {
        // constant power using sine/cosine taper
        let p = (pan.clamp(-1.0, 1.0) + 1.0) * 0.25 * core::f32::consts::PI; // map to [0, π/2]
        (p.cos(), p.sin())
    }
}
