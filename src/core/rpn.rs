//! HP-style 4-level RPN (Reverse Polish Notation) stack engine.

use crate::core::engine::Engine;
use crate::core::history::HistoryLog;
use crate::core::register::Register;
use crate::core::types::{BinaryOp, Key, MemoryOp, StatusFlags, UnaryOp};

/// Classic HP 4-level operational stack (X, Y, Z, T).
#[derive(Debug, Clone, PartialEq)]
pub struct RpnStack {
    /// Level X: Bottom of stack (currently active on LCD display).
    pub x: f64,
    /// Level Y: Second stack level.
    pub y: f64,
    /// Level Z: Third stack level.
    pub z: f64,
    /// Level T: Top stack level (replicated on pop in classic HP RPN).
    pub t: f64,
}

impl Default for RpnStack {
    fn default() -> Self {
        Self::new()
    }
}

impl RpnStack {
    /// Creates a new zeroed 4-level operational stack.
    pub fn new() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            t: 0.0,
        }
    }

    /// Pushes a value onto the stack: T <- Z, Z <- Y, Y <- X, X <- val.
    pub fn push(&mut self, val: f64) {
        self.t = self.z;
        self.z = self.y;
        self.y = self.x;
        self.x = val;
    }

    /// Pops the X register and rolls down: X <- Y, Y <- Z, Z <- T.
    pub fn pop(&mut self) -> f64 {
        let old_x = self.x;
        self.x = self.y;
        self.y = self.z;
        self.z = self.t;
        old_x
    }

    /// Swaps the X and Y registers (X <-> Y).
    pub fn swap(&mut self) {
        std::mem::swap(&mut self.x, &mut self.y);
    }

    /// Rolls the stack downwards: X <- Y, Y <- Z, Z <- T, T <- old_X.
    pub fn roll_down(&mut self) {
        let old_x = self.x;
        self.x = self.y;
        self.y = self.z;
        self.z = self.t;
        self.t = old_x;
    }

    /// Clears the entire stack to 0.0.
    pub fn clear(&mut self) {
        self.x = 0.0;
        self.y = 0.0;
        self.z = 0.0;
        self.t = 0.0;
    }
}

/// RPN Calculator Controller.
#[derive(Debug, Clone, PartialEq)]
pub struct RpnCalculator {
    stack: RpnStack,
    register: Register,
    is_entering: bool,
    engine: Engine,
    has_error: bool,
    history: HistoryLog,
}

impl Default for RpnCalculator {
    fn default() -> Self {
        Self::new()
    }
}

impl RpnCalculator {
    /// Creates a new RPN calculator in initial state.
    pub fn new() -> Self {
        Self {
            stack: RpnStack::new(),
            register: Register::new(),
            is_entering: false,
            engine: Engine::new(),
            has_error: false,
            history: HistoryLog::default(),
        }
    }

    /// Returns the current 4-level operational stack.
    pub fn stack(&self) -> &RpnStack {
        &self.stack
    }

    /// Returns the active LCD display string (Register if typing, else stack level X).
    pub fn display_string(&self) -> String {
        if self.has_error {
            return "0".to_string();
        }

        if self.is_entering {
            self.register.display_string().to_string()
        } else if let Ok(reg) = Register::from_f64(self.stack.x) {
            reg.display_string().to_string()
        } else {
            format!("{:.8}", self.stack.x)
        }
    }

    /// Returns the LCD status flags.
    pub fn status_flags(&self) -> StatusFlags {
        let negative = if self.is_entering {
            self.register.is_negative()
        } else {
            self.stack.x < 0.0
        };

        StatusFlags {
            has_error: self.has_error,
            memory_active: self.engine.has_memory(),
            negative,
            active_operator: None,
        }
    }

    /// Returns a slice of calculation history log entries.
    pub fn history(&self) -> &[crate::core::history::HistoryEntry] {
        self.history.entries()
    }

    /// Clears the history log.
    pub fn clear_history(&mut self) {
        self.history.clear();
    }

    /// Processes an input key event in RPN mode.
    pub fn process_key(&mut self, key: Key) {
        if self.has_error {
            if matches!(key, Key::Clear) {
                self.reset();
            }
            return;
        }

        match key {
            Key::Digit(d) => {
                if !self.is_entering {
                    self.register.clear();
                    self.is_entering = true;
                }
                self.register.append_digit(d);
            }

            Key::DecimalPoint => {
                if !self.is_entering {
                    self.register.clear();
                    self.is_entering = true;
                }
                self.register.append_decimal();
            }

            Key::Equals => {
                // In RPN mode, Equals acts as ENTER (pushes X onto stack)
                self.handle_enter();
            }

            Key::BinaryOp(op) => {
                self.commit_entering_buffer();
                self.handle_binary_op(op);
            }

            Key::UnaryOp(op) => {
                self.commit_entering_buffer();
                self.handle_unary_op(op);
            }

            Key::MemoryOp(op) => {
                self.handle_memory_op(op);
            }

            Key::Clear => {
                self.reset();
            }

            Key::ClearEntry => {
                if self.is_entering {
                    self.register.clear();
                    self.is_entering = false;
                } else {
                    self.stack.x = 0.0;
                }
            }
        }
    }

