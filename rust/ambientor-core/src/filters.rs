//! Filters: lightweight one-poles and a TPT (state-variable) filter.
//!
//! Goals
//! - `no_std`-friendly, allocation free
//! - Stable, musically-pleasant responses
//! - Clear APIs and predictable parameterization
//!
//! Contents
//! - `OnePoleLP`  : “RC-style” one-pole low-pass (cheap smoother/tilt)
//! - `OnePoleHP`  : “RC-style” one-pole high-pass (DC blocker-ish)
//! - `DcBlock`    : convenience wrapper specialized for DC removal
//! - `SvfMode`    : LP/HP/BP/Notch modes for the SVF
//! - `SvfTpt`     : State-Variable Filter via Topology Preserving Transform
//!
//! Notes
//! - `OnePole*` use the inexpensive `y += a * (x - y)` form, where
//!   `a = 1 - exp(-2π fc / sr)`. These are not bilinear/TPT matched;
//!   they’re great for parameter smoothing and gentle tonal shaping.
//! - `SvfTpt` uses the “g = tan(π fc / sr)” formulation with `R = 1/(2Q)`.
//!   It is robust to high resonance and parameter modulation.

use crate::dsp::{kill_denormals, one_pole_coeff_hz, tpt_g};
use core::fmt::Debug;

/// One-pole low-pass `y += a * (x - y)`.
///
/// `a` is derived from cutoff (Hz) and sample rate:
/// `a = 1 - exp(-2π * fc / sr)`.
#[derive(Copy, Clone, Debug)]
pub struct OnePoleLP {
    a: f32,
    y: f32,
    sr: f32,
    fc: f32,
}

impl OnePoleLP {
    /// Create a low-pass with cutoff `cut_hz` and sample rate `sr`.
    #[inline]
    pub fn new(cut_hz: f32, sr: f32) -> Self {
        let mut s = Self {
            a: 0.0,
            y: 0.0,
            sr: sr.max(1.0),
            fc: cut_hz.max(0.0),
        };
        s.update_coeffs();
        s
    }

    #[inline]
    pub fn set_sample_rate(&mut self, sr: f32) {
        self.sr = sr.max(1.0);
        self.update_coeffs();
    }

    #[inline]
    pub fn set_cutoff_hz(&mut self, cut_hz: f32) {
        self.fc = cut_hz.max(0.0);
        self.update_coeffs();
    }

    #[inline]
    fn update_coeffs(&mut self) {
        // For the “y += a*(x-y)” form, many references set a = 1 - exp(..).
        // We compute `exp(-..)` once and fold to a.
        let exp_term = one_pole_coeff_hz(self.fc, self.sr); // = exp(-2π fc / sr)
        self.a = 1.0 - exp_term;
    }

    /// Process one sample.
    #[inline]
    pub fn process(&mut self, x: f32) -> f32 {
        self.y += self.a * (x - self.y);
        kill_denormals(self.y)
    }

    #[inline] pub fn value(&self) -> f32 { self.y }
}

/// One-pole high-pass using the standard “leaky integrator” form:
///
/// Difference equation:
/// `y[n] = x[n] - x[n-1] + b * y[n-1]`, with `b = exp(-2π fc / sr)`.
#[derive(Copy, Clone, Debug)]
pub struct OnePoleHP {
    b: f32,
    x1: f32,
    y1: f32,
    sr: f32,
    fc: f32,
}

impl OnePoleHP {
    #[inline]
    pub fn new(cut_hz: f32, sr: f32) -> Self {
        let mut s = Self {
            b: 0.0,
            x1: 0.0,
            y1: 0.0,
            sr: sr.max(1.0),
            fc: cut_hz.max(0.0),
        };
        s.update_coeffs();
        s
    }

    #[inline] pub fn set_sample_rate(&mut self, sr: f32) { self.sr = sr.max(1.0); self.update_coeffs(); }
    #[inline] pub fn set_cutoff_hz(&mut self, cut_hz: f32) { self.fc = cut_hz.max(0.0); self.update_coeffs(); }

    #[inline]
    fn update_coeffs(&mut self) {
        // HP leaky integrator uses the exponential directly.
        self.b = one_pole_coeff_hz(self.fc, self.sr); // exp(-2π fc / sr)
    }

    #[inline]
    pub fn process(&mut self, x: f32) -> f32 {
        let y = x - self.x1 + self.b * self.y1;
        self.x1 = x;
        self.y1 = y;
        kill_denormals(y)
    }

    #[inline] pub fn value(&self) -> f32 { self.y1 }
}

/// Convenience DC blocker: a high-pass with a very low cutoff (e.g., 5–30 Hz).
#[derive(Copy, Clone, Debug)]
pub struct DcBlock {
    hp: OnePoleHP,
}

impl DcBlock {
    /// `cut_hz` default recommendation: 20 Hz.
    #[inline]
    pub fn new(cut_hz: f32, sr: f32) -> Self {
        Self { hp: OnePoleHP::new(cut_hz, sr) }
    }

    #[inline] pub fn set_sample_rate(&mut self, sr: f32) { self.hp.set_sample_rate(sr); }
    #[inline] pub fn set_cutoff_hz(&mut self, hz: f32) { self.hp.set_cutoff_hz(hz); }

