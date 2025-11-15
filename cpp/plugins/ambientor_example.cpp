/**
 * Ambientor Example Plugin
 * -------------------------
 * Demonstrates embedding the Rust Ambientor engine in another C++ module.
 * Here we simply generate 5 seconds of stereo audio into a local buffer,
 * tweak some parameters dynamically, and compute the RMS.
 */

#include "ambientor.h"
#include <iostream>
#include <vector>
#include <cmath>
#include <numeric>

int main() {
    const uint32_t SR = 44100;
    const uint32_t CH = 2;
    const uint32_t DURATION = 5;
    const uint32_t FRAMES = SR * DURATION;

    AmbientorEngine* eng = ambientor_create((float)SR);
    if (!eng) {
        std::cerr << "Failed to create engine.\n";
        return 1;
    }

    // Parameter automation demo
    ambientor_scene_set_cut_base(eng, 1200.0f);
    ambientor_scene_set_cut_span(eng, 600.0f);
    ambientor_scene_set_drive(eng, 1.2f);
    ambientor_scene_set_detune_cents(eng, 10.0f);
    ambientor_scene_set_out_gain(eng, 0.4f);

    std::vector<float> buffer((size_t)FRAMES * CH);
    uint32_t wrote = ambientor_render_interleaved_f32(eng, buffer.data(), FRAMES, CH);
    if (wrote != FRAMES) {
        std::cerr << "Warning: wrote " << wrote << " / " << FRAMES << " frames\n";
    }

    // Compute simple RMS for diagnostic
    double sumsq = 0.0;
    for (float s : buffer) sumsq += s * s;
    double rms = std::sqrt(sumsq / buffer.size());
    std::cout << "Rendered " << wrote << " frames. RMS amplitude = " << rms << "\n";

    ambientor_destroy(eng);
    return 0;
}
