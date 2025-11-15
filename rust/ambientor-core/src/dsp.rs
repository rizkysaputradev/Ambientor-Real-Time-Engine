//! Generic DSP utilities and math helpers.
//!
//! Design goals:
//! - `no_std` ready (guarded by the crate feature `no-std`)
//! - Math backend selection that works in both `std` and `no_std` contexts
//! - Optional `fast-math` approximations for hot paths
//! - Clean, side-effect free helpers that are easy to test
//!
//! Features used by this file:
//! - `fast-math` : enables polynomial/rational approximations (faster, approx.)
//! - `simd`      : (hook points only here; actual SIMD in mixing modules)
//!
//! Conventions:
//! - All functions are `#[inline]` where useful to help the optimizer.
//! - Argument and return domains are documented per function.

#![allow(clippy::excessive_precision)]

use core::f32::consts::PI;

use cfg_if::cfg_if;

// ----------------------------- Math backend selection -----------------------------

cfg_if! {
    // micromath preferred if explicitly requested (works in no_std)
    if #[cfg(feature = "micromath")] {
        use micromath::F32Ext as _;
        #[inline] fn m_sin(x: f32) -> f32 { x.sin() }
        #[inline] fn m_cos(x: f32) -> f32 { x.cos() }
        #[inline] fn m_exp(x: f32) -> f32 { x.exp() }
        #[inline] fn m_tanh(x: f32) -> f32 { x.tanh() }
        #[inline] fn m_tan(x: f32) -> f32 { (x.sin()) / (x.cos()) }
    // libm (C math) in no_std
    } else if #[cfg(feature = "no-std")] {
        #[inline] fn m_sin(x: f32) -> f32 { libm::sinf(x) }
        #[inline] fn m_cos(x: f32) -> f32 { libm::cosf(x) }
        #[inline] fn m_exp(x: f32) -> f32 { libm::expf(x) }
        #[inline] fn m_tanh(x: f32) -> f32 { libm::tanhf(x) }
        #[inline] fn m_tan(x: f32) -> f32 { libm::tanf(x) }
    // std backend
    } else {
        #[inline] fn m_sin(x: f32) -> f32 { x.sin() }
        #[inline] fn m_cos(x: f32) -> f32 { x.cos() }
        #[inline] fn m_exp(x: f32) -> f32 { x.exp() }
        #[inline] fn m_tanh(x: f32) -> f32 { x.tanh() }
        #[inline] fn m_tan(x: f32) -> f32 { x.tan() }
    }
}

// --------------------------------- Constants -------------------------------------

/// 2π (commonly useful)
pub const TAU: f32 = 2.0 * PI;

/// A very small epsilon used in denormal handling and safe divisions.
pub const EPS_SMALL: f32 = 1.0e-20;

// --------------------------------- Utilities -------------------------------------

#[inline]
pub fn clamp(x: f32, lo: f32, hi: f32) -> f32 {
    if x < lo { lo } else if x > hi { x } else { x }
}

#[inline]
pub fn signum_nonzero(x: f32) -> f32 {
    if x >= 0.0 { 1.0 } else { -1.0 }
}

#[inline]
pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

