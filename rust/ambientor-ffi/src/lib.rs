//! C ABI wrapper for the Ambientor engine.
//!
//! Exposes a small set of functions to create/destroy an engine,
//! render interleaved f32 samples, and tweak a few scene parameters.
//!
//! ABI notes
//! - All functions are `extern "C"` and `#[no_mangle]`.
//! - Opaque handle type: `AmbientorEngine` (heap-allocated; you own/delete it).
//! - Render path produces **mono** internally and duplicates to N channels.
//!
//! Threading
//! - The object is NOT thread-safe; call all functions from the same audio thread.

use ambientor_engine::{Engine};
use ambientor_engine::scenes::Scene;
use ambientor_engine::Generator;


/// Opaque engine wrapper we hand to C.
///
/// We keep the sample rate here so we can call `engine.next(sr)` without the caller
/// passing SR for every sample. The host should call `ambientor_reset(engine, sr)`
/// on reconfiguration.
#[repr(C)]
pub struct AmbientorEngine {
    sr: f32,
    gain: f32,
    inner: Engine<Scene>,
}

impl AmbientorEngine {
    fn new(sr: f32) -> Self {
        let sr = sr.max(1.0);
        let scene = Scene::slow_drone(sr);
        let mut e = Engine::new(scene);
        // ensure scene got the exact SR we want
        e.scene_mut().reset(sr);
        Self { sr, gain: 1.0, inner: e }
    }
}

// --- Creation / destruction -------------------------------------------------------

/// Create a new engine with a default “slow_drone” scene.
/// Returns a non-null pointer on success, or null on allocation failure.
#[no_mangle]
pub extern "C" fn ambientor_create(sample_rate: f32) -> *mut AmbientorEngine {
    let eng = AmbientorEngine::new(sample_rate);
    match Box::into_raw(Box::new(eng)) as *mut AmbientorEngine {
        p if p.is_null() => std::ptr::null_mut(),
        p => p,
    }
}

/// Destroy an engine previously returned by `ambientor_create`.
#[no_mangle]
pub extern "C" fn ambientor_destroy(engine: *mut AmbientorEngine) {
    if !engine.is_null() {
        unsafe { drop(Box::from_raw(engine)); }
    }
}

/// Reset the engine to a new sample rate (e.g., when host changes device config).
#[no_mangle]
pub extern "C" fn ambientor_reset(engine: *mut AmbientorEngine, sample_rate: f32) {
    if engine.is_null() { return; }
    let e = unsafe { &mut *engine };
    e.sr = sample_rate.max(1.0);
    e.inner.scene_mut().reset(e.sr);
}

// --- Rendering -------------------------------------------------------------------

/// Render `frames` of audio into an interleaved f32 buffer with `channels` channels.
/// The internal generator is mono; the sample is duplicated to all channels.
///
/// Returns the number of frames rendered (0 on error).
#[no_mangle]
pub extern "C" fn ambientor_render_interleaved_f32(
    engine: *mut AmbientorEngine,
    out_interleaved: *mut f32,
    frames: u32,
    channels: u32,
) -> u32 {
    if engine.is_null() || out_interleaved.is_null() || frames == 0 || channels == 0 {
        return 0;
    }
    let e = unsafe { &mut *engine };
    let out = unsafe { std::slice::from_raw_parts_mut(out_interleaved, (frames as usize) * (channels as usize)) };

    let sr = e.sr;
    let ch = channels as usize;

    // Generate samples
    let mut idx = 0usize;
    for _ in 0..(frames as usize) {
        let s = e.inner.next(sr) * e.gain;
        for _c in 0..ch {
            out[idx] = s;
            idx += 1;
        }
    }
    frames
}

// --- Scene parameter helpers ------------------------------------------------------

/// Set overall output gain (0..1 suggested). Values are clamped to [0, +inf).
#[no_mangle]
pub extern "C" fn ambientor_set_gain(engine: *mut AmbientorEngine, gain: f32) {
    if engine.is_null() { return; }
    let e = unsafe { &mut *engine };
    e.gain = if gain.is_finite() { gain.max(0.0) } else { 1.0 };
}

/// Set the base low-pass cutoff (Hz) for the scene.
#[no_mangle]
pub extern "C" fn ambientor_scene_set_cut_base(engine: *mut AmbientorEngine, hz: f32) {
    if engine.is_null() { return; }
    let e = unsafe { &mut *engine };
    e.inner.scene_mut().set_cut_base(hz);
}

/// Set the modulation span (Hz) around the base cutoff.
#[no_mangle]
pub extern "C" fn ambientor_scene_set_cut_span(engine: *mut AmbientorEngine, hz: f32) {
    if engine.is_null() { return; }
    let e = unsafe { &mut *engine };
    e.inner.scene_mut().set_cut_span(hz);
}

/// Set drive (saturation intensity), clamped internally to [0.1, 5.0].
#[no_mangle]
pub extern "C" fn ambientor_scene_set_drive(engine: *mut AmbientorEngine, drive: f32) {
    if engine.is_null() { return; }
    let e = unsafe { &mut *engine };
    e.inner.scene_mut().set_drive(drive);
}

/// Set scene output gain (pre-FFI gain smoothing).
#[no_mangle]
pub extern "C" fn ambientor_scene_set_out_gain(engine: *mut AmbientorEngine, gain: f32) {
    if engine.is_null() { return; }
    let e = unsafe { &mut *engine };
    e.inner.scene_mut().set_gain(gain);
}

/// Set detune depth (in cents) for slow drift + LFO.
#[no_mangle]
pub extern "C" fn ambientor_scene_set_detune_cents(engine: *mut AmbientorEngine, cents: f32) {
    if engine.is_null() { return; }
    let e = unsafe { &mut *engine };
    e.inner.scene_mut().set_detune_cents(cents);
}
