# 🎼 Multithreaded Ambientor System — Cross Platform Dynamic based Real-Time Ambient Sound Engine using Low Level DSP and SIMD handlings

## 📘 Overview
**Ambientor** is a **real-time ambient music engine** designed as both a **creative instrument** and a **systems-level DSP playground**. This project in particular encapsulates the capacity of **multi-language stack** which combines:

* 🦀 **Rust core DSP** (filters, envelopes, waveshaping, scenes)
* 💻 **C++ host** for low-level integration and embedding
* 🐍 **Python bindings** for quick scripting and experimentation
* 🧩 **Hand-tuned SIMD assembly** (AVX/SSE on x86, NEON on ARM64)
* 🎧 **Realtime CLI player** using `cpal` on macOS (and portable to Linux)

The primary goal of this particular project is to develop a **foundation** to a sound engine system with its robustness to perform several **low-level based** functionalities, likewise:
* Run **realtime ambient pads / drones / textures**.
* Demonstrate **SIMD acceleration** and **cross-language FFI**.
* Be used as a **reference project** for:
  * Writing DSP based program or code in **Rust**.
  * Calling it with other languages such as **C++** and **Python**.
  * Selectively dropping down to **assembly** for hot paths.

> Think of it as a **simple ambient DAW engine based kernel** that can be open, utilized, research, extend, and plug into other projects.

<p align="center">
  <img alt="Rust" src="https://img.shields.io/badge/Rust-ambientor_core-orange">
  <img alt="C++" src="https://img.shields.io/badge/C++-host-00599C">
  <img alt="Python" src="https://img.shields.io/badge/Python-bindings-3776AB">
  <img alt="DSP" src="https://img.shields.io/badge/DSP-envelopes%20%7C%20filters%20%7C%20waveshapers-purple">
  <img alt="Audio" src="https://img.shields.io/badge/Audio-cpal%20F32%2048kHz-ff69b4">
  <img alt="License" src="https://img.shields.io/badge/License-MIT-lightgrey">
</p>

---

## 🌲 Project Layout
At a high level, the project is specifically structured as the following (*Note: subject to change alongside development*):
```graphql
ambientor/
├─ asm/                      # Hand-written SIMD assembly
│  ├─ arm/
│  │  ├─ neon_mix.S          # NEON-accelerated mixing for f32 buffers
│  │  └─ neon_sine.S         # NEON sine table / poly evaluation
│  └─ x86/
│     ├─ avx_mix.S           # AVX mixing loop, SSE fallback for tails
│     └─ sse_sine.S          # SSE sine approximation (poly-based)
│
├─ cpp/                      # C++ host and FFI layer
│  ├─ CMakeLists.txt
│  ├─ include/
│  │  └─ ambientor/
│  │     └─ dsp_engine.hpp   # C++ view over the Rust core engine
│  └─ src/
│     ├─ main.cpp            # Example C++ host (standalone demo)
│     └─ dsp_engine.cpp      # FFI hooks into Rust (extern "C")
│
├─ rust/
│  ├─ ambientor-core/        # Core DSP algorithms and primitives
│  │  ├─ src/
│  │  │  ├─ lib.rs           # Crate root + public API
│  │  │  ├─ dsp.rs           # Math helpers, meters, DC blocker, fast trig
│  │  │  ├─ envelopes.rs     # ADSR (linear/exp), AR, slew
│  │  │  ├─ filters.rs       # One-pole filters, SVF helpers
│  │  │  ├─ arch.rs          # Arch-specific glue (SIMD hooks)
│  │  │  └─ scenes.rs        # High-level “scene” building blocks
│  │  └─ build.rs            # ASM compilation via cc-rs
│  │
│  ├─ ambientor-engine/      # Higher-level engine and scene logic
│  │  ├─ src/
│  │  │  ├─ lib.rs           # Engine abstraction over ambientor-core
│  │  │  ├─ graph.rs         # Routing, layers, buses
│  │  │  └─ scene_presets.rs # Hard-coded scene presets ("slow-drone", ...)
│  │  └─ Cargo.toml
│  │
│  └─ ambientor-cli/         # Realtime CLI player using cpal
│     ├─ src/
│     │  └─ main.rs          # Audio callback, device selection, meter
│     └─ Cargo.toml
│
├─ python/
│  ├─ pyproject.toml         # Maturin-based Python package config
│  ├─ README.md              # Python-specific docs and examples
│  ├─ src/
│  │  └─ ambientor_py/
│  │     └─ lib.rs           # pyo3 bindings to ambientor-engine
│  └─ ambientor_py/
│     ├─ __init__.py         # Import convenience + version
│     └─ _version.py
│
├─ scripts/
│  ├─ build_all.sh           # Build C++, Rust, Python in a single command
│  ├─ dev_setup.sh           # Dev env bootstrap (fmt, tools, venv)
│  └─ fmt_all.sh             # Format Rust/C++/Python/Shell
│
├─ Makefile                  # Top-level shortcuts (build, fmt, test, etc.)
├─ LICENSE                   # MIT License
├─ .gitignore                # Tuned for Rust/C++/Python/venv/artifacts
└─ README.md                 # This file
```

