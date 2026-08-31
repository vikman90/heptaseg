//! Finite State Machine controller for the Heptaseg calculator.

use crate::core::engine::Engine;
use crate::core::history::HistoryLog;
use crate::core::register::Register;
use crate::core::rpn::RpnStack;
use crate::core::state::CalculatorState;
use crate::core::types::{BinaryOp, CalculatorMode, Key, MemoryOp, StatusFlags, UnaryOp};

/// Central calculator finite state machine.
#[derive(Debug, Clone, PartialEq)]
pub struct CalculatorFsm {
    state: CalculatorState,
    engine: Engine,
    history: HistoryLog,
    mode: CalculatorMode,
    rpn_stack: RpnStack,
    rpn_enter_pressed: bool,
    has_error: bool,
}

impl Default for CalculatorFsm {
    fn default() -> Self {
        Self::new()
    }
}

impl CalculatorFsm {
    /// Creates a new calculator FSM in the default `Ready` state.
    pub fn new() -> Self {
        Self {
            state: CalculatorState::Ready,
            engine: Engine::new(),
            history: HistoryLog::default(),
            mode: CalculatorMode::Standard,
            rpn_stack: RpnStack::new(),
            rpn_enter_pressed: false,
            has_error: false,
        }
    }

    /// Returns the active calculator mode (Standard or Rpn).
    pub fn mode(&self) -> CalculatorMode {
        self.mode
    }

    /// Sets the operational mode.
    pub fn set_mode(&mut self, mode: CalculatorMode) {
        self.mode = mode;
        self.reset();
    }

    /// Toggles between Standard and RPN modes.
    pub fn toggle_mode(&mut self) -> CalculatorMode {
        self.mode = match self.mode {
            CalculatorMode::Standard => CalculatorMode::Rpn,
            CalculatorMode::Rpn => CalculatorMode::Standard,
        };
        self.reset();
        self.mode
    }

    /// Returns a reference to the RPN stack registers.
    pub fn rpn_stack(&self) -> &RpnStack {
        &self.rpn_stack
    }

    /// Returns a reference to the recorded calculation history.
    pub fn history(&self) -> &HistoryLog {
        &self.history
    }

    /// Clears the recorded calculation history.
    pub fn clear_history(&mut self) {
        self.history.clear();
    }

    /// Processes an input key event and transitions the state machine accordingly.
    pub fn process_key(&mut self, key: Key) {
        // If in error state, only Clear (AC) can unlock the calculator
        if self.has_error {
            if matches!(key, Key::Clear) {
                self.reset();
            }
            return;
        }

        if self.mode == CalculatorMode::Rpn {
            self.process_rpn_key(key);
            return;
        }

        match key {
            Key::Clear => {
                self.reset();
            }

            Key::ClearEntry => {
                self.handle_clear_entry();
            }

            Key::Digit(d) => {
                self.handle_digit(d);
            }

            Key::DecimalPoint => {
                self.handle_decimal();
            }

            Key::BinaryOp(op) => {
                self.handle_binary_op(op);
            }

            Key::UnaryOp(op) => {
                self.handle_unary_op(op);
            }

            Key::MemoryOp(op) => {
                self.handle_memory_op(op);
            }

            Key::Equals => {
                self.handle_equals();
            }
        }
    }

    fn current_display_val(&self) -> f64 {
        match &self.state {
            CalculatorState::Ready => 0.0,
            CalculatorState::EnteringOperand1 { register }
            | CalculatorState::EnteringOperand2 { register, .. }
            | CalculatorState::ResultDisplayed { register, .. } => register.to_f64(),
            CalculatorState::OperatorPending { accumulator, .. } => *accumulator,
            CalculatorState::Error => 0.0,
        }
    }

