#!/usr/bin/env bash
#
# dev_setup.sh — One-time / occasional development environment setup for Ambientor.
#
# Responsibilities:
#   - Verify core tools: git, rustup/cargo, python3, pip
#   - Optionally install a Rust toolchain via rustup
#   - Set up a Python virtualenv (./.venv) at the repo root if missing
#   - Install Python dev tools: maturin, black, ruff, mypy (optional but nice)
#
# Usage:
#   ./scripts/dev_setup.sh [options]
#
# Options:
#   --no-rust          Skip Rust toolchain checks/installation
#   --no-python        Skip Python venv + tooling setup
#   --venv-name NAME   Use a different venv directory name (default: .venv)
#   -h, --help         Show this help
#

set -euo pipefail

usage() {
  sed -n '1,60p' "$0" | sed 's/^# \{0,1\}//'
  exit 1
}

DO_RUST=1
DO_PY=1
VENV_NAME=".venv"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --no-rust)
      DO_RUST=0
      shift
      ;;
    --no-python)
      DO_PY=0
      shift
      ;;
    --venv-name)
      VENV_NAME="${2:-.venv}"
      shift 2
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

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"

echo "[INFO] Ambientor dev setup"
echo "  ROOT_DIR : ${ROOT_DIR}"
echo "  Rust     : $( ((DO_RUST)) && echo 'ON' || echo 'OFF')"
echo "  Python   : $( ((DO_PY)) && echo 'ON' || echo 'OFF')"
echo "  Venv     : ${VENV_NAME}"
echo

# -------------------------------- Rust setup -------------------------------------

setup_rust() {
  echo "====================== [RUST] Toolchain check ========================"

  if ! command -v git >/dev/null 2>&1; then
    echo "[ERR] git not found in PATH. Please install git first."
    exit 1
  fi

  if command -v cargo >/dev/null 2>&1; then
    echo "[OK] cargo is available."
  else
    echo "[WARN] 'cargo' not found. Attempting to install Rust via rustup…"
    if command -v rustup >/dev/null 2>&1; then
      echo "[INFO] rustup is present. Installing stable toolchain…"
      rustup toolchain install stable
      rustup default stable
    else
      echo "[ERR] rustup not found. Install Rust via:"
      echo "  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
      echo "Then re-run this script."
      exit 1
    fi
  fi

  echo "[INFO] Current Rust toolchain:"
  rustc --version || true
  cargo --version || true

  echo "======================================================================"
  echo
}

# ------------------------------ Python / venv setup ------------------------------

setup_python() {
  echo "====================== [PYTHON] Environment setup ===================="

  if ! command -v python3 >/dev/null 2>&1; then
    echo "[ERR] python3 not found in PATH. Please install Python 3.9+."
    exit 1
  fi

  if ! command -v pip3 >/dev/null 2>&1; then
    echo "[WARN] pip3 not found. Trying 'python3 -m pip' anyway…"
  fi

  cd "${ROOT_DIR}"

  if [[ ! -d "${VENV_NAME}" ]]; then
    echo "[PY] Creating virtual environment: ${VENV_NAME}"
    python3 -m venv "${VENV_NAME}"
  else
    echo "[PY] Reusing existing virtual environment: ${VENV_NAME}"
  fi

  # Activate venv
  # shellcheck disable=SC1090
  source "${VENV_NAME}/bin/activate"

  echo "[PY] Upgrading pip in venv…"
  python -m pip install --upgrade pip

  echo "[PY] Installing development tools (maturin, black, ruff, mypy)…"
  python -m pip install \
    maturin \
    black \
    ruff \
    mypy

  echo "[PY] Python info:"
  python -c "import sys; print('Python', sys.version)"
  echo "Virtualenv: $(which python)"

  echo "======================================================================"
  echo
}

# -------------------------------- Main ------------------------------------------

if (( DO_RUST )); then
  setup_rust
else
  echo "[INFO] Skipping Rust setup (--no-rust)."
fi

if (( DO_PY )); then
  setup_python
else
  echo "[INFO] Skipping Python setup (--no-python)."
fi

echo "[OK] Development environment setup complete."
echo "Tip: activate your venv with:"
echo "  source ${ROOT_DIR}/${VENV_NAME}/bin/activate"