#!/usr/bin/env bash
#
# build_all.sh — Top-level build helper for the Ambientor project.
#
# Responsibilities:
#   - Build Rust crates (ambientor-core, ambientor-engine, ambientor-cli)
#   - Build C++ RtAudio host (cpp/ambientor_host)
#   - Build Python bindings (python/ambientor-py) via maturin
#
# Usage:
#   ./scripts/build_all.sh [options]
#
# Options:
#   --debug           Build in debug mode (default: release)
#   --release         Build in release mode (default)
#   --no-rust         Skip Rust build
#   --no-cpp          Skip C++ build
#   --no-python       Skip Python build
#   --no-tests        Skip Rust unit tests
#   --python-develop  Use `maturin develop` (default)
#   --python-wheel    Build a wheel with `maturin build` instead of develop
#   -h, --help        Show this help message
#

set -euo pipefail

# --------------------------- Helper / argument parsing ---------------------------

usage() {
  sed -n '1,50p' "$0" | sed 's/^# \{0,1\}//'
  exit 1
}

BUILD_MODE="release"
DO_RUST=1
DO_CPP=1
DO_PY=1
DO_TESTS=1
PY_MODE="develop"  # or "wheel"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --debug)
      BUILD_MODE="debug"
      shift
      ;;
    --release)
      BUILD_MODE="release"
      shift
      ;;
    --no-rust)
      DO_RUST=0
      shift
      ;;
    --no-cpp)
      DO_CPP=0
      shift
      ;;
    --no-python)
      DO_PY=0
      shift
      ;;
    --no-tests)
      DO_TESTS=0
      shift
      ;;
    --python-develop)
      PY_MODE="develop"
      shift
      ;;
    --python-wheel)
      PY_MODE="wheel"
      shift
      ;;
    -h|--help)
      usage
      ;;
    *)
      echo "[WARN] Unknown argument: $1"
      shift
      ;;
  esac
done

# --------------------------- Locate project directories --------------------------

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"

RUST_DIR="${ROOT_DIR}/rust"
CPP_DIR="${ROOT_DIR}/cpp"
PY_DIR="${ROOT_DIR}/python"

echo "[INFO] Ambientor build helper"
echo "  ROOT_DIR : ${ROOT_DIR}"
echo "  BUILD_MODE: ${BUILD_MODE}"
echo "  RUST   : $( ((DO_RUST)) && echo 'ON' || echo 'OFF')"
echo "  C++    : $( ((DO_CPP)) && echo 'ON' || echo 'OFF')"
echo "  Python : $( ((DO_PY)) && echo 'ON' || echo 'OFF')"
echo "  Tests  : $( ((DO_TESTS)) && echo 'ON' || echo 'OFF')"
echo "  Python mode: ${PY_MODE}"
echo

# --------------------------- Rust build ------------------------------------------

build_rust() {
  echo "===================== [1/3] Building Rust crates ====================="
  cd "${RUST_DIR}"

  if [[ "${BUILD_MODE}" == "release" ]]; then
    echo "[RUST] cargo build --release -p ambientor-core -p ambientor-engine -p ambientor-cli"
    cargo build --release -p ambientor-core -p ambientor-engine -p ambientor-cli
  else
    echo "[RUST] cargo build -p ambientor-core -p ambientor-engine -p ambientor-cli"
    cargo build -p ambientor-core -p ambientor-engine -p ambientor-cli
  fi

  if (( DO_TESTS )); then
    echo
    echo "[RUST] Running ambientor-core tests (${BUILD_MODE})…"
    if [[ "${BUILD_MODE}" == "release" ]]; then
      cargo test -p ambientor-core --release
    else
      cargo test -p ambientor-core
    fi
  else
    echo "[RUST] Skipping tests (per --no-tests)."
  fi

  echo "======================================================================"
  echo
}

# --------------------------- C++ build (RtAudio host) ---------------------------

build_cpp() {
  echo "===================== [2/3] Building C++ host ========================"
  cd "${CPP_DIR}"

  BUILD_SUBDIR="build"
  mkdir -p "${BUILD_SUBDIR}"
  cd "${BUILD_SUBDIR}"

  # Prefer Ninja if available, otherwise fall back to Makefiles
  if command -v ninja >/dev/null 2>&1; then
    GENERATOR="-G Ninja"
    BUILD_CMD="ninja"
  else
    GENERATOR=""
    BUILD_CMD="cmake --build ."
  fi

  echo "[C++] Configuring CMake…"
  if [[ "${BUILD_MODE}" == "release" ]]; then
    cmake .. ${GENERATOR} -DCMAKE_BUILD_TYPE=Release
  else
    cmake .. ${GENERATOR} -DCMAKE_BUILD_TYPE=Debug
  fi

  echo "[C++] Building targets (ambientor_host, ambientor_example)…"
  if [[ "${BUILD_MODE}" == "release" ]]; then
    ${BUILD_CMD}
  else
    ${BUILD_CMD}
  fi

  echo "======================================================================"
  echo
}

# --------------------------- Python build (maturin) -----------------------------

build_python() {
  echo "===================== [3/3] Building Python bindings ================="
  cd "${PY_DIR}"

  if ! command -v maturin >/dev/null 2>&1; then
    echo "[WARN] 'maturin' not found in PATH."
    echo "       To build ambientor-py, install maturin in your venv:"
    echo "         pip install maturin"
    echo "       Skipping Python build."
    echo "======================================================================"
    echo
    return
  fi

  if [[ "${PY_MODE}" == "develop" ]]; then
    echo "[PY] maturin develop (install into current environment)…"
    maturin develop
  else
    echo "[PY] maturin build (build wheel in target/wheels)…"
    maturin build
  fi

  echo "======================================================================"
  echo
}

# --------------------------- Main driver ----------------------------------------

if (( DO_RUST )); then
  build_rust
else
  echo "[INFO] Skipping Rust build (--no-rust)."
fi

if (( DO_CPP )); then
  build_cpp
else
  echo "[INFO] Skipping C++ build (--no-cpp)."
fi

if (( DO_PY )); then
  build_python
else
  echo "[INFO] Skipping Python build (--no-python)."
fi

echo "[OK] Build pipeline finished."