    fn process_rpn_key(&mut self, key: Key) {
        match key {
            Key::Clear => {
                self.reset();
                self.rpn_stack.clear();
                self.rpn_enter_pressed = false;
            }
            Key::ClearEntry => {
                self.state = CalculatorState::Ready;
                self.rpn_stack.x = 0.0;
            }
            Key::Digit(d) => {
                if self.rpn_enter_pressed {
                    let mut reg = Register::new();
                    reg.append_digit(d);
                    self.rpn_stack.x = reg.to_f64();
                    self.state = CalculatorState::EnteringOperand1 { register: reg };
                    self.rpn_enter_pressed = false;
                } else {
                    match &mut self.state {
                        CalculatorState::EnteringOperand1 { register } => {
                            register.append_digit(d);
                            self.rpn_stack.x = register.to_f64();
                        }
                        _ => {
                            let mut reg = Register::new();
                            reg.append_digit(d);
                            self.rpn_stack.x = reg.to_f64();
                            self.state = CalculatorState::EnteringOperand1 { register: reg };
                        }
                    }
                }
            }
            Key::DecimalPoint => {
                if self.rpn_enter_pressed {
                    let mut reg = Register::new();
                    reg.append_decimal();
                    self.rpn_stack.x = reg.to_f64();
                    self.state = CalculatorState::EnteringOperand1 { register: reg };
                    self.rpn_enter_pressed = false;
                } else {
                    match &mut self.state {
                        CalculatorState::EnteringOperand1 { register } => {
                            register.append_decimal();
                            self.rpn_stack.x = register.to_f64();
                        }
                        _ => {
                            let mut reg = Register::new();
                            reg.append_decimal();
                            self.rpn_stack.x = reg.to_f64();
                            self.state = CalculatorState::EnteringOperand1 { register: reg };
                        }
                    }
                }
            }
            Key::Equals => {
                // Enter key in RPN: pushes X onto the stack
                let current_val = self.current_display_val();
                self.rpn_stack.x = current_val;
                self.rpn_stack.push_enter();
                self.rpn_enter_pressed = true;
                if let Ok(reg) = Register::from_f64(current_val) {
                    self.state = CalculatorState::ResultDisplayed {
                        register: reg,
                        last_operation: None,
                    };
                }
            }
            Key::BinaryOp(op) => {
                let y = self.rpn_stack.y;
                let x = self.current_display_val();
                self.rpn_stack.x = x;
                match self
                    .rpn_stack
                    .execute_binary(|y, x| self.engine.execute_binary(op, y, x))
                {
                    Ok(res) => {
                        self.history.record_binary(y, op, x, res);
                        match Register::from_f64(res) {
                            Ok(reg) => {
                                self.state = CalculatorState::ResultDisplayed {
                                    register: reg,
                                    last_operation: None,
                                };
                                self.rpn_enter_pressed = true;
                            }
                            Err(_) => self.enter_error(),
                        }
                    }
                    Err(_) => self.enter_error(),
                }
            }
            Key::UnaryOp(op) => {
                let x = self.current_display_val();
                self.rpn_stack.x = x;
                if let UnaryOp::Negate = op {
                    match &mut self.state {
                        CalculatorState::EnteringOperand1 { register }
                        | CalculatorState::ResultDisplayed { register, .. } => {
                            register.toggle_sign();
                            self.rpn_stack.x = register.to_f64();
                        }
                        _ => {
                            let mut reg = Register::new();
                            reg.toggle_sign();
                            self.rpn_stack.x = reg.to_f64();
                            self.state = CalculatorState::EnteringOperand1 { register: reg };
                        }
                    }
                } else {
                    match self.engine.execute_unary(op, x, None) {
                        Ok(res) => {
                            self.history.record_unary(x, op, res);
                            self.rpn_stack.x = res;
                            match Register::from_f64(res) {
                                Ok(reg) => {
                                    self.state = CalculatorState::ResultDisplayed {
                                        register: reg,
                                        last_operation: None,
                                    };
                                    self.rpn_enter_pressed = true;
                                }
                                Err(_) => self.enter_error(),
                            }
                        }
                        Err(_) => self.enter_error(),
                    }
                }
            }
            Key::MemoryOp(op) => {
                self.handle_memory_op(op);
            }
        }
    }

