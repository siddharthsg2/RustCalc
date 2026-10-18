//! The calculator state machine.

use crate::format;
use crate::history::{History, HistoryEntry};
use crate::op::Op;

/// Maximum number of digits (sign and decimal point excluded) in an entry.
pub const MAX_ENTRY_DIGITS: usize = 16;

/// A calculation failure. The calculator locks until it is cleared.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CalcError {
    /// Division by zero was attempted.
    DivideByZero,
    /// The result was not a finite number.
    Overflow,
}

impl CalcError {
    /// Message shown on the display.
    pub fn message(self) -> &'static str {
        match self {
            CalcError::DivideByZero => "Cannot divide by zero",
            CalcError::Overflow => "Overflow",
        }
    }
}

/// Standard-mode calculator state machine.
#[derive(Debug)]
pub struct Calculator {
    entry: String,
    accumulator: f64,
    pending: Option<Op>,
    /// The next digit or decimal point starts a new entry.
    fresh: bool,
    /// The display shows the accumulator awaiting a right operand.
    awaiting_rhs: bool,
    last_op: Option<Op>,
    last_rhs: f64,
    last_expression: String,
    error: Option<CalcError>,
    history: History,
}

impl Default for Calculator {
    fn default() -> Self {
        Self::new()
    }
}

impl Calculator {
    /// Create a calculator showing `0` with nothing pending.
    pub fn new() -> Self {
        Self {
            entry: "0".to_string(),
            accumulator: 0.0,
            pending: None,
            fresh: false,
            awaiting_rhs: false,
            last_op: None,
            last_rhs: 0.0,
            last_expression: String::new(),
            error: None,
            history: History::default(),
        }
    }

    /// The main display line, e.g. `"1,234.5"` or an error message.
    pub fn display(&self) -> String {
        match self.error {
            Some(error) => error.message().to_string(),
            None => format::group_digits(&self.entry),
        }
    }

    /// The small expression line, e.g. `"12 +"` or `"12 + 3 ="`.
    pub fn expression(&self) -> &str {
        &self.last_expression
    }

    /// The current error, if the calculator is locked.
    pub fn error(&self) -> Option<CalcError> {
        self.error
    }

    /// Type a digit (`0..=9`).
    pub fn input_digit(&mut self, digit: u8) {
        if digit > 9 {
            return;
        }
        self.recover_from_error();
        self.last_op = None;
        self.awaiting_rhs = false;
        let ch = char::from(b'0' + digit);
        if self.fresh {
            self.entry = ch.to_string();
            self.fresh = false;
            return;
        }
        if self.entry == "0" {
            self.entry = ch.to_string();
            return;
        }
        if self.digit_count() < MAX_ENTRY_DIGITS {
            self.entry.push(ch);
        }
    }

    /// Type the decimal point.
    pub fn input_decimal(&mut self) {
        self.recover_from_error();
        self.last_op = None;
        self.awaiting_rhs = false;
        if self.fresh {
            self.entry = "0.".to_string();
            self.fresh = false;
            return;
        }
        if !self.entry.contains('.') && self.digit_count() < MAX_ENTRY_DIGITS {
            self.entry.push('.');
        }
    }

    /// Apply an operator key.
    pub fn set_op(&mut self, op: Op) {
        if self.error.is_some() {
            return;
        }
        self.last_op = None;
        if let Some(pending) = self.pending {
            if self.awaiting_rhs {
                // The user changed their mind about the operator.
            } else {
                let rhs = self.current_value();
                let expression = format!(
                    "{} {} {}",
                    format::group_digits(&format::format_number(self.accumulator)),
                    pending.symbol(),
                    format::group_digits(&format::format_number(rhs))
                );
                match pending.apply(self.accumulator, rhs) {
                    Ok(result) => {
                        self.accumulator = result;
                        self.entry = format::format_number(result);
                    }
                    Err(error) => {
                        self.error = Some(error);
                        self.pending = None;
                        self.fresh = true;
                        self.awaiting_rhs = false;
                        self.last_expression = format!("{expression} =");
                        return;
                    }
                }
            }
        } else {
            self.accumulator = self.current_value();
        }
        self.last_expression = format!(
            "{} {}",
            format::group_digits(&format::format_number(self.accumulator)),
            op.symbol()
        );
        self.pending = Some(op);
        self.fresh = true;
        self.awaiting_rhs = true;
    }

