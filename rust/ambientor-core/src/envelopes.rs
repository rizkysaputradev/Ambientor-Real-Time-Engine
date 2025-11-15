//! Envelope generators and parameter slewing primitives.
//!
//! Provided envelopes:
//! - `AdsrLinear`    : classic ADSR with **linear** segments
//! - `AdsrExp`       : ADSR with **exponential (RC-like)** segments (more “musical”)
//! - `ArExp`         : fast AR percussion envelope (exp attack/decay)
//! - `SlewLimiter`   : one-pole slew/smoother for arbitrary control signals
//!
//! All envelopes are `no_std` friendly and avoid heap allocations.
//! Each exposes a `next(dt)` or `next(sr)` style tick and simple gate control.

use core::fmt::Debug;
use crate::dsp::{one_pole_coeff_ms, clamp};

// -------------------------------- Linear ADSR ------------------------------------

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum AdsrStage {
    Idle,
    Attack,
    Decay,
    Sustain,
    Release,
}

/// Linear ADSR envelope.
/// Times are specified in milliseconds. Sustain is [0,1].
/// Call `set_sr(sample_rate)` once if your `next()` variant uses `sr`.
#[derive(Copy, Clone, Debug)]
pub struct AdsrLinear {
    atk_ms: f32,
    dec_ms: f32,
    sus:    f32,
    rel_ms: f32,
    sr:     f32,

    // state
    env:   f32,
    gate:  bool,
    stage: AdsrStage,
    // cached per-sample increments
    a_inc: f32,
    d_dec: f32,
    r_dec: f32,
}

impl AdsrLinear {
    #[inline]
    pub fn new(atk_ms: f32, dec_ms: f32, sus: f32, rel_ms: f32, sr: f32) -> Self {
        let mut s = Self {
            atk_ms,
            dec_ms,
            sus: clamp(sus, 0.0, 1.0),
            rel_ms,
            sr,
            env: 0.0,
            gate: false,
            stage: AdsrStage::Idle,
            a_inc: 0.0,
            d_dec: 0.0,
            r_dec: 0.0,
        };
        s.recalc_increments();
        s
    }

    #[inline]
    pub fn set_sr(&mut self, sr: f32) {
        self.sr = sr.max(1.0);
        self.recalc_increments();
    }

    #[inline]
    pub fn set_params(&mut self, atk_ms: f32, dec_ms: f32, sus: f32, rel_ms: f32) {
        self.atk_ms = atk_ms.max(0.0);
        self.dec_ms = dec_ms.max(0.0);
        self.sus    = clamp(sus, 0.0, 1.0);
        self.rel_ms = rel_ms.max(0.0);
        self.recalc_increments();
    }

    #[inline]
    fn recalc_increments(&mut self) {
        let sr = self.sr.max(1.0);
        self.a_inc = if self.atk_ms <= 0.0 {
            // instant attack
            1.0
        } else {
            1.0 / (self.atk_ms * 0.001 * sr)
        };
        self.d_dec = if self.dec_ms <= 0.0 {
            // instant decay
            1.0
        } else {
            (1.0 - self.sus) / (self.dec_ms * 0.001 * sr)
        };
        self.r_dec = if self.rel_ms <= 0.0 {
            // instant release
            1.0
        } else {
            // linear ramp from sustain to 0
            self.sus / (self.rel_ms * 0.001 * sr)
        };
    }

    #[inline]
    pub fn gate_on(&mut self)  {
        self.gate  = true;
        self.stage = AdsrStage::Attack;
    }

    #[inline]
    pub fn gate_off(&mut self) {
        self.gate  = false;
        self.stage = AdsrStage::Release;
    }