    /// Returns the active string to show on the LCD digit cells.
    pub fn display_string(&self) -> String {
        match &self.state {
            CalculatorState::Ready => "0".to_string(),
            CalculatorState::EnteringOperand1 { register } => register.display_string().to_string(),
            CalculatorState::OperatorPending { accumulator, .. } => {
                Register::from_f64(*accumulator)
                    .map(|r| r.display_string().to_string())
                    .unwrap_or_else(|_| "0".to_string())
            }
            CalculatorState::EnteringOperand2 { register, .. } => {
                register.display_string().to_string()
            }
            CalculatorState::ResultDisplayed { register, .. } => {
                register.display_string().to_string()
            }
            CalculatorState::Error => "0".to_string(),
        }
    }

    /// Returns the current status flags for LCD annunciators (M, -, ERR, RPN, active operator).
    pub fn status_flags(&self) -> StatusFlags {
        let (negative, active_op) = match &self.state {
            CalculatorState::Ready => (false, None),
            CalculatorState::EnteringOperand1 { register } => (register.is_negative(), None),
            CalculatorState::OperatorPending {
                accumulator,
                operator,
            } => (*accumulator < 0.0, Some(*operator)),
            CalculatorState::EnteringOperand2 {
                register, operator, ..
            } => (register.is_negative(), Some(*operator)),
            CalculatorState::ResultDisplayed { register, .. } => (register.is_negative(), None),
            CalculatorState::Error => (false, None),
        };

        StatusFlags {
            has_error: self.has_error,
            memory_active: self.engine.has_memory(),
            negative,
            is_rpn: self.mode == CalculatorMode::Rpn,
            active_operator: active_op,
        }
    }

    /// Returns the current state enum.
    pub fn state(&self) -> &CalculatorState {
        &self.state
    }

    /// Resets the calculator to the initial `Ready` state (retaining memory).
    pub fn reset(&mut self) {
        self.state = CalculatorState::Ready;
        self.has_error = false;
        self.rpn_enter_pressed = false;
    }

    fn enter_error(&mut self) {
        self.has_error = true;
        self.state = CalculatorState::Error;
    }

    fn handle_digit(&mut self, d: u8) {
        match &mut self.state {
            CalculatorState::Ready | CalculatorState::ResultDisplayed { .. } => {
                let mut reg = Register::new();
                reg.append_digit(d);
                self.state = CalculatorState::EnteringOperand1 { register: reg };
            }
            CalculatorState::EnteringOperand1 { register } => {
                register.append_digit(d);
            }
            CalculatorState::OperatorPending {
                accumulator,
                operator,
            } => {
                let acc = *accumulator;
                let op = *operator;
                let mut reg = Register::new();
                reg.append_digit(d);
                self.state = CalculatorState::EnteringOperand2 {
                    accumulator: acc,
                    operator: op,
                    register: reg,
                };
            }
            CalculatorState::EnteringOperand2 { register, .. } => {
                register.append_digit(d);
            }
            CalculatorState::Error => {}
        }
    }

    fn handle_decimal(&mut self) {
        match &mut self.state {
            CalculatorState::Ready | CalculatorState::ResultDisplayed { .. } => {
                let mut reg = Register::new();
                reg.append_decimal();
                self.state = CalculatorState::EnteringOperand1 { register: reg };
            }
            CalculatorState::EnteringOperand1 { register } => {
                register.append_decimal();
            }
            CalculatorState::OperatorPending {
                accumulator,
                operator,
            } => {
                let acc = *accumulator;
                let op = *operator;
                let mut reg = Register::new();
                reg.append_decimal();
                self.state = CalculatorState::EnteringOperand2 {
                    accumulator: acc,
                    operator: op,
                    register: reg,
                };
            }
            CalculatorState::EnteringOperand2 { register, .. } => {
                register.append_decimal();
            }
            CalculatorState::Error => {}
        }
    }

