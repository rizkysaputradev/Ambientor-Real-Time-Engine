#![cfg_attr(not(feature = "std"), no_std)]
//! Ambientor Core — no_std-ready DSP primitives with optional fast-math and SIMD hooks.
//!
//! Features
//! - `std`      : (default) use the Rust standard library
//! - `no-std`   : build with `#![no_std]` and use `libm`/`micromath` math backends
//! - `fast-math`: enable approximations (polys/rationals) for tanh/trig, etc.
//! - `simd`     : enable portable SIMD helper code paths (wide/safe_arch)
//!
//! Modules
//! - [`dsp`]       : math backend, utils (db/lin, smoothing, fast trig, meters)
//! - [`envelopes`] : ADSR (linear/exp), AR, slew limiter
//! - [`filters`]   : one-pole LP/HP/DC blocker, TPT SVF
//!
//! Design
//! - No heap allocations; pure sample-by-sample stateless/statEful primitives
//! - Clear separation between math helpers and filter/envelope building blocks
//! - Friendly to embedded / real-time targets

pub mod dsp;
pub mod envelopes;
pub mod filters;

/// Commonly used types/functions for convenience:
pub mod prelude {
    pub use crate::dsp::{
        clamp, db_to_lin, kill_denormals, lerp, lin_to_db, one_pole_coeff_hz, one_pole_coeff_ms,
        soft_clip, tpt_g, TAU,
    };
    pub use crate::envelopes::{AdsrExp, AdsrLinear, ArExp, SlewLimiter};
    pub use crate::filters::{DcBlock, OnePoleHP, OnePoleLP, SvfMode, SvfTpt};
}

#[cfg(test)]
mod smoke {

    #[test]
    fn prelude_exists() {
        use crate::prelude::*;
        let _ = db_to_lin(-6.0);
        let _ = AdsrLinear::new(10.0, 50.0, 0.5, 100.0, 48000.0);
        let mut lp = OnePoleLP::new(1000.0, 48000.0);
        let _ = lp.process(0.1);
    }
}