## 🧠 Concept and Purpose
This multithreaded Ambientor project is designed as a compact, transparent, and academically grounded exploration of how a real-time ambient sound engine can be constructed across multiple programming ecosystems. Instead of expanding into a large or opaque codebase, the system adopts a clean, layered architecture that spans **Rust, C++, Python, and low-level assembly**, providing a reproducible model for understanding real-time audio generation, systems programming, digital signal processing, and cross-language integration.

At its core, the project emphasizes **clarity, modularity, and cross-language interoperability**. It demonstrates how:

- Rust’s safety guarantees can coexist with external components via **C FFI**.  
- Rust modules can be embedded within **C++** applications without architectural friction.  
- High-level languages such as **Python** can interface with the same engine using `pyo3` and `maturin`.  
- **SIMD-optimized assembly** can be integrated in a controlled, testable, and portable manner.

Functionally, the engine provides **real-time stereo audio streaming** through the `cpal` backend while generating evolving ambient textures. These include amplitude envelopes, lightweight filtering, gradual modulation, and slowly dissipating low-frequency layers—behaviors aligned with foundational principles in sound synthesis and modern DSP. This makes the engine not only musically capable but also suitable for experimentation and academic analysis.

Ambientor is additionally structured around **benchmarkability and reproducibility**. Its design supports:

- Direct comparison between scalar and SIMD implementations  
- Measurement of CPU load, latency, and real-time performance  
- Rapid insertion of new waveforms, scenes, or modulation structures  
- Systematic evaluation of synthesis components under varying computational conditions  

In summary, Ambientor functions as a hybrid of **educational reference**, **real-time audio synthesizer**, and **performance-oriented research tool**, illustrating how a small but well-engineered system can bridge languages, abstractions, and low-level optimizations within a unified ambient audio engine.

## 🎛️ Core Features

### 🏗️ Rust Based DSP
- **Fast and Robust Math Helpers (`dsp.rs`):**
  - `fast_sin`, `fast_cos` using polynomial approximations  
  - dB/linear conversion (`db_to_lin`, `lin_to_db`)  
  - `tpt_g` helper for TPT filters  
  - RMS meter and DC-block high-pass  

- **Envelopes (`envelopes.rs`):**
  - `AdsrLinear` — classic line-segment ADSR  
  - `AdsrExp` — RC-style musical ADSR  
  - `ArExp` — percussive AR envelope  
  - `SlewLimiter` — parameter smoothing  

- **Filters (`filters.rs`):**
  - One-pole LP/HP  
  - SVF-style helpers (low-pass, band-pass, etc.)  
  - Coefficients built using `one_pole_coeff_ms` and `tpt_g`  

- **Scenes (`scenes.rs`):**
  - Building blocks for ambient layers:  
    - Slowly evolving drones  
    - Gentle noise beds  
    - Sine-based “breathing” bass with envelopes/filters  

### 🧩 Assembly Based SIMD
- **ARM NEON (aarch64):**
  - `neon_mix.S` — efficient `dst[i] += src[i] * gain` mixing  
  - `neon_sine.S` — vector sine approximation for blocks  

