//! Ambientor Engine — graph + building blocks + scenes.
//!
//! Crate layout:
//! - [`graph`]  : `Generator` trait and `Engine<G>` wrapper
//! - [`nodes`]  : oscillators, modulators, utility DSP nodes
//! - [`reverb`] : lightweight reverbs/diffusers (implemented separately)
//! - [`scenes`] : musical scene graphs that implement `Generator`
//!
//! The engine deliberately avoids heap allocations in the audio thread.
//! Scenes are plain structs; parameters are simple floats with optional
//! per-sample smoothing.

pub mod graph;
pub mod nodes;
pub mod reverb;
pub mod scenes;

// Re-export some commonly used items to make downstream imports ergonomic.
pub use graph::{Engine, Generator};
pub use nodes::{NoiseMod, Osc, Wave, Lfo, Mix2, PanLaw, OnePoleSmoother};
