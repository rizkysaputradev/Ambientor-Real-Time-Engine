/ python/src/lib.rs

//! Python bindings for the Ambientor engine.
//!
//! This crate is built as a Python extension module using `pyo3`.
//! It talks to the C FFI layer (`libambientor_ffi`) which in turn
//! wraps the Rust `ambientor-engine` and `ambientor-core` crates.

use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use std::ffi::c_void;

// ----------------------------- FFI bridge ---------------------------------------

// Opaque engine handle as seen from C.
#[repr(C)]
pub struct AmbientorEngineOpaque {
    _private: [u8; 0],
}

type AmbientorEngineHandle = AmbientorEngineOpaque;

#[link(name = "ambientor_ffi")]
extern "C" {
    fn ambientor_create(sample_rate: f32) -> *mut AmbientorEngineHandle;
    fn ambientor_destroy(engine: *mut AmbientorEngineHandle);
    fn ambientor_scene_set_out_gain(engine: *mut AmbientorEngineHandle, gain: f32);
    fn ambientor_render_interleaved_f32(
        engine: *mut AmbientorEngineHandle,
        out: *mut f32,
        frames: u32,
        channels: u32,
    ) -> u32;
}

// ----------------------------- Helper: WAV writer -------------------------------

fn write_wav_i16(path: &str, sr: u32, channels: u16, data: &[i16]) -> std::io::Result<()> {
    use std::fs::File;
    use std::io::Write;

    let mut f = File::create(path)?;

    let bytes_per_sample: u16 = 2;
    let block_align: u16 = channels * bytes_per_sample;
    let byte_rate: u32 = sr * block_align as u32;
    let data_len_bytes: u32 = (data.len() * 2) as u32;
    let riff_chunk_size: u32 = 36 + data_len_bytes;

    // RIFF header
    f.write_all(b"RIFF")?;
    f.write_all(&riff_chunk_size.to_le_bytes())?;
    f.write_all(b"WAVE")?;

    // fmt chunk
    f.write_all(b"fmt ")?;
    f.write_all(&16u32.to_le_bytes())?; // PCM chunk size
    f.write_all(&1u16.to_le_bytes())?; // PCM format
    f.write_all(&channels.to_le_bytes())?;
    f.write_all(&sr.to_le_bytes())?;
    f.write_all(&byte_rate.to_le_bytes())?;
    f.write_all(&block_align.to_le_bytes())?;
    f.write_all(&16u16.to_le_bytes())?; // bits per sample

    // data chunk
    f.write_all(b"data")?;
    f.write_all(&data_len_bytes.to_le_bytes())?;
    for s in data {
        f.write_all(&s.to_le_bytes())?;
    }

    f.flush()?;
    Ok(())
}

// ----------------------------- Python class -------------------------------------

/// High-level Python wrapper around the Ambientor engine.
///
/// Usage from Python:
///
/// ```python
/// from ambientor_py import AmbientorEngine
///
/// eng = AmbientorEngine(sample_rate=48_000, channels=2, gain=0.35)
/// block = eng.render_block(1024)            # returns list[float]
/// eng.render_to_file("test.wav", 10.0)      # offline render
/// ```
#[pyclass]
pub struct AmbientorEngine {
    ptr: *mut AmbientorEngineHandle,
    sample_rate: f32,
    channels: u32,
}

unsafe impl Send for AmbientorEngine {}
unsafe impl Sync for AmbientorEngine {}