- **x86 AVX/SSE (x86_64):**
  - `avx_mix.S` — AVX main loop with SSE tail handling  
  - `sse_sine.S` — SSE polynomial sine approximation  

All of these built are wired through Rust `extern` functions and **feature-gated** implementations. Thus, the project always ensure a **safe scalar fallback** for faulty cases. However, make sure to test individually before deployment.

### 📢 Multilanguage Front-Ends
- **Rust CLI (`ambientor-cli`)**
    - Uses `cpal` to open your default audio device  
    - Configured for **48 kHz / F32 stereo**  
    - Streams a default scene (e.g., `"slow-drone"`)  
    - Includes a running peak meter  

- **C++ Host (`cpp/`)**
    - `main.cpp` demonstrates:
        - How to invoke the Rust engine from C++  
        - How to pass audio buffers back and forth  
        - How to embed Ambientor inside any existing C++ application  

- **Python Bindings (`python/`)**
    - `ambientor_py.Engine` class providing:
        - `render_block(num_frames, sr)`  
        - `set_scene(name, params)`  
    - Optional Python wheel build via **maturin**

## ⚙️ Build and Setup

### 🧑🏻‍🏫 Prerequisites
The ambientor system follows these specific preqrequisites throughout it development (subject to change):
- **Rust** (stable, via `rustup`), with `cargo`.
- **CMake** (for the C++ demo).
- A C compiler (`clang` / `gcc`).
- For Python:
  - Python 3.9–3.12
  - `maturin`
  - `virtualenv` or `venv` (recommended).

- Optional but recommended:
  - `clang-format` (for C++ formatting).
  - `black` and `ruff` (for Python formatting).
  - `cargo-edit`, `cargo-outdated`, etc. for Rust tooling.

### ☝🏻 One-shot Build (All Components)
From the project root, make sure to build and setup the project as follows:
```bash
make build
```

This bash executable is equivalent to:
- Running `scripts/build_all.sh --release` which:
  - Builds Rust crates (`ambientor-core`, `ambientor-engine`, `ambientor-cli`),
  - Builds the **C++ host** (via **CMake**),
  - Optionally builds **Python** bindings (depending on how it is configured).

For a debug build, make sure to use the following executable as such:
```bash
make debug
```

### 🕹️ Individual Components

#### Rust only
```bash
cd rust
cargo build --release -p ambientor-core -p ambientor-engine -p ambientor-cli
```

#### C++ only
```bash
cd cpp
mkdir -p build
cd build
cmake .. -DCMAKE_BUILD_TYPE=Release
cmake --build .
```

#### Python bindings
```bash
cd python

# Create & activate venv (recommended)
python3 -m venv .venv
source .venv/bin/activate

# Install maturin if needed
pip install --upgrade pip
pip install maturin

# Build editable dev install
maturin develop
```

### 🗼 Environment Bootstrap
In contrast, a “one command and forget” setup is similarly established as such:
```bash
make dev-setup
```

This specifically calls the following executable script:
```bash
scripts/dev_setup.sh
```

where it is able to do several of these functionalities:
- Check for `cargo`, `cmake`, and `python`,
- Install optional **linters/formatters** (if it is preferred),
- Set up **Python venv**,
- Run the **initial format passes**.

## ▶️ Running the Real-Time Player (Rust CLI)

The simplest way to *hear* Ambientor without any external handling is via the Rust CLI as shown below:
```bash
cd rust
cargo run --release -p ambientor-cli
```

The expected and common output is demonstrated by the following case:
```text
ambientor-cli — real-time ambient player

Using device: MacBook Pro Speakers
Stream config: StreamConfig { channels: 2, sample_rate: SampleRate(48000), buffer_size: … }
Scene: slow-drone  |  Gain: 0.35
Press Ctrl+C to stop…

[meter] peak ~ 0.061
[meter] peak ~ 0.051
[meter] peak ~ 0.065
...
```

- The default scene is rendering its audio like a **slow evolving drone** with **soft bass movement**.
- The **peak meter** gives a sense of output level and that the callback is alive/active.
- Hit the `Ctrl+C` key to stop/terminate the program or execution entirely.

> At the moment, the CLI focuses on **real-time playback**. “Render to file” commands are reserved for future extension (or can be done via Python bindings or a separate Rust binary).

