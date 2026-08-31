//! Arithmetic engine and memory register management.

use crate::core::types::{BinaryOp, CalculatorError, UnaryOp};

/// Execution engine responsible for all mathematical calculations and memory storage.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Engine {
    /// Internal memory register (M).
    memory: f64,
}

impl Engine {
    /// Creates a new engine with cleared memory.
    pub fn new() -> Self {
        Self { memory: 0.0 }
    }

    /// Evaluates a binary arithmetic operation.
    /// Returns `Err(CalculatorError)` on division by zero, non-finite result, or overflow.
    pub fn execute_binary(&self, op: BinaryOp, a: f64, b: f64) -> Result<f64, CalculatorError> {
        let result = match op {
            BinaryOp::Add => a + b,
            BinaryOp::Subtract => a - b,
            BinaryOp::Multiply => a * b,
            BinaryOp::Divide => {
                if b == 0.0 {
                    return Err(CalculatorError::DivisionByZero);
                }
                a / b
            }
        };

        if result.is_finite() {
            Ok(result)
        } else {
            Err(CalculatorError::Overflow)
        }
    }

    /// Evaluates a unary operation.
    /// `base_val` is provided when evaluating percentage in the context of an ongoing binary operation.
    pub fn execute_unary(
        &self,
        op: UnaryOp,
        val: f64,
        base_val: Option<f64>,
    ) -> Result<f64, CalculatorError> {
        match op {
            UnaryOp::SquareRoot => {
                if val < 0.0 {
                    return Err(CalculatorError::NegativeSquareRoot);
                }
                let res = val.sqrt();
                if res.is_finite() {
                    Ok(res)
                } else {
                    Err(CalculatorError::Overflow)
                }
            }
            UnaryOp::Percentage => {
                // In pocket calculators:
                // If an operation like `100 + 10 %` is in progress, 10% evaluates to `100 * (10 / 100) = 10`.
                // If standalone `50 %`, evaluates to `50 / 100 = 0.5`.
                let res = if let Some(base) = base_val {
                    base * (val / 100.0)
                } else {
                    val / 100.0
                };
                if res.is_finite() {
                    Ok(res)
                } else {
                    Err(CalculatorError::Overflow)
                }
            }
            UnaryOp::Negate => Ok(-val),
        }
    }

    /// Adds value to memory register (M+).
    pub fn memory_add(&mut self, val: f64) -> Result<(), CalculatorError> {
        let new_mem = self.memory + val;
        if new_mem.is_finite() {
            self.memory = new_mem;
            Ok(())
        } else {
            Err(CalculatorError::Overflow)
        }
    }

    /// Subtracts value from memory register (M-).
    pub fn memory_sub(&mut self, val: f64) -> Result<(), CalculatorError> {
        let new_mem = self.memory - val;
        if new_mem.is_finite() {
            self.memory = new_mem;
            Ok(())
        } else {
            Err(CalculatorError::Overflow)
        }
    }

    /// Recalls current memory register value (MR).
    pub fn memory_recall(&self) -> f64 {
        self.memory
    }

    /// Clears memory register (MC).
    pub fn memory_clear(&mut self) {
        self.memory = 0.0;
    }

    /// Returns whether the memory register contains a non-zero value.
    pub fn has_memory(&self) -> bool {
        self.memory != 0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_binary_operations() {
        let engine = Engine::new();
        assert_eq!(engine.execute_binary(BinaryOp::Add, 15.0, 27.0), Ok(42.0));
        assert_eq!(
            engine.execute_binary(BinaryOp::Subtract, 100.0, 42.0),
            Ok(58.0)
        );
        assert_eq!(
            engine.execute_binary(BinaryOp::Multiply, 6.0, 7.0),
            Ok(42.0)
        );
        assert_eq!(engine.execute_binary(BinaryOp::Divide, 84.0, 2.0), Ok(42.0));
        assert_eq!(
            engine.execute_binary(BinaryOp::Divide, 42.0, 0.0),
            Err(CalculatorError::DivisionByZero)
        );
    }

    #[test]
    fn test_unary_operations() {
        let engine = Engine::new();
        assert_eq!(
            engine.execute_unary(UnaryOp::SquareRoot, 16.0, None),
            Ok(4.0)
        );
        assert_eq!(
            engine.execute_unary(UnaryOp::SquareRoot, -1.0, None),
            Err(CalculatorError::NegativeSquareRoot)
        );
        assert_eq!(engine.execute_unary(UnaryOp::Negate, 42.0, None), Ok(-42.0));

        // Percentage standalone
        assert_eq!(
            engine.execute_unary(UnaryOp::Percentage, 25.0, None),
            Ok(0.25)
        );
        // Percentage relative to base (e.g. 200 + 10%)
        assert_eq!(
            engine.execute_unary(UnaryOp::Percentage, 10.0, Some(200.0)),
            Ok(20.0)
        );
    }

    #[test]
    fn test_memory_register() {
        let mut engine = Engine::new();
        assert!(!engine.has_memory());
        assert_eq!(engine.memory_recall(), 0.0);

        engine.memory_add(100.0).unwrap();
        assert!(engine.has_memory());
        assert_eq!(engine.memory_recall(), 100.0);

        engine.memory_sub(40.0).unwrap();
        assert_eq!(engine.memory_recall(), 60.0);

        engine.memory_clear();
        assert!(!engine.has_memory());
        assert_eq!(engine.memory_recall(), 0.0);
    }
}
