//! 4-Level Stack Engine for HP-Style Reverse Polish Notation (RPN).

use crate::core::types::CalculatorError;

/// Classic 4-register operational stack (X, Y, Z, T).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct RpnStack {
    /// Bottom of the stack (displayed value / primary operand).
    pub x: f64,
    /// Second stack level.
    pub y: f64,
    /// Third stack level.
    pub z: f64,
    /// Top stack level.
    pub t: f64,
}

impl RpnStack {
    /// Creates a new empty RPN stack with all registers zeroed.
    pub fn new() -> Self {
        Self::default()
    }

    /// Pushes the stack upwards: T is overwritten by Z, Z by Y, Y by X.
    /// X remains unchanged until edited by the user.
    pub fn push_enter(&mut self) {
        self.t = self.z;
        self.z = self.y;
        self.y = self.x;
    }

    /// Evaluates a binary operation: `(Y op X) -> X`, dropping Z into Y and T into Z.
    pub fn execute_binary<F>(&mut self, op: F) -> Result<f64, CalculatorError>
    where
        F: FnOnce(f64, f64) -> Result<f64, CalculatorError>,
    {
        let res = op(self.y, self.x)?;
        self.x = res;
        self.y = self.z;
        self.z = self.t;
        Ok(res)
    }

    /// Swaps the X and Y registers (X <-> Y).
    pub fn swap_xy(&mut self) {
        std::mem::swap(&mut self.x, &mut self.y);
    }

    /// Rolls the stack downwards: X -> T, Y -> X, Z -> Y, T -> Z.
    pub fn roll_down(&mut self) {
        let old_x = self.x;
        self.x = self.y;
        self.y = self.z;
        self.z = self.t;
        self.t = old_x;
    }

    /// Clears all 4 stack levels to 0.0.
    pub fn clear(&mut self) {
        self.x = 0.0;
        self.y = 0.0;
        self.z = 0.0;
        self.t = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rpn_push_and_binary_execution() {
        let mut stack = RpnStack::new();

        // Push 3, then 4
        stack.x = 3.0;
        stack.push_enter();
        stack.x = 4.0;

        assert_eq!(stack.x, 4.0);
        assert_eq!(stack.y, 3.0);

        // 3 + 4 = 7
        let res = stack.execute_binary(|y, x| Ok(y + x)).unwrap();
        assert_eq!(res, 7.0);
        assert_eq!(stack.x, 7.0);
        assert_eq!(stack.y, 0.0);
    }

    #[test]
    fn test_rpn_swap_and_roll() {
        let mut stack = RpnStack {
            x: 1.0,
            y: 2.0,
            z: 3.0,
            t: 4.0,
        };

        stack.swap_xy();
        assert_eq!(stack.x, 2.0);
        assert_eq!(stack.y, 1.0);

        stack.roll_down();
        assert_eq!(stack.x, 1.0);
        assert_eq!(stack.y, 3.0);
        assert_eq!(stack.z, 4.0);
        assert_eq!(stack.t, 2.0);
    }
}
