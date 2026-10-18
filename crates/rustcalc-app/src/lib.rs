#![forbid(unsafe_code)]
//! RustCalc GUI shell.
//!
//! This crate renders the calculator and forwards input to `rustcalc-core`.
//! All calculator logic lives in `rustcalc_core`; this crate is a thin shell.

pub mod app;
pub mod icon;
mod icon_pixels;
pub mod theme;
