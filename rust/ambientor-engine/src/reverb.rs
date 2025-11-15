//! Lightweight mono reverb (no heap, realtime-safe).
//!
//! Design
//! - Simple “Schroeder-ish” structure: 2 short all-passes → 4 LP-combs in parallel → 2 all-passes.
//! - No allocations; fixed-size delay lines sized for up to ~0.7 s at 48 kHz.
//! - Tunable `room` (feedback), `damp` (HF damping in feedback), `mix` (dry/wet).
//!
//! This is intentionally modest in CPU and memory while still giving a pleasant wash
//! for ambient drones. Output is **mono**; the CLI duplicates it to device channels.

use core::fmt::Debug;
use ambientor_core::dsp::{kill_denormals};
use ambientor_core::filters::OnePoleLP;

/// Fixed sizes for delay lines (compile-time, stack-allocated inside the struct).
const MAX_PRE_AP: usize   = 2048;   // ~43 ms @ 48k
const MAX_TANK:   usize   = 34000;  // ~0.708 s @ 48k
const MAX_POST_AP: usize  = 4096;   // ~85 ms @ 48k

#[derive(Copy, Clone, Debug)]
struct DelayLine<const N: usize> {
    buf: [f32; N],
    i: usize,
    len: usize,
}
impl<const N: usize> DelayLine<N> {
    #[inline] fn new() -> Self { Self { buf: [0.0; N], i: 0, len: N.min(1) } }
    #[inline] fn set_len(&mut self, len: usize) { self.len = len.max(1).min(N); if self.i >= self.len { self.i = 0; } }
    #[inline] fn read(&self) -> f32 { self.buf[self.i] }
    #[inline] fn write_advance(&mut self, x: f32) {
        self.buf[self.i] = x;
        self.i += 1;
        if self.i >= self.len { self.i = 0; }
    }
}

/// Simple all-pass: y = -g*x + d + g*y_prev_path, with a single delay.
/// Canonical “feedforward + feedback” all-pass.
#[derive(Copy, Clone, Debug)]
struct Allpass<const N: usize> {
    d: DelayLine<N>,
    g: f32,
}
impl<const N: usize> Allpass<N> {
    #[inline] fn new(g: f32) -> Self { Self { d: DelayLine::new(), g } }
    #[inline] fn set_len(&mut self, len: usize) { self.d.set_len(len); }
    #[inline] fn set_g(&mut self, g: f32) { self.g = g.clamp(-0.999, 0.999); }
    #[inline] fn process(&mut self, x: f32) -> f32 {
        let z = self.d.read();
        let y = z - self.g * x;
        self.d.write_advance(x + self.g * y);
        kill_denormals(y)
    }
}

/// Feedback comb with an LP filter inside the feedback path (for damping).
#[derive(Copy, Clone, Debug)]
struct CombLp<const N: usize> {
    d: DelayLine<N>,
    fb: f32,
    lp: OnePoleLP, // simple tone in the loop (acts like HF damping when cut is low)
}
impl<const N: usize> CombLp<N> {
    #[inline] fn new(sr: f32) -> Self { Self { d: DelayLine::new(), fb: 0.7, lp: OnePoleLP::new(8000.0, sr) } }
    #[inline] fn set_len(&mut self, len: usize) { self.d.set_len(len); }
    #[inline] fn set_feedback(&mut self, fb: f32) { self.fb = fb.clamp(0.0, 0.99); }
    #[inline] fn set_damp_cut(&mut self, hz: f32) { self.lp.set_cutoff_hz(hz); }
    #[inline] fn set_sr(&mut self, sr: f32) { self.lp.set_sample_rate(sr); }
    #[inline] fn process(&mut self, x: f32) -> f32 {
        let z = self.d.read();
        let z_damped = self.lp.process(z);
        let y = z; // feed-out
        self.d.write_advance(x + self.fb * z_damped);
        kill_denormals(y)
    }
}

