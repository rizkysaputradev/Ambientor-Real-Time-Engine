# 🤖 Ambientor - Python Guide

This project structure specifically set the python bindings for the **Ambientor** real-time ambient sound engine.

This package exposes a small, clean API around the native Ambientor engine, so the project can be rendered with evolving ambient textures directly from Python, or embedded in notebooks, tools, and experiments.

- 🧠 Backed by the Rust `ambientor-engine` / `ambientor-core` crates  
- 🎧 Generates interleaved floating-point audio buffers  
- 💾 Can render directly to 16-bit PCM WAV files  
- 🐍 Distributed as a `pyo3`-based extension module via **maturin**

---

## 💳 Sub Project Layout
This project is specifically stuctured as the following layout:
```text
python/
  pyproject.toml      # maturin / PEP 621 metadata
  README.md           # this file
  src/
    lib.rs            # Rust → Python bindings (pyo3)
  ambientor_py/
    __init__.py       # high-level Python API + factory
    _version.py       # version shim
```

The compiled **Rust extension** is built and exported from **ambientor_py/__init__.py** as such:
```text
ambientor_py/_ambientor.*   # platform-specific extension module
```

## ✅ Requirements
The default requirement that was setup within the project development is shown below:
* Python 3.9+
* A Rust toolchain (*stable*)
* maturin >= 1.7 installed in your Python environment
> Adjustment is possible but may require cohesive debugging with other in-built projects (Rust, C, C++, Bash projects) and inclusions for cross platform devices.

Specifically, **rustup** and **maturin** can be installed on macOS with Homebrew as follows:
```bash
brew install rustup
rustup-init   # if you haven't already
pip install maturin
```

## 🔩 Development and Installation
From the python/ directory:
```bash
cd python

# Build and install the extension into the current virtualenv
maturin develop
```

This will:
	•	Compile the Rust extension (src/lib.rs) as a cdylib
	•	Install the ambientor_py package into your current Python environment

If you’re inside a virtualenv / .venv, after maturin develop you can
immediately import and use the package.

## Quickstart

```python
from ambientor_py import engine

# Create an engine with default parameters
eng = engine(sample_rate=48_000.0, channels=2, gain=0.35)

# Render a small block of audio (interleaved floats: [L0, R0, L1, R1, ...])
block = eng.render_block(1024)
print(f"Rendered {len(block)} samples ({len(block) // 2} frames of stereo)")

# Offline render to a WAV file
eng.render_to_file("/tmp/ambientor_demo.wav", seconds=10.0)
print("Wrote /tmp/ambientor_demo.wav")
```

If you prefer to work directly with the class:
```python
from ambientor_py import AmbientorEngine

eng = AmbientorEngine(sample_rate=44_100.0, channels=2, gain=0.25)
eng.set_gain(0.5)

frames = 2048
buf = eng.render_block(frames)
# do something with `buf` (NumPy, sounddevice, etc.)
```

## Relationship to the Rust / C++ code

The stack looks like this:
```text
Python (ambientor_py)
    ↓
Rust extension (python/src/lib.rs, pyo3)
    ↓
C FFI (libambientor_ffi)
    ↓
Rust engine (ambientor-engine + ambientor-core)
```

The Python bindings call into the same FFI layer used by the C++ RtAudio host,
so the sound and behavior are consistent across:
	•	ambientor-cli (Rust)
	•	ambientor_host (C++ / RtAudio)
	•	ambientor_py (Python)

You just get a more ergonomic Python API on top.

## Development tips
	•	When you change the Rust side (e.g. src/lib.rs), re-run:
```bash
maturin develop
```
to rebuild and reinstall the extension.

	•	When you change only Python files under ambientor_py/, you can just
re-import the module in your Python session (or restart the REPL).
	•	To run the Rust unit tests for the core engine:

```bash
# from the repo root
cargo test -p ambientor-core --release
```

## License

This specific Python bindings are currently licensed under MIT. However, the project can be adjusted to match the main Ambientor project license if needed.

```

You can drop those straight into:

- `python/pyproject.toml`
- `python/README.md`

Then the next steps are:

```bash
cd python
maturin develop
python -q
>>> from ambientor_py import engine
>>> eng = engine()
>>> eng.render_to_file("/tmp/ambientor_test.wav", 5.0)
```