    /// Evaluate the pending operation, or repeat the last one.
    pub fn equals(&mut self) {
        if self.error.is_some() {
            return;
        }
        let op = match (self.pending, self.last_op) {
            (Some(op), _) => op,
            (None, Some(op)) => op,
            (None, None) => {
                self.fresh = true;
                return;
            }
        };
        let rhs = if self.pending.is_some() {
            if self.awaiting_rhs {
                self.accumulator
            } else {
                self.current_value()
            }
        } else {
            self.last_rhs
        };
        let expression = format!(
            "{} {} {}",
            format::group_digits(&format::format_number(self.accumulator)),
            op.symbol(),
            format::group_digits(&format::format_number(rhs))
        );
        match op.apply(self.accumulator, rhs) {
            Ok(result) => {
                let result_display = format::format_number(result);
                self.history.push(HistoryEntry {
                    expression: expression.clone(),
                    result: format::group_digits(&result_display),
                    value: result,
                });
                self.entry = result_display;
                self.accumulator = result;
                self.last_op = Some(op);
                self.last_rhs = rhs;
                self.last_expression = format!("{expression} =");
            }
            Err(error) => {
                self.error = Some(error);
                self.last_expression = format!("{expression} =");
            }
        }
        self.pending = None;
        self.fresh = true;
        self.awaiting_rhs = false;
    }

    /// Convert the current entry to a percentage (Windows-style).
    ///
    /// With a pending `+` or `-`, the entry becomes the accumulator's share:
    /// `50 + 10 %` converts `10` to `5`. Otherwise the entry is divided
    /// by 100.
    pub fn percent(&mut self) {
        if self.error.is_some() {
            return;
        }
        self.last_op = None;
        let value = self.current_value();
        let result = match self.pending {
            Some(Op::Add) | Some(Op::Subtract) => self.accumulator * value / 100.0,
            _ => value / 100.0,
        };
        self.fresh = true;
        self.awaiting_rhs = false;
        if result.is_finite() {
            self.entry = format::format_number(result);
        } else {
            self.error = Some(CalcError::Overflow);
            self.pending = None;
        }
    }

    /// Flip the sign of the current entry.
    pub fn toggle_sign(&mut self) {
        if self.error.is_some() {
            return;
        }
        self.last_op = None;
        self.awaiting_rhs = false;
        if self.entry == "0" {
            return;
        }
        match self.entry.strip_prefix('-') {
            Some(unsigned) => self.entry = unsigned.to_string(),
            None => self.entry.insert(0, '-'),
        }
    }

    /// Reset everything except the history.
    pub fn clear(&mut self) {
        self.entry = "0".to_string();
        self.accumulator = 0.0;
        self.pending = None;
        self.fresh = false;
        self.awaiting_rhs = false;
        self.last_op = None;
        self.last_rhs = 0.0;
        self.last_expression.clear();
        self.error = None;
    }

    /// Clear only the current entry, keeping any pending operation.
    pub fn clear_entry(&mut self) {
        if self.error.is_some() {
            self.clear();
            return;
        }
        self.entry = "0".to_string();
        self.fresh = false;
        self.awaiting_rhs = false;
        self.last_op = None;
    }

    /// Delete the last typed character.
    pub fn backspace(&mut self) {
        if self.error.is_some() {
            self.clear();
            return;
        }
        if self.fresh {
            return;
        }
        self.entry.pop();
        if self.entry.is_empty() || self.entry == "-" {
            self.entry = "0".to_string();
        }
    }

    /// The recorded calculations, oldest first.
    pub fn history(&self) -> &[HistoryEntry] {
        self.history.entries()
    }

    /// Forget all recorded calculations.
    pub fn clear_history(&mut self) {
        self.history.clear();
    }

    /// Replace the recorded history, e.g. after loading it from disk.
    ///
    /// At most the newest [`crate::HISTORY_CAPACITY`] entries are kept.
    pub fn import_history(&mut self, entries: Vec<HistoryEntry>) {
        self.history.replace(entries);
    }

    /// Put the result of history entry `index` into the display.
    ///
    /// Returns `false` when the index is out of range.
    pub fn recall_history(&mut self, index: usize) -> bool {
        let value = match self.history.entries().get(index) {
            Some(entry) => entry.value,
            None => return false,
        };
        if self.error.is_some() {
            self.clear();
        }
        self.entry = format::format_number(value);
        self.fresh = true;
        self.awaiting_rhs = false;
        self.last_op = None;
        true
    }

    fn recover_from_error(&mut self) {
        if self.error.is_some() {
            self.clear();
        }
    }

    fn digit_count(&self) -> usize {
        self.entry.chars().filter(char::is_ascii_digit).count()
    }

    fn current_value(&self) -> f64 {
        self.entry.parse().unwrap_or(0.0)
    }
}