/// Mono reverb with small footprint.
#[derive(Copy, Clone, Debug)]
pub struct ReverbLite {
    sr: f32,
    // pre-diffusion
    ap1: Allpass<MAX_PRE_AP>,
    ap2: Allpass<MAX_PRE_AP>,
    // tank: 4 combs with slightly detuned lengths
    c1: CombLp<MAX_TANK>,
    c2: CombLp<MAX_TANK>,
    c3: CombLp<MAX_TANK>,
    c4: CombLp<MAX_TANK>,
    // post diffusion
    ap3: Allpass<MAX_POST_AP>,
    ap4: Allpass<MAX_POST_AP>,
    // user params
    room: f32,  // 0..1 → mapped to feedback
    damp: f32,  // 0..1 → mapped to comb LP cutoff
    mix:  f32,  // 0..1 (wet)
    pre_delay_samps: usize,
}
impl ReverbLite {
    #[inline]
    pub fn new(sr: f32) -> Self {
        let mut s = Self {
            sr,
            ap1: Allpass::new(0.7),
            ap2: Allpass::new(0.7),
            c1: CombLp::new(sr),
            c2: CombLp::new(sr),
            c3: CombLp::new(sr),
            c4: CombLp::new(sr),
            ap3: Allpass::new(0.6),
            ap4: Allpass::new(0.6),
            room: 0.6,
            damp: 0.4,
            mix:  0.25,
            pre_delay_samps: 0,
        };
        s.reset(sr);
        s
    }

    #[inline]
    pub fn reset(&mut self, sr: f32) {
        self.sr = sr.max(1.0);
        // Set allpass/comb lengths (in samples) relative to SR.
        // Keep within compile-time maximums.
        let scale = self.sr / 48000.0;
        self.ap1.set_len(( 641.0 * scale) as usize); self.ap1.set_g(0.72);
        self.ap2.set_len(( 997.0 * scale) as usize); self.ap2.set_g(0.70);

        // Choose mutually prime-ish lengths to avoid obvious ringing.
        self.c1.set_len(( 7789.0 * scale) as usize);
        self.c2.set_len(( 8513.0 * scale) as usize);
        self.c3.set_len(( 9449.0 * scale) as usize);
        self.c4.set_len((10867.0 * scale) as usize);
        for c in [&mut self.c1, &mut self.c2, &mut self.c3, &mut self.c4] {
            c.set_sr(self.sr);
        }

        self.ap3.set_len((  579.0 * scale) as usize); self.ap3.set_g(0.65);
        self.ap4.set_len((  773.0 * scale) as usize); self.ap4.set_g(0.61);

        // Default pre-delay ~ 12 ms
        self.pre_delay_samps = (self.sr * 0.012) as usize;
        self.update_params();
    }

    #[inline]
    fn update_params(&mut self) {
        let fb = 0.55 + 0.40 * self.room.clamp(0.0, 1.0); // 0.55..0.95
        let cut = 2000.0 + 12000.0 * (1.0 - self.damp.clamp(0.0, 1.0)); // damp=1 → darker
        for c in [&mut self.c1, &mut self.c2, &mut self.c3, &mut self.c4] {
            c.set_feedback(fb);
            c.set_damp_cut(cut);
        }
        self.mix = self.mix.clamp(0.0, 1.0);
    }

    #[inline] pub fn set_room(&mut self, v: f32) { self.room = v; self.update_params(); }
    #[inline] pub fn set_damp(&mut self, v: f32) { self.damp = v; self.update_params(); }
    #[inline] pub fn set_mix(&mut self, v: f32)  { self.mix  = v; self.update_params(); }

    /// Process one mono sample; returns the reverberated (dry+wet) sample.
    #[inline]
    pub fn process(&mut self, x: f32) -> f32 {
        // Optional pre-delay: we approximate by pushing zeros before starting the tank
        // when the scene first runs. For simplicity in a streaming context, we model
        // it as two short APs acting as a diffuser (already set up above).
        let pre = self.ap2.process(self.ap1.process(x));

        // Parallel combs
        let y1 = self.c1.process(pre);
        let y2 = self.c2.process(pre);
        let y3 = self.c3.process(pre);
        let y4 = self.c4.process(pre);
        let sum = 0.25 * (y1 + y2 + y3 + y4);

        // Post diffusion
        let post = self.ap4.process(self.ap3.process(sum));

        // Mix
        let wet = post;
        let dry = x;
        let y = (1.0 - self.mix) * dry + self.mix * wet;
        kill_denormals(y)
    }
}