    /// Advance by **one sample** using the configured sample rate.
    #[inline]
    pub fn next(&mut self) -> f32 {
        match self.stage {
            AdsrStage::Idle => {
                self.env = 0.0;
            }
            AdsrStage::Attack => {
                self.env += self.a_inc;
                if self.env >= 1.0 {
                    self.env = 1.0;
                    self.stage = AdsrStage::Decay;
                }
            }
            AdsrStage::Decay => {
                if !self.gate {
                    // if gate dropped mid-decay, go straight to release
                    self.stage = AdsrStage::Release;
                } else if self.env > self.sus {
                    self.env -= self.d_dec;
                    if self.env <= self.sus {
                        self.env = self.sus;
                        self.stage = AdsrStage::Sustain;
                    }
                } else {
                    self.env = self.sus;
                    self.stage = AdsrStage::Sustain;
                }
            }
            AdsrStage::Sustain => {
                if !self.gate {
                    self.stage = AdsrStage::Release;
                } else {
                    self.env = self.sus;
                }
            }
            AdsrStage::Release => {
                if self.rel_ms <= 0.0 {
                    self.env = 0.0;
                    self.stage = AdsrStage::Idle;
                } else if self.env > 0.0 {
                    self.env -= self.r_dec;
                    if self.env <= 0.0 {
                        self.env = 0.0;
                        self.stage = AdsrStage::Idle;
                    }
                } else {
                    self.env = 0.0;
                    self.stage = AdsrStage::Idle;
                }
            }
        }
        self.env
    }

    #[inline] pub fn value(&self) -> f32 { self.env }
}

// ------------------------------- Exponential ADSR --------------------------------

/// Exponential (RC-like) ADSR envelope.
/// Attack/Decay/Release are **time constants in ms** controlling the curvature.
/// Sustain is [0,1]. This is often more “musical” than linear segments.
#[derive(Copy, Clone, Debug)]
pub struct AdsrExp {
    atk_ms: f32,
    dec_ms: f32,
    sus:    f32,
    rel_ms: f32,
    sr:     f32,

    env:  f32,
    gate: bool,
    // per-stage coefficients a = exp(-1/(tau*sr))
    a_a: f32,
    a_d: f32,
    a_r: f32,
}

impl AdsrExp {
    #[inline]
    pub fn new(atk_ms: f32, dec_ms: f32, sus: f32, rel_ms: f32, sr: f32) -> Self {
        let mut s = Self {
            atk_ms, dec_ms, sus: clamp(sus, 0.0, 1.0), rel_ms,
            sr,
            env: 0.0,
            gate: false,
            a_a: 0.0,
            a_d: 0.0,
            a_r: 0.0,
        };
        s.recalc_coeffs();
        s
    }

    #[inline]
    pub fn set_sr(&mut self, sr: f32) { self.sr = sr.max(1.0); self.recalc_coeffs(); }

    #[inline]
    pub fn set_params(&mut self, atk_ms: f32, dec_ms: f32, sus: f32, rel_ms: f32) {
        self.atk_ms = atk_ms.max(0.0);
        self.dec_ms = dec_ms.max(0.0);
        self.sus    = clamp(sus, 0.0, 1.0);
        self.rel_ms = rel_ms.max(0.0);
        self.recalc_coeffs();
    }

    #[inline]
    fn recalc_coeffs(&mut self) {
        let sr = self.sr;
        self.a_a = one_pole_coeff_ms(self.atk_ms, sr);
        self.a_d = one_pole_coeff_ms(self.dec_ms, sr);
        self.a_r = one_pole_coeff_ms(self.rel_ms, sr);
    }

    #[inline] pub fn gate_on(&mut self)  { self.gate = true; }
    #[inline] pub fn gate_off(&mut self) { self.gate = false; }

    /// Advance by one sample and return the envelope value.
    ///
    /// Stage equations (exponential towards target):
    /// - Attack:  env += (1 - env) * (1 - a_a)
    /// - Decay:   env += (sus - env) * (1 - a_d)
    /// - Release: env += (0   - env) * (1 - a_r)
    #[inline]
    pub fn next(&mut self) -> f32 {
        if self.gate {
            if self.env < 0.9999 {
                self.env += (1.0 - self.env) * (1.0 - self.a_a);
            } else if self.env > self.sus {
                self.env += (self.sus - self.env) * (1.0 - self.a_d);
            } else {
                self.env = self.sus; // hold
            }
        } else {
            self.env += (0.0 - self.env) * (1.0 - self.a_r);
            if self.env.abs() < 1e-6 { self.env = 0.0; }
        }
        self.env
    }

    #[inline] pub fn value(&self) -> f32 { self.env }
}

// ------------------------------- AR (percussive) ---------------------------------

/// Exponential AR envelope for percussive sounds.
/// Attack and release are ms time constants (RC style). Calling `trigger()` restarts from zero.
#[derive(Copy, Clone, Debug)]
pub struct ArExp {
    atk_ms: f32,
    rel_ms: f32,
    sr:     f32,
    env:    f32,
    rising: bool,
    a_a:    f32,
    a_r:    f32,
}