    #[inline] pub fn process(&mut self, x: f32) -> f32 { self.hp.process(x) }
    #[inline] pub fn value(&self) -> f32 { self.hp.value() }
}

/// SVF output tap selection.
#[derive(Copy, Clone, Debug)]
pub enum SvfMode {
    Lowpass,
    Highpass,
    Bandpass,
    Notch,
}

/// Topology-Preserving Transform SVF (State-Variable Filter).
///
/// Parameters:
/// - `cut_hz`  : cutoff / center frequency in Hz
/// - `q`       : quality factor (>= ~0.5 typical; lower increases damping)
///
/// Internals:
/// - `g = tan(π fc / sr)`
/// - `R = 1 / (2Q)`
///
/// This implementation follows common SVF/TPT references (Vadim Zavalishin et al.).
#[derive(Copy, Clone, Debug)]
pub struct SvfTpt {
    sr: f32,
    cut: f32,
    q: f32,
    // derived
    g: f32,
    r: f32,
    // states
    ic1eq: f32,
    ic2eq: f32,
}

impl SvfTpt {
    #[inline]
    pub fn new(cut_hz: f32, q: f32, sr: f32) -> Self {
        let mut s = Self {
            sr: sr.max(1.0),
            cut: cut_hz.max(0.0),
            q: q.max(1e-4),
            g: 0.0,
            r: 0.0,
            ic1eq: 0.0,
            ic2eq: 0.0,
        };
        s.recalc();
        s
    }

    #[inline] pub fn set_sample_rate(&mut self, sr: f32) { self.sr = sr.max(1.0); self.recalc(); }
    #[inline] pub fn set_cutoff_hz(&mut self, cut_hz: f32) { self.cut = cut_hz.max(0.0); self.recalc(); }
    #[inline] pub fn set_q(&mut self, q: f32) { self.q = q.max(1e-4); self.recalc(); }

    #[inline]
    fn recalc(&mut self) {
        self.g = tpt_g(self.cut, self.sr);       // tan(π fc / sr)
        self.r = 1.0 / (2.0 * self.q);           // damping
    }

    /// Process one sample, returning the selected mode output.
    ///
    /// Also returns the four taps `(lp, bp, hp, notch)` in a tuple if you need all.
    #[inline]
    pub fn process_all(&mut self, x: f32) -> (f32, f32, f32, f32) {
        // TPT SVF (Zavalishin):
        // v0 = x - r * ic1eq - ic2eq
        // v1 = g * v0 + ic1eq
        // v2 = g * v1 + ic2eq
        // ic1eq' = g * v0 + v1
        // ic2eq' = g * v1 + v2
        let v0 = x - self.r * self.ic1eq - self.ic2eq;
        let v1 = self.g * v0 + self.ic1eq;
        let v2 = self.g * v1 + self.ic2eq;

        // Update states (leaky integrators)
        self.ic1eq = self.g * v0 + v1;
        self.ic2eq = self.g * v1 + v2;

        // taps
        let lp = v2;
        let bp = v1;
        let hp = v0 - self.r * v1 - v2;   // algebraic combination
        let notch = hp + lp;

        (lp, bp, hp, notch)
    }

    /// Process one sample, returning only the mode requested.
    #[inline]
    pub fn process(&mut self, x: f32, mode: SvfMode) -> f32 {
        let (lp, bp, hp, n) = self.process_all(x);
        match mode {
            SvfMode::Lowpass => lp,
            SvfMode::Highpass => hp,
            SvfMode::Bandpass => bp,
            SvfMode::Notch => n,
        }
    }

    /// Convenience helpers per mode
    #[inline] pub fn process_lp(&mut self, x: f32) -> f32 { self.process(x, SvfMode::Lowpass) }
    #[inline] pub fn process_hp(&mut self, x: f32) -> f32 { self.process(x, SvfMode::Highpass) }
    #[inline] pub fn process_bp(&mut self, x: f32) -> f32 { self.process(x, SvfMode::Bandpass) }
    #[inline] pub fn process_notch(&mut self, x: f32) -> f32 { self.process(x, SvfMode::Notch) }
}

// ------------------------------------ Tests --------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_pole_lp_moves_towards_input() {
        let sr = 48000.0;
        let mut lp = OnePoleLP::new(1000.0, sr);
        let mut y = 0.0;
        for _ in 0..(sr as usize) {
            y = lp.process(1.0);
        }
        assert!(y > 0.9, "y={}", y);
    }

    #[test]
    fn one_pole_hp_blocks_dc() {
        let sr = 48000.0;
        let mut hp = OnePoleHP::new(20.0, sr);
        let mut y = 0.0;
        for _ in 0..(sr as usize) {
            y = hp.process(1.0);
        }
        assert!(y.abs() < 1e-2, "y={}", y);
    }

    #[test]
    fn svf_lp_is_sane() {
        let sr = 48000.0;
        let mut svf = SvfTpt::new(1000.0, 0.707, sr);
        // Feed a step; LP should approach a bounded value (with some ringing possible).
        let mut acc = 0.0;
        for _ in 0..(sr as usize) {
            acc = svf.process_lp(1.0);
        }
        assert!(acc <= 2.0, "svf runaway? {}", acc);
    }
}
