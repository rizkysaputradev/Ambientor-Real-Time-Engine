#!/usr/bin/env bash
#
# fmt_all.sh — Format all source code in the Ambientor repository.
#
# Responsibilities:
#   - Run `cargo fmt` on Rust crates
#   - Run `clang-format` on C/C++ sources/headers (if available)
#   - Run `black` + `ruff --fix` on Python sources (if available)
#   - Run `shfmt` on shell scripts (if available)
#
# Usage:
#   ./scripts/fmt_all.sh
#
# Optional environment variables:
#   FMT_RUST=0     → skip Rust formatting
#   FMT_CPP=0      → skip C++ formatting
#   FMT_PY=0       → skip Python formatting
#   FMT_SHELL=0    → skip shell script formatting
#

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"

RUST_DIR="${ROOT_DIR}/rust"
CPP_DIR="${ROOT_DIR}/cpp"
PY_DIR="${ROOT_DIR}/python"

FMT_RUST="${FMT_RUST:-1}"
FMT_CPP="${FMT_CPP:-1}"
FMT_PY="${FMT_PY:-1}"
FMT_SHELL="${FMT_SHELL:-1}"

echo "[INFO] Ambientor formatter"
echo "  ROOT_DIR  : ${ROOT_DIR}"
echo "  Rust      : ${FMT_RUST}"
echo "  C++       : ${FMT_CPP}"
echo "  Python    : ${FMT_PY}"
echo "  Shell     : ${FMT_SHELL}"
echo

# -------------------------------- Rust formatting --------------------------------

fmt_rust() {
  echo "====================== [RUST] cargo fmt =============================="
  if ! command -v cargo >/dev/null 2>&1; then
    echo "[WARN] cargo not found; skipping Rust formatting."
    echo "======================================================================"
    echo
    return
  fi

  cd "${RUST_DIR}"
  cargo fmt --all
  echo "======================================================================"
  echo
}

# -------------------------------- C++ formatting -------------------------------

fmt_cpp() {
  echo "====================== [C++] clang-format ============================"

  if ! command -v clang-format >/dev/null 2>&1; then
    echo "[WARN] clang-format not found; skipping C++ formatting."
    echo "       Install via Homebrew:  brew install clang-format"
    echo "======================================================================"
    echo
    return
  fi

  cd "${CPP_DIR}"

  # Format .cpp, .cc, .c, .hpp, .hh, .h in cpp/
  mapfile -t FILES < <(find . -type f \( \
      -name '*.cpp' -o -name '*.cc' -o -name '*.c' -o \
      -name '*.hpp' -o -name '*.hh' -o -name '*.h' \
    \))

  if [[ "${#FILES[@]}" -eq 0 ]]; then
    echo "[C++] No C/C++ source files found under cpp/."
  else
    echo "[C++] Running clang-format on ${#FILES[@]} files…"
    clang-format -i "${FILES[@]}"
  fi

  echo "======================================================================"
  echo
}

# -------------------------------- Python formatting -----------------------------

fmt_python() {
  echo "====================== [PYTHON] black / ruff ========================="

  cd "${ROOT_DIR}"

  # Prefer venv if present
  if [[ -d ".venv" ]]; then
    PY_BIN="./.venv/bin/python"
    BLACK_BIN="./.venv/bin/black"
    RUFF_BIN="./.venv/bin/ruff"
  else
    PY_BIN="python3"
    BLACK_BIN="black"
    RUFF_BIN="ruff"
  fi

  if ! command -v "${BLACK_BIN}" >/dev/null 2>&1; then
    echo "[WARN] black not found; skipping Python formatting."
    echo "       (Install in your venv: pip install black)"
  else
    echo "[PY] Running black on python/ and any top-level *.py…"
    "${BLACK_BIN}" python ambientor_py.py 2>/dev/null || "${BLACK_BIN}" python
  fi

  if ! command -v "${RUFF_BIN}" >/dev/null 2>&1; then
    echo "[WARN] ruff not found; skipping ruff lint/format."
    echo "       (Install in your venv: pip install ruff)"
  else
    echo "[PY] Running ruff --fix on python/"
    "${RUFF_BIN}" check --fix python || true
  fi

  echo "======================================================================"
  echo
}

# -------------------------------- Shell formatting ------------------------------

fmt_shell() {
  echo "====================== [SHELL] shfmt ================================="

  if ! command -v shfmt >/dev/null 2>&1; then
    echo "[WARN] shfmt not found; skipping shell formatting."
    echo "       Install via Homebrew:  brew install shfmt"
    echo "======================================================================"
    echo
    return
  fi

  cd "${ROOT_DIR}"

  mapfile -t SHFILES < <(find . -type f \( \
      -name '*.sh' -o \
      -name '*.env' \
    \))

  if [[ "${#SHFILES[@]}" -eq 0 ]]; then
    echo "[SHELL] No shell-like files found."
  else
    echo "[SHELL] Running shfmt on ${#SHFILES[@]} files…"
    shfmt -w "${SHFILES[@]}"
  fi

  echo "======================================================================"
  echo
}

# -------------------------------- Main ------------------------------------------

if [[ "${FMT_RUST}" == "1" ]]; then
  fmt_rust
else
  echo "[INFO] Skipping Rust formatting (FMT_RUST=0)."
fi

if [[ "${FMT_CPP}" == "1" ]]; then
  fmt_cpp
else
  echo "[INFO] Skipping C++ formatting (FMT_CPP=0)."
fi

if [[ "${FMT_PY}" == "1" ]]; then
  fmt_python
else
  echo "[INFO] Skipping Python formatting (FMT_PY=0)."
fi

if [[ "${FMT_SHELL}" == "1" ]]; then
  fmt_shell
else
  echo "[INFO] Skipping shell formatting (FMT_SHELL=0)."
fi

echo "[OK] Formatting pass complete."