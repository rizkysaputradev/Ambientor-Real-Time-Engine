//! Musical scenes that implement the realtime [`Generator`](crate::graph::Generator) trait.
//!
//! Scenes are **mono** generators; the CLI duplicates the sample to however many
//! channels the device needs. Keep scenes allocation-free and cheap per sample.

use crate::graph::Generator;
use crate::nodes::{Osc, Lfo, NoiseMod, Wave, OnePoleSmoother};
use ambientor_core::filters::OnePoleLP;
use ambientor_core::dsp::{saturate};
use crate::reverb::ReverbLite;

/// A single scene instance. Add new fields as new scenes/features grow.
///
/// This starter scene is a **slow evolving drone**:
/// - Two oscillators (tri + saw) near a musical interval,
/// - Very slow drift in cutoff and detune,
/// - Gentle low-pass tone control,
/// - Mild saturation,
/// - Lightweight mono reverb for space.
#[derive(Copy, Clone)]
pub struct Scene {
    // tone sources
    osc_a: Osc,
    osc_b: Osc,
    // motion
    lfo_cut: Lfo,
    drift_detune: NoiseMod,
    // tone shaping
    lp: OnePoleLP,
    // output stage
    rev: ReverbLite,
    // parameters
    sr: f32,
    base_cut: f32,
    cut_span: f32,
    detune_cents: f32,
    drive: f32,
    out_gain: f32,
    // smoothed controls
    gain_sm: OnePoleSmoother,
}
impl core::fmt::Debug for Scene {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Scene::slow_drone")
            .field("sr", &self.sr)
            .finish()
    }
}

impl Scene {
    /// Construct the default “slow_drone” scene. Safe defaults for 44.1–48 kHz.
    pub fn slow_drone(sr: f32) -> Self {
        let mut s = Self {
            // Sources (rough A2 + sub/5th; adjust by ear)
            osc_a: Osc::new(110.0, Wave::Tri),
            osc_b: Osc::new(110.0 * 0.498, Wave::Saw),
            // Motion
            lfo_cut: Lfo::sine(0.05), // ~20 s period
            drift_detune: NoiseMod::new(-6.0, 6.0, 7.5, 0.25, sr), // ±6 cents target, pick every ~7.5 s
            // Tone shaping
            lp: OnePoleLP::new(900.0, sr),
            // Space
            rev: ReverbLite::new(sr),
            // Params
            sr,
            base_cut: 900.0,
            cut_span: 600.0,
            detune_cents: 3.0, // depth of LFO on detune (additional to noise drift)
            drive: 0.9,
            out_gain: 0.33,
            gain_sm: OnePoleSmoother::new_ms(30.0, sr),
        };
        s.gain_sm.reset(s.out_gain);
        s
    }

    /// Tweakers (optional use at runtime from host if you expose a control UI)
    #[inline] pub fn set_cut_base(&mut self, hz: f32) { self.base_cut = hz.max(50.0); }
    #[inline] pub fn set_cut_span(&mut self, hz: f32) { self.cut_span = hz.max(0.0); }
    #[inline] pub fn set_drive(&mut self, d: f32)     { self.drive = d.clamp(0.1, 5.0); }
    #[inline] pub fn set_gain(&mut self, g: f32)      { self.out_gain = g.clamp(0.0, 1.0); }
    #[inline] pub fn set_detune_cents(&mut self, c: f32) { self.detune_cents = c.clamp(0.0, 25.0); }

    #[inline]
    fn cents_to_ratio(c: f32) -> f32 {
        // 1200 cents = 2x; ratio = 2^(c/1200)
        (core::f32::consts::LN_2 * (c / 1200.0)).exp()
    }
}

impl Generator for Scene {
    #[inline]
    fn reset(&mut self, sr: f32) {
        self.sr = sr.max(1.0);
        self.lp.set_sample_rate(self.sr);
        self.lfo_cut.set_rate(0.05);
        self.drift_detune.reset_sr(self.sr);
        self.rev.reset(self.sr);
        self.gain_sm.set_time_ms(30.0, self.sr);
    }

    #[inline]
    fn next(&mut self) -> f32 {
        let sr = self.sr;

        // Evolving cutoff: base ± span via very slow LFO
        let lfo01 = self.lfo_cut.next01(sr); // 0..1
        let cut = self.base_cut + (lfo01 - 0.5) * 2.0 * self.cut_span;
        self.lp.set_cutoff_hz(cut.max(80.0));

        // Very slow detune drift (in cents) + subtle LFO detune
        let drift_cents = self.drift_detune.next(sr);            // in [-6, +6] by design
        let lfo_cents   = (lfo01 - 0.5) * 2.0 * self.detune_cents;
        let ratio_a = Self::cents_to_ratio(drift_cents + 0.5 * lfo_cents);
        let ratio_b = Self::cents_to_ratio(-drift_cents + lfo_cents);

        self.osc_a.set_freq(110.0 * ratio_a);
        self.osc_b.set_freq(110.0 * 0.498 * ratio_b);

        // Tone + very light saturation
        let x = 0.5 * (self.osc_a.next(sr) + self.osc_b.next(sr));
        let tone = self.lp.process(x);
        let sat = saturate(tone, self.drive);

        // Reverb space
        let wet = self.rev.process(sat);

        // Smooth output gain to avoid clicks on runtime tweaks
        let g = self.gain_sm.process(self.out_gain);

        // Final output
        (wet * g).clamp(-1.0, 1.0)
    }
}
