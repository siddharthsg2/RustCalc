//! Arithmetic operations.

use crate::calculator::CalcError;

/// A binary arithmetic operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Op {
    /// Addition (`+`).
    Add,
    /// Subtraction (`-`).
    Subtract,
    /// Multiplication (`×`).
    Multiply,
    /// Division (`÷`).
    Divide,
}

impl Op {
    /// Symbol used in the expression line and in history entries.
    pub fn symbol(self) -> &'static str {
        match self {
            Op::Add => "+",
            Op::Subtract => "-",
            Op::Multiply => "×",
            Op::Divide => "÷",
        }
    }

    /// Apply the operation to two finite operands.
    ///
    /// Fails with [`CalcError::DivideByZero`] when dividing by zero and with
    /// [`CalcError::Overflow`] when the result is not finite.
    pub(crate) fn apply(self, lhs: f64, rhs: f64) -> Result<f64, CalcError> {
        if self == Op::Divide && rhs == 0.0 {
            return Err(CalcError::DivideByZero);
        }
        let result = match self {
            Op::Add => lhs + rhs,
            Op::Subtract => lhs - rhs,
            Op::Multiply => lhs * rhs,
            Op::Divide => lhs / rhs,
        };
        if result.is_finite() {
            Ok(result)
        } else {
            Err(CalcError::Overflow)
        }
    }
}