## 🧪 Running Tests
The main unit tests live in the **Rust core** crate (`ambientor-core`). From the Rust directory, execute the following command via the terminal likewise shown below:
```bash
cd rust
cargo test -p ambientor-core --release
```

The executable should give the following output (some **FAILED** states can be accepted due to audio package version mismatches that may not directly affected the audio generation):

```text
running 11 tests
test dsp::tests::db_lin_roundtrip ... ok
test dsp::tests::soft_clip_is_bounded ... ok
test dsp::tests::rms_decreases_to_zero ... ok
test envelopes::tests::adsr_linear_reaches_sustain ... ok
test envelopes::tests::adsr_exp_behaves ... ok
test envelopes::tests::ar_exp_triggers_and_dies ... ok
test envelopes::tests::slew_moves_towards_target ... ok
test filters::tests::one_pole_lp_moves_towards_input ... ok
test filters::tests::svf_lp_is_sane ... ok
test filters::tests::one_pole_hp_blocks_dc ... ok
test smoke::prelude_exists ... ok
```

These tests ensures that:
- dB/linear conversions are approximately **invertible**.
- Soft clip is **bounded**.
- RMS meter **decays to zero**.
- ADSR envelopes **reach expected states**.
- Filters **move towards or away from DC** as intended.
- The crate’s public prelude **compiles and exports key items**.

## 🧮 DSP Internals
This specific project setup a Rust based DSP through modules with specific crucial roles as highlighted with their respective implementations likewise shown below:

### ♾️ Math and Utilities (`dsp.rs`)
- **Constants**
  - `TAU = 2π`
  - `EPS_SMALL = 1e-20` for denormal handling.

- **Core helpers**
  - `clamp`, `signum_nonzero`, `lerp`, `smoothstep`.
  - `wrap_phase01(p)`: wrap a phase to `[0,1]` using a fast floor trick.
  - `kill_denormals(x)`: zeroes very small values to avoid denormal slowdowns.

- **dB / linear**
  - `db_to_lin(db)` using `exp(ln(10)/20 * db)`.
  - `lin_to_db(lin)` using `20 * log10(lin)` with floor at `−120dB`.

- **Fast trig**
  - `fast_sin(x)`:
    - Range-reduce to `[−π, π]`.
    - 5th-order polynomial approximation when `fast-math` feature is on.
    - Falls back to `_m_sin` otherwise.
  - `fast_cos(x)` via `fast_sin(x + π/2)`.

- **Nonlinearities**
  - `soft_clip(x)`:
    - If `fast-math` enabled: rational approx of `tanh`.
    - Else: actual `tanh` backend (`std`, `libm`, or `micromath` depending on features).
  - `saturate(x, drive)` = `soft_clip(drive * x)`.

- **Smoothing**
  - `one_pole_coeff_ms(t_ms, sr)`: compute 1-pole smoothing coefficient from time constant.
  - `one_pole_coeff_hz(cut_hz, sr)`: frequency-based coefficient.
  - `tpt_g(cut_hz, sr)`: TPT parameter `g = tan(π * fc / sr)` with optional fast trigonometries.

- **Meters**
  - `Rms`: exponential-window RMS with `tick(x)`.

- **DC blocker**
  - `DcBlock`:
    - Difference eq: `y[n] = x[n] − x[n-1] + a * y[n-1]`.
    - Configurable `a` close to `1.0` yields very low cutoff.

### 🕰️ Shapes in Time (`envelopes.rs`)

- **AdsrLinear**
  - Attack, Decay, Release are in milliseconds.
  - Sustain is `[0, 1]`.
  - Internally precomputes:
    - `a_inc` = step to reach 1.0 in `atk_ms`.
    - `d_dec` = step to go from 1.0 → `sus` in `dec_ms`.
    - `r_dec` = step to go from `sus` → 0.0 in `rel_ms`.
    - These tests ensures that after a suitable execution or benchmarked run:
      - Envelope value is near `sus`.
      - Once gate is off, the envelope returns to ~0.