#[inline]
pub fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = clamp((x - edge0) / (edge1 - edge0), 0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Wrap phase into [0, 1).
#[inline]
pub fn wrap_phase01(mut p: f32) -> f32 {
    // fast + branchless wrap using floor
    p = p - (p + 1.0).floor() + 1.0;
    if p >= 1.0 { p - 1.0 } else { p }
}

/// Kill denormal/subnormal values. Returns 0.0 if |x| < EPS_SMALL.
#[inline]
pub fn kill_denormals(x: f32) -> f32 {
    if x.abs() < EPS_SMALL { 0.0 } else { x }
}

// --------------------------------- dB / linear -----------------------------------

/// Convert dB to linear gain: lin = 10^(db/20).
#[inline]
pub fn db_to_lin(db: f32) -> f32 {
    if db <= -120.0 { 0.0 } else { m_exp(0.11512925464970229_f32 * db) } // ln(10)/20 ≈ 0.115129...
}

/// Convert linear gain to dB: db = 20*log10(lin).
#[inline]
pub fn lin_to_db(lin: f32) -> f32 {
    if lin <= EPS_SMALL { -120.0 }
    else { 8.685889638065036553_f32 * lin.ln() } // 20/ln(10)
}

// --------------------------------- Fast trig -------------------------------------

/// Fast sine with range reduction into [-π, π] and 5th-order minimax-style poly.
/// Max abs error ~1e-3 for musical uses when `fast-math` is enabled; falls back to exact otherwise.
#[inline]
pub fn fast_sin(x: f32) -> f32 {
    cfg_if! {
        if #[cfg(feature = "fast-math")] {
            // Range reduce to [-π, π] without making the parameter mutable in the signature.
            let mut xr = x;
            let k = (xr / TAU).round();
            xr -= k * TAU;

            // 5th-order odd polynomial: sin(x) ≈ x * (a + b x^2 + c x^4)
            let x2 = xr * xr;
            xr * (0.999_979_313_3 + x2 * (-0.166_624_432_0 + x2 * 0.008_308_978_98))
        } else {
            m_sin(x)
        }
    }
}

#[inline]
pub fn fast_cos(x: f32) -> f32 {
    cfg_if! {
        if #[cfg(feature = "fast-math")] {
            // cos(x) = sin(x + π/2)
            fast_sin(x + core::f32::consts::PI * 0.5)
        } else {
            m_cos(x)
        }
    }
}

// --------------------------------- Nonlinearities --------------------------------

/// Soft clip via tanh. If `fast-math` is enabled, uses a stable rational approximation.
///
/// Approximation used when `fast-math`:
/// `tanh(x) ≈ x * (27 + x^2) / (27 + 9 x^2)`
///
/// This is smooth, monotonic, and clamps towards ±1.
#[inline]
pub fn soft_clip(x: f32) -> f32 {
    #[cfg(feature = "fast-math")]
    {
        let x2 = x * x;
        let num = x * (27.0 + x2);
        let den = 27.0 + 9.0 * x2;
        return num / den;
    }
    m_tanh(x)
}

/// Drive + soft saturation helper: `tanh(drive * x)` (or fast approx).
#[inline]
pub fn saturate(x: f32, drive: f32) -> f32 {
    soft_clip(x * drive)
}

// --------------------------------- Exponentials / smoothing ----------------------

/// One-pole smoothing coefficient for a time constant `t_ms` (milliseconds).
///
/// The discrete one-pole form: `y[n] += a * (x[n] - y[n])`
/// where `a = exp(-1/(tau * sr))` for first-order lag with time constant `tau`.
///
/// We interpret `t_ms` as the time to reach ~63% (1 - 1/e). Common for parameter smoothing.
#[inline]
pub fn one_pole_coeff_ms(t_ms: f32, sr: f32) -> f32 {
    if t_ms <= 0.0 { return 1.0; }
    let tau = t_ms * 0.001;
    m_exp(-1.0 / (tau * sr))
}

/// Convert cutoff in Hz to a simple one-pole (non-TPT) coefficient.
/// Same form as `y += a * (x - y)`. This is not exactly a bilinear-matched filter;
/// it’s a lightweight “RC” style discretization.
#[inline]
pub fn one_pole_coeff_hz(cut_hz: f32, sr: f32) -> f32 {
    let fc = cut_hz.max(0.0).min(0.499 * sr);
    m_exp(-2.0 * PI * fc / sr)
}

/// TPT (Topology-Preserving Transform) `g = tan(π fc / sr)` helper for state-variable filters.
///
/// If `fast-math` is enabled and `tan` is expensive, we compute `tan(x)`
/// via `sin(x)/cos(x)` using our faster approximations, which is generally sufficient for musical ranges.
#[inline]
pub fn tpt_g(cut_hz: f32, sr: f32) -> f32 {
    let x = core::f32::consts::PI * (cut_hz / sr);
    cfg_if! {
        if #[cfg(feature = "fast-math")] {
            let s = fast_sin(x);
            let c = fast_cos(x);
            s / c
        } else {
            m_tan(x)
        }
    }
}

// --------------------------------- Simple meters ---------------------------------

