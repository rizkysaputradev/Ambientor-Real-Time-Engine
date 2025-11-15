# python/ambientor_py/__init__.py

"""
ambientor_py
============

High-level Python bindings for the Ambientor real-time ambient engine.

This package provides:

- :class:`AmbientorEngine` — Python wrapper around the native Ambientor engine.
- :func:`engine` — convenience constructor.

Typical usage
-------------

.. code-block:: python

    from ambientor_py import engine

    eng = engine(sample_rate=48_000, channels=2, gain=0.35)
    block = eng.render_block(1024)       # list[float], interleaved
    eng.render_to_file("demo.wav", 10.0) # offline render

"""

from __future__ import annotations

from ._version import __version__

try:
    # Native extension (built from src/lib.rs via pyo3)
    from . import _ambientor as _core
except Exception as exc:  # pragma: no cover - import-time failure path
    _core = None
    _IMPORT_ERROR = exc
else:  # pragma: no cover - trivial branch
    _IMPORT_ERROR = None

if _core is not None:
    AmbientorEngine = _core.AmbientorEngine  # type: ignore[attr-defined]
    __all__ = ["AmbientorEngine", "engine", "__version__"]
else:
    # Extension is missing – user can still introspect metadata.
    __all__ = ["engine", "__version__"]


def engine(
    sample_rate: float = 48_000.0,
    channels: int = 2,
    gain: float = 0.35,
):
    """
    Convenience factory for :class:`AmbientorEngine`.

    Parameters
    ----------
    sample_rate:
        Sample rate in Hz. Default: 48000.0
    channels:
        Number of output channels. Default: 2
    gain:
        Linear output gain multiplier. Typical range 0.0–1.0. Default: 0.35

    Returns
    -------
    AmbientorEngine

    Raises
    ------
    ImportError
        If the native extension could not be imported (e.g. not built yet).
    """
    if _core is None:
        raise ImportError(
            "ambientor_py native extension is not available.\n"
            "Make sure the Rust extension is built (e.g. `pip install .` "
            "or `maturin develop`) before importing `ambientor_py.engine()`."
        ) from _IMPORT_ERROR

    return AmbientorEngine(
        sample_rate=sample_rate,
        channels=channels,
        gain=gain,
    )