    fn commit_entering_buffer(&mut self) {
        if self.is_entering {
            self.stack.x = self.register.to_f64();
            self.is_entering = false;
        }
    }

    fn handle_enter(&mut self) {
        self.commit_entering_buffer();
        // HP RPN Enter replicates X into Y and lifts the stack
        let x = self.stack.x;
        self.stack.push(x);
    }

    fn handle_binary_op(&mut self, op: BinaryOp) {
        let b = self.stack.pop(); // Pop X
        let a = self.stack.x; // Y becomes new X

        match self.engine.execute_binary(op, a, b) {
            Ok(res) => {
                self.stack.x = res;
                if let Ok(reg) = Register::from_f64(res) {
                    self.history.record(
                        format!("{} {} {} =", format_rpn_val(a), op, format_rpn_val(b)),
                        reg.display_string().to_string(),
                    );
                }
            }
            Err(_) => self.enter_error(),
        }
    }

    fn handle_unary_op(&mut self, op: UnaryOp) {
        let val = self.stack.x;
        match op {
            UnaryOp::Negate => {
                self.stack.x = -val;
            }
            UnaryOp::SquareRoot | UnaryOp::Percentage => {
                match self.engine.execute_unary(op, val, None) {
                    Ok(res) => {
                        self.stack.x = res;
                        if let Ok(reg) = Register::from_f64(res) {
                            let expr = match op {
                                UnaryOp::SquareRoot => format!("{} √ =", format_rpn_val(val)),
                                UnaryOp::Percentage => format!("{} % =", format_rpn_val(val)),
                                UnaryOp::Negate => String::new(),
                            };
                            if !expr.is_empty() {
                                self.history.record(expr, reg.display_string().to_string());
                            }
                        }
                    }
                    Err(_) => self.enter_error(),
                }
            }
        }
    }

    fn handle_memory_op(&mut self, op: MemoryOp) {
        self.commit_entering_buffer();
        let val = self.stack.x;

        match op {
            MemoryOp::Add => {
                if self.engine.memory_add(val).is_err() {
                    self.enter_error();
                }
            }
            MemoryOp::Subtract => {
                if self.engine.memory_sub(val).is_err() {
                    self.enter_error();
                }
            }
            MemoryOp::Recall => {
                let mem = self.engine.memory_recall();
                self.stack.push(mem);
            }
            MemoryOp::Clear => {
                self.engine.memory_clear();
            }
        }
    }

    /// Resets the RPN calculator state.
    pub fn reset(&mut self) {
        self.stack.clear();
        self.register.clear();
        self.is_entering = false;
        self.has_error = false;
    }

    fn enter_error(&mut self) {
        self.has_error = true;
    }
}

fn format_rpn_val(v: f64) -> String {
    if let Ok(reg) = Register::from_f64(v) {
        reg.display_string().to_string()
    } else {
        format!("{v}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rpn_addition_workflow() {
        let mut calc = RpnCalculator::new();
        // Calculate: 5 Enter 3 + = 8
        calc.process_key(Key::Digit(5));
        calc.process_key(Key::Equals); // ENTER
        assert_eq!(calc.stack().x, 5.0);
        assert_eq!(calc.stack().y, 5.0);

        calc.process_key(Key::Digit(3));
        calc.process_key(Key::BinaryOp(BinaryOp::Add));

        assert_eq!(calc.display_string(), "8");
        assert_eq!(calc.stack().x, 8.0);
    }

    #[test]
    fn test_rpn_four_level_stack_lift_and_drop() {
        let mut calc = RpnCalculator::new();
        // Push 1, 2, 3, 4
        calc.process_key(Key::Digit(1));
        calc.process_key(Key::Equals);
        calc.process_key(Key::Digit(2));
        calc.process_key(Key::Equals);
        calc.process_key(Key::Digit(3));
        calc.process_key(Key::Equals);
        calc.process_key(Key::Digit(4));
        calc.commit_entering_buffer();

        assert_eq!(calc.stack().x, 4.0);
        assert_eq!(calc.stack().y, 3.0);
        assert_eq!(calc.stack().z, 2.0);
        assert_eq!(calc.stack().t, 1.0);

        // Add 4 + 3 -> X=7, Y=2, Z=1
        calc.process_key(Key::BinaryOp(BinaryOp::Add));
        assert_eq!(calc.stack().x, 7.0);
        assert_eq!(calc.stack().y, 2.0);
        assert_eq!(calc.stack().z, 1.0);

        // Multiply 7 * 2 -> X=14, Y=1
        calc.process_key(Key::BinaryOp(BinaryOp::Multiply));
        assert_eq!(calc.stack().x, 14.0);
        assert_eq!(calc.stack().y, 1.0);

        // Subtract 14 - 1 -> X=13
        calc.process_key(Key::BinaryOp(BinaryOp::Subtract));
        assert_eq!(calc.stack().x, -13.0); // Y - X = 1 - 14 = -13
    }
}