impl ArExp {
    #[inline]
    pub fn new(atk_ms: f32, rel_ms: f32, sr: f32) -> Self {
        let mut s = Self {
            atk_ms, rel_ms, sr,
            env: 0.0, rising: false,
            a_a: 0.0, a_r: 0.0,
        };
        s.recalc();
        s
    }

    #[inline] pub fn set_sr(&mut self, sr: f32) { self.sr = sr.max(1.0); self.recalc(); }

    #[inline]
    pub fn set_params(&mut self, atk_ms: f32, rel_ms: f32) {
        self.atk_ms = atk_ms.max(0.0);
        self.rel_ms = rel_ms.max(0.0);
        self.recalc();
    }

    #[inline] fn recalc(&mut self) {
        self.a_a = one_pole_coeff_ms(self.atk_ms, self.sr);
        self.a_r = one_pole_coeff_ms(self.rel_ms, self.sr);
    }

    /// Start from 0, go up, then decay.
    #[inline] pub fn trigger(&mut self) { self.env = 0.0; self.rising = true; }

    #[inline]
    pub fn next(&mut self) -> f32 {
        if self.rising {
            self.env += (1.0 - self.env) * (1.0 - self.a_a);
            if self.env >= 0.9999 { self.rising = false; }
        } else {
            self.env += (0.0 - self.env) * (1.0 - self.a_r);
            if self.env <= 1e-5 { self.env = 0.0; }
        }
        self.env
    }

    #[inline] pub fn value(&self) -> f32 { self.env }
}

// -------------------------------- Slew Limiter -----------------------------------

/// One-pole slew/smoother: `y += (x - y) * (1 - a)`
///
/// Use `alpha = one_pole_coeff_ms(t_ms, sr)`.
#[derive(Copy, Clone, Debug)]
pub struct SlewLimiter {
    alpha: f32,
    y:     f32,
}

impl SlewLimiter {
    #[inline]
    pub fn new(t_ms: f32, sr: f32) -> Self {
        Self { alpha: one_pole_coeff_ms(t_ms, sr), y: 0.0 }
    }

    #[inline]
    pub fn set_time_ms(&mut self, t_ms: f32, sr: f32) {
        self.alpha = one_pole_coeff_ms(t_ms, sr);
    }

    #[inline]
    pub fn reset(&mut self, y0: f32) { self.y = y0; }

    #[inline]
    pub fn process(&mut self, x: f32) -> f32 {
        self.y += (x - self.y) * (1.0 - self.alpha);
        self.y
    }

    #[inline]
    pub fn value(&self) -> f32 { self.y }
}

// ------------------------------------ Tests --------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adsr_linear_reaches_sustain() {
        let sr = 48000.0;
        let mut env = AdsrLinear::new(10.0, 50.0, 0.5, 200.0, sr);
        env.gate_on();
        // Run ~0.5s
        for _ in 0..(sr as usize / 2) { env.next(); }
        let v = env.value();
        assert!(v > 0.45 && v < 0.55, "v={v}");
        env.gate_off();
        for _ in 0..(sr as usize) { env.next(); }
        assert!(env.value() < 0.01);
    }

    #[test]
    fn adsr_exp_behaves() {
        let sr = 48000.0;
        let mut env = AdsrExp::new(5.0, 100.0, 0.3, 200.0, sr);
        env.gate_on();
        for _ in 0..(sr as usize / 4) { env.next(); }
        assert!(env.value() <= 1.0 + 1e-3);
        env.gate_off();
        for _ in 0..(sr as usize / 2) { env.next(); }
        assert!(env.value() < 0.05);
    }

    #[test]
    fn ar_exp_triggers_and_dies() {
        let sr = 48000.0;
        let mut e = ArExp::new(1.0, 200.0, sr);
        e.trigger();
        let mut maxv = 0.0;
        for _ in 0..(sr as usize) {
            let v = e.next();
            if v > maxv { maxv = v; }
        }
        assert!(maxv > 0.8 && e.value() < 0.01);
    }

    #[test]
    fn slew_moves_towards_target() {
        let sr = 48000.0;
        let mut s = SlewLimiter::new(50.0, sr);
        for _ in 0..(sr as usize) { s.process(1.0); }
        assert!(s.value() > 0.9);
    }
}