    fn handle_binary_op(&mut self, op: BinaryOp) {
        match &self.state {
            CalculatorState::Ready => {
                self.state = CalculatorState::OperatorPending {
                    accumulator: 0.0,
                    operator: op,
                };
            }
            CalculatorState::EnteringOperand1 { register } => {
                self.state = CalculatorState::OperatorPending {
                    accumulator: register.to_f64(),
                    operator: op,
                };
            }
            CalculatorState::OperatorPending { accumulator, .. } => {
                // Change pending operator
                self.state = CalculatorState::OperatorPending {
                    accumulator: *accumulator,
                    operator: op,
                };
            }
            CalculatorState::EnteringOperand2 {
                accumulator,
                operator: prev_op,
                register,
            } => {
                // Evaluate intermediate result (chain calculation)
                let b = register.to_f64();
                let a = *accumulator;
                let op_prev = *prev_op;
                match self.engine.execute_binary(op_prev, a, b) {
                    Ok(result) => match Register::from_f64(result) {
                        Ok(_) => {
                            self.history.record_binary(a, op_prev, b, result);
                            self.state = CalculatorState::OperatorPending {
                                accumulator: result,
                                operator: op,
                            };
                        }
                        Err(_) => self.enter_error(),
                    },
                    Err(_) => self.enter_error(),
                }
            }

            CalculatorState::ResultDisplayed { register, .. } => {
                self.state = CalculatorState::OperatorPending {
                    accumulator: register.to_f64(),
                    operator: op,
                };
            }
            CalculatorState::Error => {}
        }
    }

    fn handle_unary_op(&mut self, op: UnaryOp) {
        match &mut self.state {
            CalculatorState::Ready => {
                if let UnaryOp::Negate = op {
                    // Negating 0 stays 0
                }
            }
            CalculatorState::EnteringOperand1 { register } => {
                if let UnaryOp::Negate = op {
                    register.toggle_sign();
                } else {
                    let val = register.to_f64();
                    match self.engine.execute_unary(op, val, None) {
                        Ok(res) => match Register::from_f64(res) {
                            Ok(new_reg) => {
                                self.history.record_unary(val, op, res);
                                self.state = CalculatorState::ResultDisplayed {
                                    register: new_reg,
                                    last_operation: None,
                                };
                            }
                            Err(_) => self.enter_error(),
                        },
                        Err(_) => self.enter_error(),
                    }
                }
            }
            CalculatorState::OperatorPending {
                accumulator,
                operator,
            } => {
                let acc = *accumulator;
                let pending_op = *operator;
                match self.engine.execute_unary(op, acc, None) {
                    Ok(res) => {
                        self.history.record_unary(acc, op, res);
                        self.state = CalculatorState::OperatorPending {
                            accumulator: res,
                            operator: pending_op,
                        };
                    }
                    Err(_) => self.enter_error(),
                }
            }
            CalculatorState::EnteringOperand2 {
                accumulator,
                operator,
                register,
            } => {
                if let UnaryOp::Negate = op {
                    register.toggle_sign();
                } else {
                    let val = register.to_f64();
                    let base = Some(*accumulator);
                    match self.engine.execute_unary(op, val, base) {
                        Ok(res) => match Register::from_f64(res) {
                            Ok(mut new_reg) => {
                                self.history.record_unary(val, op, res);
                                new_reg.set_editing(true);
                                self.state = CalculatorState::EnteringOperand2 {
                                    accumulator: *accumulator,
                                    operator: *operator,
                                    register: new_reg,
                                };
                            }
                            Err(_) => self.enter_error(),
                        },
                        Err(_) => self.enter_error(),
                    }
                }
            }
            CalculatorState::ResultDisplayed { register, .. } => {
                if let UnaryOp::Negate = op {
                    register.toggle_sign();
                } else {
                    let val = register.to_f64();
                    match self.engine.execute_unary(op, val, None) {
                        Ok(res) => match Register::from_f64(res) {
                            Ok(new_reg) => {
                                self.history.record_unary(val, op, res);
                                self.state = CalculatorState::ResultDisplayed {
                                    register: new_reg,
                                    last_operation: None,
                                };
                            }
                            Err(_) => self.enter_error(),
                        },
                        Err(_) => self.enter_error(),
                    }
                }
            }
            CalculatorState::Error => {}
        }
    }