/// Running RMS meter (windowed via exponential smoothing). Call once per sample.
///
/// `alpha` is the smoothing factor in [0,1]; a good choice is `alpha = one_pole_coeff_ms(50, sr)`.
#[derive(Copy, Clone, Debug)]
pub struct Rms {
    pub alpha: f32,
    state: f32,
}
impl Rms {
    #[inline]
    pub fn new(alpha: f32) -> Self { Self { alpha, state: 0.0 } }

    #[inline]
    pub fn reset(&mut self) { self.state = 0.0; }

    #[inline]
    pub fn tick(&mut self, x: f32) -> f32 {
        let x2 = x * x;
        self.state += self.alpha * (x2 - self.state);
        self.state.sqrt()
    }
}

// --------------------------------- Simple DC blocker ------------------------------

/// DC blocker (one-pole high-pass) with given coefficient `a` (close to 1.0).
/// Difference equation:
///   y[n] = x[n] - x[n-1] + a * y[n-1]
#[derive(Copy, Clone, Debug)]
pub struct DcBlock {
    a: f32,
    x1: f32,
    y1: f32,
}
impl DcBlock {
    #[inline]
    pub fn new(a: f32) -> Self { Self { a, x1: 0.0, y1: 0.0 } }

    #[inline]
    pub fn set_coeff(&mut self, a: f32) { self.a = a; }

    #[inline]
    pub fn process(&mut self, x: f32) -> f32 {
        let y = x - self.x1 + self.a * self.y1;
        self.x1 = x;
        self.y1 = y;
        y
    }
}

// --------------------------------- Mix / sine block (scalar) ---------------------

/// In-place mix: `dst[i] += src[i] * gain` (pure scalar, portable).
#[inline]
pub fn mix_in_place(dst: &mut [f32], src: &[f32], gain: f32) {
    if dst.len() != src.len() || dst.is_empty() {
        return;
    }
    for (d, s) in dst.iter_mut().zip(src.iter()) {
        *d += *s * gain;
    }
}

/// Fill `out` with a sine using a running phase accumulator.
/// After the call, `*phase` is advanced by `out.len() * phase_inc` and wrapped to [-π, π].
#[inline]
pub fn fill_sine(out: &mut [f32], phase: &mut f32, phase_inc: f32) {
    if out.is_empty() {
        return;
    }

    let two_pi = TAU;
    let inv_two_pi = 1.0 / two_pi;

    for y in out.iter_mut() {
        // range-reduce current phase to [-π, π]
        let mut xr = *phase;
        let k = (xr * inv_two_pi).round();
        xr -= k * two_pi;

        // 7th-order odd polynomial approximation:
        // sin(x) ≈ x + c3*x^3 + c5*x^5 + c7*x^7
        let x2 = xr * xr;
        let x3 = x2 * xr;
        let y_poly = xr
            + (-1.0 / 6.0) * x3
            + (1.0 / 120.0) * x3 * x2
            + (-1.0 / 5040.0) * x3 * x2 * x2;

        *y = y_poly;

        // advance phase; keep bounded occasionally
        *phase += phase_inc;
        if *phase > two_pi || *phase < -two_pi {
            let k2 = (*phase * inv_two_pi).round();
            *phase -= k2 * two_pi;
        }
    }
}

// --------------------------------- Tests (std only) ------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn db_lin_roundtrip() {
        for db in [-60.0, -20.0, -6.0, 0.0, 6.0, 12.0, 24.0] {
            let lin = db_to_lin(db);
            let back = lin_to_db(lin);
            assert!((db - back).abs() < 0.1, "db={}, back={}", db, back);
        }
    }

    #[test]
    fn soft_clip_is_bounded() {
        for x in [-10.0, -2.0, -1.0, 0.0, 1.0, 2.0, 10.0] {
            let y = soft_clip(x);
            assert!(y <= 1.0 + 1e-4 && y >= -1.0 - 1e-4, "x={} y={}", x, y);
        }
    }

    #[test]
    fn rms_decreases_to_zero() {
        let mut rms = Rms::new(one_pole_coeff_ms(10.0, 48000.0));
        let mut v = 0.0;
        for _ in 0..10000 {
            v = rms.tick(0.0);
        }
        assert!(v < 1e-3);
    }
}