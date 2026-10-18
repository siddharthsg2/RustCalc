#![forbid(unsafe_code)]
//! Pure calculator logic for RustCalc.
//!
//! This crate contains the calculator state machine, calculation history and
//! display formatting. It has no GUI dependencies and is fully unit-testable.

/// Product name shown in window titles and help output.
pub const APP_NAME: &str = "RustCalc";
