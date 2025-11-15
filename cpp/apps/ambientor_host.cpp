/**
 * Ambientor Host — Real-Time C++ Host using RtAudio
 * Streams the Rust ambientor_ffi engine through RtAudio.
 */

#include "ambientor.h"
#include <rtaudio/RtAudio.h>
#include <iostream>
#include <vector>
#include <thread>
#include <chrono>
#include <cstring>
#include <csignal>
#include <exception>
#include <algorithm>

static bool g_running = true;
static void signal_handler(int) { g_running = false; }

struct HostState {
    AmbientorEngine* engine = nullptr;
    unsigned int sr       = 48000;
    unsigned int channels = 2;
    float host_gain       = 0.35f;   // host-side duplication gain

    // Scene params (sent to Rust via FFI)
    float cut_base_hz     = 1200.0f;
    float cut_span_hz     = 800.0f;
    float drive           = 1.2f;
    float scene_out_gain  = 0.80f;
    float detune_cents    = 7.0f;
};

// RtAudio callback: pull frames from the Rust engine via FFI.
static int audio_callback(
    void* outputBuffer, void* /*inputBuffer*/,
    unsigned int nBufferFrames, double /*streamTime*/,
    unsigned int status, void* userData)
{
    if (status) std::cerr << "[RtAudio] Stream under/overflow!\n";
    auto* st  = reinterpret_cast<HostState*>(userData);
    auto* out = reinterpret_cast<float*>(outputBuffer);

    if (!st || !st->engine || !out || nBufferFrames == 0) {
        return 1; // stop if invalid
    }

    const uint32_t wrote = ambientor_render_interleaved_f32(
        st->engine, out, nBufferFrames, st->channels);

    if (wrote < nBufferFrames) {
        const size_t off = static_cast<size_t>(wrote) * st->channels;
        const size_t rem = static_cast<size_t>(nBufferFrames) * st->channels - off;
        std::fill(out + off, out + off + rem, 0.0f);
    }
    return g_running ? 0 : 1; // non-zero -> stop
}

