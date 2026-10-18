#![forbid(unsafe_code)]
//! Pure calculator logic for RustCalc.
//!
//! This crate contains the calculator state machine, calculation history and
//! display formatting. It has no GUI dependencies and is fully unit-testable.

mod calculator;
mod format;
mod history;
mod op;

pub use calculator::{CalcError, Calculator, MAX_ENTRY_DIGITS};
pub use format::format_number;
pub use history::{HISTORY_CAPACITY, HistoryEntry, deserialize_history, serialize_history};
pub use op::Op;

/// Product name shown in window titles and help output.
pub const APP_NAME: &str = "RustCalc";
