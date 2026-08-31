//! State representations for the pocket calculator FSM.

use crate::core::register::Register;
use crate::core::types::BinaryOp;

/// Internal state variants of the calculator finite state machine.
#[derive(Debug, Clone, PartialEq)]
pub enum CalculatorState {
    /// Initialized state with zero on display.
    Ready,

    /// User is actively typing the first operand.
    EnteringOperand1 { register: Register },

    /// A binary operator has been chosen; waiting for the second operand.
    OperatorPending {
        accumulator: f64,
        operator: BinaryOp,
    },

    /// User is actively typing the second operand.
    EnteringOperand2 {
        accumulator: f64,
        operator: BinaryOp,
        register: Register,
    },

    /// A calculation result is displayed.
    ResultDisplayed {
        register: Register,
        /// Last executed operation and operand for repeat equals functionality (e.g., `5 + 3 = 8 = 11 = 14`).
        last_operation: Option<(BinaryOp, f64)>,
    },

    /// An arithmetic error or overflow has occurred.
    Error,
}