int main(int argc, char** argv) {
    HostState st;
    unsigned int seconds = 0;

    // Minimal CLI
    for (int i = 1; i < argc; ++i) {
        if (!std::strcmp(argv[i], "--sr")       && i + 1 < argc) st.sr         = static_cast<unsigned int>(std::stoi(argv[++i]));
        else if (!std::strcmp(argv[i], "--gain")&& i + 1 < argc) st.host_gain  = std::stof(argv[++i]);
        else if (!std::strcmp(argv[i], "--ch")  && i + 1 < argc) st.channels   = static_cast<unsigned int>(std::stoi(argv[++i]));
        else if (!std::strcmp(argv[i], "--duration") && i + 1 < argc) seconds  = static_cast<unsigned int>(std::stoi(argv[++i]));
        // NEW scene shaping flags
        else if (!std::strcmp(argv[i], "--cut-base") && i + 1 < argc) st.cut_base_hz = std::stof(argv[++i]);
        else if (!std::strcmp(argv[i], "--cut-span") && i + 1 < argc) st.cut_span_hz = std::stof(argv[++i]);
        else if (!std::strcmp(argv[i], "--drive")    && i + 1 < argc) st.drive       = std::stof(argv[++i]);
        else if (!std::strcmp(argv[i], "--scene-gain") && i + 1 < argc) st.scene_out_gain = std::stof(argv[++i]);
        else if (!std::strcmp(argv[i], "--detune")   && i + 1 < argc) st.detune_cents = std::stof(argv[++i]);
    }

    std::cout << "Ambientor Host (RtAudio)\n"
              << "----------------------------------------\n"
              << "Sample rate : " << st.sr << "\n"
              << "Channels    : " << st.channels << "\n"
              << "Host gain   : " << st.host_gain << "\n"
              << "Duration    : " << (seconds ? std::to_string(seconds) + " s" : "∞") << "\n"
              << "Scene params: cut_base=" << st.cut_base_hz
              << " Hz  cut_span=" << st.cut_span_hz
              << " Hz  drive=" << st.drive
              << "  scene_gain=" << st.scene_out_gain
              << "  detune=" << st.detune_cents << " cents\n";

    // Init Rust engine
    st.engine = ambientor_create(static_cast<float>(st.sr));
    if (!st.engine) {
        std::cerr << "[FATAL] FFI ambientor_create() returned null.\n";
        return 1;
    }

    // Push scene params into Rust via FFI (these map to your ambientor-ffi setters)
    ambientor_scene_set_cut_base(st.engine, st.cut_base_hz);
    ambientor_scene_set_cut_span(st.engine, st.cut_span_hz);
    ambientor_scene_set_drive(st.engine, st.drive);
    ambientor_scene_set_out_gain(st.engine, st.scene_out_gain);
    ambientor_scene_set_detune_cents(st.engine, st.detune_cents);

    // RtAudio host setup
    RtAudio dac;
    unsigned int deviceCount = 0;
    try {
        deviceCount = dac.getDeviceCount();
    } catch (const std::exception& e) {
        std::cerr << "[ERR] RtAudio getDeviceCount failed: " << e.what() << "\n";
        ambientor_destroy(st.engine);
        return 1;
    }

    if (deviceCount < 1) {
        std::cerr << "[ERR] No audio devices found!\n";
        ambientor_destroy(st.engine);
        return 1;
    }

    std::cout << "Available output devices:\n";
    for (unsigned int i = 0; i < deviceCount; ++i) {
        try {
            auto info = dac.getDeviceInfo(i);
            std::cout << "  [" << i << "] " << info.name
                      << " | outputs: " << info.outputChannels
                      << (info.isDefaultOutput ? " (default)" : "")
                      << "\n";
        } catch (const std::exception& e) {
            std::cerr << "  [" << i << "] <unavailable>: " << e.what() << "\n";
        }
    }

    // Use the default output device (safe getDeviceInfo)
    unsigned int defaultId = 0;
    RtAudio::DeviceInfo info;
    try {
        defaultId = dac.getDefaultOutputDevice();
        info = dac.getDeviceInfo(defaultId);
    } catch (const std::exception& e) {
        std::cerr << "[ERR] Failed to acquire default device info: " << e.what() << "\n";
        ambientor_destroy(st.engine);
        return 1;
    }

    std::cout << "Using device: " << info.name
              << " | Outputs: " << info.outputChannels << " channels\n";

    if (info.outputChannels == 0) {
        std::cerr << "[ERR] Selected device has zero output channels.\n";
        ambientor_destroy(st.engine);
        return 1;
    }
    if (info.outputChannels < st.channels) {
        std::cerr << "[WARN] Requested " << st.channels
                  << " channels but device only supports "
                  << info.outputChannels << ". Adjusting.\n";
        st.channels = info.outputChannels;
    }

    // Prepare stream parameters
    RtAudio::StreamParameters oparams;
    oparams.deviceId     = defaultId;
    oparams.nChannels    = st.channels;
    oparams.firstChannel = 0;

    RtAudio::StreamOptions options;
    options.flags = 0;

    unsigned int bufferFrames = 256; // good default on macOS

    try {
        dac.openStream(&oparams, nullptr, RTAUDIO_FLOAT32,
                       st.sr, &bufferFrames, &audio_callback, &st, &options);
    } catch (const std::exception& e) {
        std::cerr << "[ERR] RtAudio openStream failed: " << e.what() << "\n";
        ambientor_destroy(st.engine);
        return 1;
    } catch (...) {
        std::cerr << "[ERR] RtAudio openStream unknown exception.\n";
        ambientor_destroy(st.engine);
        return 1;
    }

    // Catch SIGINT/SIGTERM to stop
    std::signal(SIGINT,  signal_handler);
    std::signal(SIGTERM, signal_handler);

    try {
        dac.startStream();
        const auto t0 = std::chrono::steady_clock::now();

        while (g_running) {
            std::this_thread::sleep_for(std::chrono::milliseconds(100));
            if (seconds) {
                const auto dt = std::chrono::steady_clock::now() - t0;
                if (std::chrono::duration_cast<std::chrono::seconds>(dt).count() >= seconds)
                    break;
            }
        }

        if (dac.isStreamRunning()) dac.stopStream();
    } catch (const std::exception& e) {
        std::cerr << "[ERR] RtAudio runtime exception: " << e.what() << "\n";
    } catch (...) {
        std::cerr << "[ERR] RtAudio runtime unknown exception.\n";
    }

    if (dac.isStreamOpen()) dac.closeStream();
    ambientor_destroy(st.engine);
    std::cout << "Exiting cleanly.\n";
    return 0;
}