- **AdsrExp**
  - Uses `one_pole_coeff_ms` to get exponential slopes.
  - Stage equations:
    - **Attack:** `env += (1 − env) * (1 − a_a)`
    - **Decay:** `env += (sus − env) * (1 − a_d)`
    - **Release:** `env += (0 − env) * (1 − a_r)`
    - These tests ensures that:
      - `env.value() <= 1.0 + ε` during attack.
      - After enough release samples, `env.value()` is small.

- **ArExp**
  - Utilized for percussive events likewise:
    - `trigger()` resets `env` and sets a rising phase.
    - Attack: quickly moves `env → 1`.
    - Release: decays `env → 0`.

- **SlewLimiter**
  - One-pole smoothing for arbitrary control values.
  - `set_time_ms` adjusts how quickly it moves towards new targets.

### 📠 Frequency Domain Helpers (`filters.rs`)

- **One-pole LP/HP filters**:
  - LP: `y += a * (x − y)`
  - HP: `y_hp = x − y_lp` or direct difference form.
- **SVF-style filter**:
  - Based on `g = tpt_g(fc, sr)` and damping factors.
  - Exposes outputs such as LP, BP and HP.
  - These tests ensures that:
    - LP moves **towards a DC input**.
    - HP **blocks DC** and **yields small RMS** for constant input.
    - SVF LP **stays stable** for reasonable parameters.

## 🎎 Architecture Across Languages

### 🟠 Rust Core (`ambientor-core`)
The rust based core acts as the **reference implementation**, where:

- All math is **first implemented in Rust**.
- Assembly hooks are strictly **optional and feature-gated**. Given that scalar fallback always exists.
- High-level functions (e.g. `fill_sine`) internally:
  - Call SIMD if available (`aarch64` NEON, `x86_64` AVX/SSE).
  - Else use the scalar polynomial path with the same approximation used by the ASM code.

This means that the project systematically ensures that:
- There is always a present **portable, yet testable** path.
- SIMD vs scalar behavior is comparable within various set of tests or benchmarks.

### 🟧 Rust Engine (`ambientor-engine`)
The overall Rust module is built under an sngine with a setup as the following specifications:

- Defines an **"engine" abstraction** over the DSP primitives:
  - One or more **scenes**, like `"slow-drone"`.
  - Each scene has the capacity to:
    - Configures oscillators (sine, noise, etc.).
    - Wires them through envelopes and filters.
    - Combines them into stereo outputs.

The engine exposes simple methods as shown below (**to be adjusted in future developments**):
```rust
fn process_block(&mut self, out_l: &mut [f32], out_r: &mut [f32]);
fn set_scene(&mut self, scene_name: &str);
fn set_master_gain(&mut self, gain_db: f32);
```
> These methods are what the **ambientor-cli, C++, and Python** primarily bind into.

### 🟣 C++ Host (`cpp/`)
The C++ host is intentionally setup as a current addition for benchmark or prototype where:

- `dsp_engine.hpp` defines a minimal interface (**to be adjusted in future developments**):
```cpp
namespace ambientor {
    class Engine {
    public:
        Engine(float sample_rate, unsigned int channels);
        ~Engine();

        void set_scene(const std::string& name);
        void set_gain(float gain);
        void process(float* interleaved, std::size_t frames);
    };
}
```

- Internally, the `dsp_engine.cpp` has a crucial role liekwise:
  - Holds a pointer to a Rust-side engine,
  - Forwards `process` to a Rust FFI function.

Example usage is shown below (simplified):
```cpp
#include "ambientor/dsp_engine.hpp"

int main() {
    ambientor::Engine engine(48000.0f, 2);
    engine.set_scene("slow-drone");
    engine.set_gain(0.35f);

    std::vector<float> buffer(48000 * 2); // 1 second stereo
    engine.process(buffer.data(), 48000);

    // ... write buffer to file, send to audio API, etc.
}
```
> This pattern is ideal the Ambientor is embedded into **C++, C, C# game engines or audio tools**.

### 🔵 Python Bindings (`python/`)
The Python package (`ambientor_py`) wraps the Rust engine using **pyo3**. Below demonstrated this specific implementation through this conceptual API as such:

```python
from ambientor_py import AmbientEngine

engine = AmbientEngine(sample_rate=48000, channels=2, scene="slow-drone", gain_db=-10.0)

# Render 1 second of stereo audio
block = engine.render_block(num_frames=48000)
# block: np.ndarray of shape (48000, 2), dtype=float32
```

Where this enables:

- Simple and quick experimentaions using **Jupyter or Colab** based environments.
- Feeding Ambientor into:
  - Offline render scripts
  - ML pipelines
  - Additional effects written in Python

The **Python README** elaborates on installation and usage (see `python/README.md` for further analysis).

## 🧹 Formatting and Code Quality

Below shows the set of executables in order to keep the repo clean:

### Rust

```bash
cd rust
cargo fmt --all
cargo clippy --all-targets --all-features
```

### C++

```bash
make fmt-cpp
# or manually
find cpp -type f \( -name '*.cpp' -o -name '*.hpp' -o -name '*.c' -o -name '*.h' \) -exec clang-format -i {} +
```

### Python

```bash
make fmt-python
# or manually
black python
ruff check --fix python
```

## All at once
This executable will specifically call the `scripts/fmt_all.sh` module:
```bash
make fmt
```

## 🧰 Troubleshooting

### 🧭 Unknown arguments in ambientor-cli
If this specific ambientor module (via `ambientor_cli`) as shown below:
```bash
cargo run --release -p ambientor-cli -- render --seconds 15 ...
```

and the terminal returns the following warning messages:
```text
[warn] unknown arg: render
[warn] unknown arg: --seconds
...
ambientor-cli — real-time ambient player
...
```

This simply means that (*Note: make sure to run **diagnostics** or **benchmarking** before executing the sound tests*):
- Current CLI mode is running under the **real-time streaming only**.
- Extra “*render*” arguments are detected and logged, but ignored.

### 🩻 Assembly build errors (AVX / SSE / NEON)
If `cargo build -p ambientor-core` fails with assembler errors:

- Ensure the system is being executed on a supported arch:
  - `x86_64` for AVX/SSE code.
  - `aarch64` for NEON code.
- On macOS, `clang` is used as the assembler and is stricter:
  - The ASM has been adapted to 64-bit addressing and AT&T/Intel differences.
  - If there are further issues, temporarily disable the SIMD handler via features or build flags.

In order to temporarily bypass all the ASM errors:

- Make sure to leave comments or gate the `build.rs` ASM registration and rely purely on the scalar fallback (*The modules for the Rust code are still working independently and is currently being tested thoroughly*).

### 🚨 No sound / device issues
Verify that the output device is available (e.g., **Laptop/PC Speakers**):

- If there is no sound, try:
  - headphones vs built-in speakers.
  - Different device in the available system sound settings.
- `cpal` sometimes picks a default device that is muted or unexpected.

### ⚡️ Performance / CPU load
This specific ambientor sound based engine is fairly light. However, very old CPUs or devices that runs under heavy concurrent load can still experience glitches and errors. Thus, in order to improve the performance, ensure that:

- The complexity within the scenes (**fewer voices / layers**) is reduced thoroughly.
- **Release build** is used (`--release`) throughout the system deployment.
- Close other CPU-intensive apps or softwares to preserve the execution power.

## 🪄 Extending the Engine

### 🎬 Adding a New Scene (Rust)

In `ambientor-engine/src/scenes.rs`, implement the following changes as shown below:

#### 1. Define a new scene configuration:
```rust
pub enum SceneId {
    SlowDrone,
    ShimmerPad,
    // Add the configs:
    DarkBassSwells,
}
```

#### 2. Implement it in a factory:
```rust
pub fn make_scene(scene: SceneId, sr: f32) -> EngineGraph {
    match scene {
        SceneId::SlowDrone      => build_slow_drone(sr),
        SceneId::ShimmerPad     => build_shimmer_pad(sr),
        SceneId::DarkBassSwells => build_dark_bass_swells(sr),
    }
}
```

#### 3. Wire it into the CLI and C++/Python bindings.
### 🛰️ Using SIMDeD Mix
In order to call the SIMD mix from Rust, make sure to utilize the unified function in the `dsp.rs` module likewise:

```rust
use ambientor_core::dsp::mix_in_place;

fn blend_buffers(dst: &mut [f32], src: &[f32], gain: f32) {
    mix_in_place(dst, src, gain);
}
```
- On **x86_64 with AVX/SSE available**, this will call the `avx_mix` / SSE tail.
- On **aarch64 with NEON**, it will call `neon_mix`.
- Otherwise, it runs a scalar loop.

## 📦 Repository Hygiene (.gitignore)
The `.gitignore` is tuned so:

- **Rust** build artifacts (`target/`) are ignored.  
- **C++** build directory (`cpp/build/`) is ignored.  
- **Python** build artifacts and virtualenvs are ignored.  
- **OS/editor cruft** (`.DS_Store`, `.vscode/`, etc.) are ignored.  
- **Temporary audio export files** (e.g., `/tmp/ambientor_*.wav`) are ignored.

If the artificats is accidentally commit, make sure to run the executables as shown below:

```bash
git rm -r --cached .
git add .
git commit -m "clean: drop build artifacts and respect .gitignore"
```

## 🛣️ Future Work and Roadmaps
In order to further enhance the implementation of the multithreaded ambientor RT system, several plausible future innovations were established as shown below:

- **Offline renderer:**
  - A `render` subcommand for `ambientor-cli` to write `.wav` files.
- **More scenes:**
  - Evolving pads, rhythmic textures, granular-like swarms.
- **Modulation matrix:**
  - LFO routing to filter cutoff, pan, gain.
- **Preset system:**
  - JSON/TOML/YAML-based scene definitions.
- **GUI front-end:**
  - A minimal desktop UI (e.g. egui or a Python Qt front-end).
- **Benchmark harness:**
  - microbenchmarks comparing scalar vs SIMD vs different filter/envelope variants.

## 👤 Author and Credentials
This project is fully established and contributed by the following author:

- **Name:** Rizky Johan Saputra
- **Institution:** Independent
- **Role:** Project Developer, Manager and Author 
- **Affiliation:** Undergraduate at Seoul National University (Enrolled at 2021, Graduating in 2026)
- **Project Scope:** System Programming, Computer Architecture, Real-Time Systems, DSP and Audio System, Cross-language and FFI Programming.

## 📜 License
This repository is distributed under an Independent Personal License tailored by the author. See `LICENSE` for the full terms. For further inquiries and requests, please contact via GitHub or Email only.
> If you intend to reuse significant portions for research and academia purposes, please open and inquire an issue to discuss attribution and terms.

## 📎 Appendix

### 🪜 Suggested CLI Usage Patterns
- **Quick sanity check**

```bash
cd rust
cargo run --release -p ambientor-cli
```

Listen for the following outputs for diagnostics:
- Clean, stable ambient output.
- No audible clicks/pops on startup,
- Smooth changes in the peak meter.
- Experiment with code changes.

#### 1. Modify a scene in `ambientor-engine` (e.g. change envelope times).

#### 2. Rebuild:
```bash
cd rust
cargo build --release -p ambientor-engine -p ambientor-cli
```

#### 3. Re-run `ambientor-cli` and listen.

## 💾 Robust Default Design Choices
For a stable and pleasant deployment, make sure to utilize the following default settings or adjust it accordingly:

- Sample rate: **48000 Hz**
- Master gain: around **−10 dB** to **−6 dB**
- Use **exponential** envelopes for musical feel:
  - Attack: **50–300 ms**
  - Decay: **500–2000 ms**
  - Release: **1000–5000 ms**
- Use a **DC blocker** on low-frequency paths.
- Apply a gentle **low-pass** (SVF) around **8–12 kHz** to smooth harshness.

## ⌨️ Keyboard and Control Dynamics
In future developments where a more robust CLI or GUI were to be developed, make sure to implement as such:
- **Quit:** `[q]`
- **Switch scenes:** `[1]/[2]/[3]` 
- **Adjust gain:** `[+]/[−]`
- **Toggle a filter or modulation mode:** `[f]`

The engine is specifically structured so wiring this kind of control is straightforward and practical.

---
# <p align="center"><b>🎧 Synthesizing Ambient Sounds from Pure Computations and Algorithms 📡 </b></p>