    fn handle_memory_op(&mut self, op: MemoryOp) {
        let current_val = match &self.state {
            CalculatorState::Ready => 0.0,
            CalculatorState::EnteringOperand1 { register } => register.to_f64(),
            CalculatorState::OperatorPending { accumulator, .. } => *accumulator,
            CalculatorState::EnteringOperand2 { register, .. } => register.to_f64(),
            CalculatorState::ResultDisplayed { register, .. } => register.to_f64(),
            CalculatorState::Error => return,
        };

        match op {
            MemoryOp::Add => {
                if self.engine.memory_add(current_val).is_err() {
                    self.enter_error();
                }
            }
            MemoryOp::Subtract => {
                if self.engine.memory_sub(current_val).is_err() {
                    self.enter_error();
                }
            }
            MemoryOp::Recall => {
                let mem_val = self.engine.memory_recall();
                match Register::from_f64(mem_val) {
                    Ok(reg) => match &self.state {
                        CalculatorState::Ready | CalculatorState::ResultDisplayed { .. } => {
                            self.state = CalculatorState::ResultDisplayed {
                                register: reg,
                                last_operation: None,
                            };
                        }
                        CalculatorState::EnteringOperand1 { .. } => {
                            self.state = CalculatorState::EnteringOperand1 { register: reg };
                        }
                        CalculatorState::OperatorPending {
                            accumulator,
                            operator,
                        } => {
                            self.state = CalculatorState::EnteringOperand2 {
                                accumulator: *accumulator,
                                operator: *operator,
                                register: reg,
                            };
                        }
                        CalculatorState::EnteringOperand2 {
                            accumulator,
                            operator,
                            ..
                        } => {
                            self.state = CalculatorState::EnteringOperand2 {
                                accumulator: *accumulator,
                                operator: *operator,
                                register: reg,
                            };
                        }
                        CalculatorState::Error => {}
                    },
                    Err(_) => self.enter_error(),
                }
            }
            MemoryOp::Clear => {
                self.engine.memory_clear();
            }
        }
    }

    fn handle_equals(&mut self) {
        match &self.state {
            CalculatorState::Ready => {}
            CalculatorState::EnteringOperand1 { register } => {
                self.state = CalculatorState::ResultDisplayed {
                    register: register.clone(),
                    last_operation: None,
                };
            }
            CalculatorState::OperatorPending {
                accumulator,
                operator,
            } => {
                // Pocket calculator: 5 + = computes 5 + 5 = 10
                let a = *accumulator;
                let b = *accumulator;
                let op = *operator;
                match self.engine.execute_binary(op, a, b) {
                    Ok(result) => match Register::from_f64(result) {
                        Ok(reg) => {
                            self.history.record_binary(a, op, b, result);
                            self.state = CalculatorState::ResultDisplayed {
                                register: reg,
                                last_operation: Some((op, b)),
                            };
                        }
                        Err(_) => self.enter_error(),
                    },
                    Err(_) => self.enter_error(),
                }
            }
            CalculatorState::EnteringOperand2 {
                accumulator,
                operator,
                register,
            } => {
                let a = *accumulator;
                let op = *operator;
                let b = register.to_f64();
                match self.engine.execute_binary(op, a, b) {
                    Ok(result) => match Register::from_f64(result) {
                        Ok(reg) => {
                            self.history.record_binary(a, op, b, result);
                            self.state = CalculatorState::ResultDisplayed {
                                register: reg,
                                last_operation: Some((op, b)),
                            };
                        }
                        Err(_) => self.enter_error(),
                    },
                    Err(_) => self.enter_error(),
                }
            }
            CalculatorState::ResultDisplayed {
                register,
                last_operation,
            } => {
                // Repeat calculation: e.g. 5 + 3 = 8, pressing = again gives 11
                if let Some((op, b)) = *last_operation {
                    let a = register.to_f64();
                    match self.engine.execute_binary(op, a, b) {
                        Ok(result) => match Register::from_f64(result) {
                            Ok(reg) => {
                                self.history.record_binary(a, op, b, result);
                                self.state = CalculatorState::ResultDisplayed {
                                    register: reg,
                                    last_operation: Some((op, b)),
                                };
                            }
                            Err(_) => self.enter_error(),
                        },
                        Err(_) => self.enter_error(),
                    }
                }
            }
            CalculatorState::Error => {}
        }
    }