#[pymethods]
impl AmbientorEngine {
    /// Create a new engine instance.
    ///
    /// Args:
    ///     sample_rate (float): Sample rate in Hz (default 48000.0).
    ///     channels (int): Number of output channels (default 2).
    ///     gain (float): Output gain multiplier (default 0.35).
    #[new]
    #[pyo3(signature = (sample_rate = 48_000.0, channels = 2, gain = 0.35))]
    pub fn new(sample_rate: f32, channels: u32, gain: f32) -> PyResult<Self> {
        if channels == 0 {
            return Err(PyRuntimeError::new_err("channels must be >= 1"));
        }

        let ptr = unsafe { ambientor_create(sample_rate) };
        if ptr.is_null() {
            return Err(PyRuntimeError::new_err(
                "ambientor_create() returned null pointer",
            ));
        }

        unsafe {
            ambientor_scene_set_out_gain(ptr, gain);
        }

        Ok(Self {
            ptr,
            sample_rate,
            channels,
        })
    }

    /// Get the current sample rate.
    #[getter]
    pub fn sample_rate(&self) -> f32 {
        self.sample_rate
    }

    /// Get the number of output channels.
    #[getter]
    pub fn num_channels(&self) -> u32 {
        self.channels
    }

    /// Set the output gain (linear, typical range 0.0–1.0).
    pub fn set_gain(&mut self, gain: f32) {
        unsafe {
            ambientor_scene_set_out_gain(self.ptr, gain);
        }
    }

    /// Render a block of audio and return it as a Python list of floats
    /// in interleaved [L0, R0, L1, R1, ...] format.
    pub fn render_block<'py>(&mut self, py: Python<'py>, frames: usize) -> PyResult<&'py PyAny> {
        if frames == 0 {
            return Ok(pyo3::types::PyList::empty(py));
        }

        let total_samples = frames * self.channels as usize;
        let mut buf = vec![0.0f32; total_samples];

        let written = unsafe {
            ambientor_render_interleaved_f32(
                self.ptr,
                buf.as_mut_ptr(),
                frames as u32,
                self.channels,
            )
        };

        let used_frames = (written as usize).min(frames);
        let used_samples = used_frames * self.channels as usize;
        buf.truncate(used_samples);

        Ok(pyo3::types::PyList::new(py, &buf))
    }

    /// Offline render straight to a 16-bit PCM WAV file.
    ///
    /// Args:
    ///     path (str): Output path for the WAV file.
    ///     seconds (float): Duration in seconds (must be > 0).
    pub fn render_to_file(&mut self, path: &str, seconds: f32) -> PyResult<()> {
        if seconds <= 0.0 {
            return Err(PyRuntimeError::new_err(
                "seconds must be positive for render_to_file()",
            ));
        }

        let total_frames = (self.sample_rate * seconds).round() as usize;
        let block_size: usize = 1024;
        let mut remaining = total_frames;

        let mut tmp = vec![0.0f32; block_size * self.channels as usize];
        let mut pcm: Vec<i16> = Vec::with_capacity(total_frames * self.channels as usize);

        while remaining > 0 {
            let frames = remaining.min(block_size);
            let written = unsafe {
                ambientor_render_interleaved_f32(
                    self.ptr,
                    tmp.as_mut_ptr(),
                    frames as u32,
                    self.channels,
                )
            } as usize;

            if written == 0 {
                break;
            }

            let used_samples = written * self.channels as usize;

            for &s in &tmp[..used_samples] {
                let x = s.clamp(-1.0, 1.0);
                let q = (x * i16::MAX as f32) as i16;
                pcm.push(q);
            }

            remaining -= written;
        }

        write_wav_i16(path, self.sample_rate as u32, self.channels as u16, &pcm)
            .map_err(|e| PyRuntimeError::new_err(format!("write_wav_i16 failed: {e}")))?;

        Ok(())
    }
}

impl Drop for AmbientorEngine {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe {
                ambientor_destroy(self.ptr);
            }
            self.ptr = std::ptr::null_mut();
        }
    }
}

// ----------------------------- Module init ---------------------------------------

/// Python module definition.
///
/// The compiled extension will be imported as `ambientor_py._ambientor`
/// and re-exported from the pure-Python `ambientor_py` package.
#[pymodule]
fn _ambientor(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<AmbientorEngine>()?;
    Ok(())
}