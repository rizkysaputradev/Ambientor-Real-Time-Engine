#ifndef AMBIENTOR_H
#define AMBIENTOR_H

// Minimal C API for the Ambientor engine.
// Link against the produced staticlib/cdylib from the Rust `ambientor-ffi` crate.
//
// Build notes (Rust side):
//   - Crate type: staticlib + cdylib
//   - On build, `build.rs` will try to generate this header with `cbindgen`.
//     If unavailable, this checked-in header is used as a fallback.
//
// Threading:
//   - All functions are NOT thread-safe; call them all from the same audio thread.

#ifdef __cplusplus
extern "C" {
#endif

#include <stdint.h>
#include <stddef.h>

typedef struct AmbientorEngine AmbientorEngine; // Opaque handle

// --- Lifecycle ---------------------------------------------------------------

/**
 * Create a new engine with the default "slow_drone" scene.
 * @param sample_rate  device sample rate in Hz (e.g., 48000.0f)
 * @return non-null handle on success; NULL on allocation failure
 */
AmbientorEngine* ambientor_create(float sample_rate);

/**
 * Destroy an engine previously created by ambientor_create.
 */
void ambientor_destroy(AmbientorEngine* engine);

/**
 * Reset the engine to a new sample rate. Call this if the host changes
 * device configuration (e.g., different SR).
 */
void ambientor_reset(AmbientorEngine* engine, float sample_rate);

// --- Rendering ---------------------------------------------------------------

/**
 * Render `frames` of audio into an interleaved f32 buffer with `channels` channels.
 * The internal generator is mono; the sample is duplicated to all channels.
 *
 * @param engine           engine handle
 * @param out_interleaved  non-null pointer to output buffer (frames * channels floats)
 * @param frames           number of frames to render
 * @param channels         channel count (1..N)
 * @return frames rendered (0 on error)
 */
uint32_t ambientor_render_interleaved_f32(
    AmbientorEngine* engine,
    float* out_interleaved,
    uint32_t frames,
    uint32_t channels
);

// --- Global gain -------------------------------------------------------------

/**
 * Set post-engine gain applied by the FFI layer (>= 0).
 * This is separate from the scene's own smoothed output gain.
 */
void ambientor_set_gain(AmbientorEngine* engine, float gain);

// --- Scene parameter helpers -------------------------------------------------

/** Set base low-pass cutoff (Hz). */
void ambientor_scene_set_cut_base(AmbientorEngine* engine, float hz);

/** Set modulation span (Hz) around the base cutoff. */
void ambientor_scene_set_cut_span(AmbientorEngine* engine, float hz);

/** Set saturation drive (clamped internally to [0.1, 5.0]). */
void ambientor_scene_set_drive(AmbientorEngine* engine, float drive);

/** Set scene output gain (pre-FFI gain). */
void ambientor_scene_set_out_gain(AmbientorEngine* engine, float gain);

/** Set detune depth (in cents) for slow drift + LFO. */
void ambientor_scene_set_detune_cents(AmbientorEngine* engine, float cents);

#ifdef __cplusplus
} // extern "C"
#endif

#endif // AMBIENTOR_H