    fn handle_clear_entry(&mut self) {
        match &mut self.state {
            CalculatorState::EnteringOperand1 { register } => {
                register.clear();
                self.state = CalculatorState::Ready;
            }
            CalculatorState::EnteringOperand2 {
                accumulator,
                operator,
                ..
            } => {
                self.state = CalculatorState::OperatorPending {
                    accumulator: *accumulator,
                    operator: *operator,
                };
            }
            CalculatorState::ResultDisplayed { .. } => {
                self.state = CalculatorState::Ready;
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_addition() {
        let mut fsm = CalculatorFsm::new();
        fsm.process_key(Key::Digit(2));
        fsm.process_key(Key::BinaryOp(BinaryOp::Add));
        fsm.process_key(Key::Digit(3));
        fsm.process_key(Key::Equals);

        assert_eq!(fsm.display_string(), "5");
        assert!(!fsm.status_flags().has_error);
    }

    #[test]
    fn test_chain_calculation() {
        let mut fsm = CalculatorFsm::new();
        // 2 + 3 * 4 = should give 20 in pocket calculator model
        fsm.process_key(Key::Digit(2));
        fsm.process_key(Key::BinaryOp(BinaryOp::Add));
        fsm.process_key(Key::Digit(3));
        fsm.process_key(Key::BinaryOp(BinaryOp::Multiply));
        assert_eq!(fsm.display_string(), "5");

        fsm.process_key(Key::Digit(4));
        fsm.process_key(Key::Equals);
        assert_eq!(fsm.display_string(), "20");
    }

    #[test]
    fn test_division_by_zero_error() {
        let mut fsm = CalculatorFsm::new();
        fsm.process_key(Key::Digit(9));
        fsm.process_key(Key::BinaryOp(BinaryOp::Divide));
        fsm.process_key(Key::Digit(0));
        fsm.process_key(Key::Equals);

        assert!(fsm.status_flags().has_error);

        // Any further digits should be ignored
        fsm.process_key(Key::Digit(5));
        assert!(fsm.status_flags().has_error);

        // Clear unlocks
        fsm.process_key(Key::Clear);
        assert!(!fsm.status_flags().has_error);
        assert_eq!(fsm.display_string(), "0");
    }

    #[test]
    fn test_repeat_equals() {
        let mut fsm = CalculatorFsm::new();
        fsm.process_key(Key::Digit(5));
        fsm.process_key(Key::BinaryOp(BinaryOp::Add));
        fsm.process_key(Key::Digit(3));
        fsm.process_key(Key::Equals);
        assert_eq!(fsm.display_string(), "8");

        fsm.process_key(Key::Equals);
        assert_eq!(fsm.display_string(), "11");

        fsm.process_key(Key::Equals);
        assert_eq!(fsm.display_string(), "14");
    }
}
