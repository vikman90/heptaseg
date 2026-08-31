//! Numeric register and formatting for 8-digit LCD displays.

use crate::core::types::CalculatorError;

/// Maximum number of numerical digits supported by the vintage 8-digit LCD panel.
pub const MAX_LCD_DIGITS: usize = 8;

/// A numeric register representing the input or output value on the LCD.
#[derive(Debug, Clone, PartialEq)]
pub struct Register {
    /// Raw string buffer representing the numeric entry (e.g., "123.45" or "0").
    buffer: String,
    /// Whether the value is negative.
    is_negative: bool,
    /// Flag indicating whether the register is currently in active editing mode (user typing digits).
    is_editing: bool,
}

impl Default for Register {
    fn default() -> Self {
        Self::new()
    }
}

impl Register {
    /// Creates a new register initialized to zero.
    pub fn new() -> Self {
        Self {
            buffer: "0".to_string(),
            is_negative: false,
            is_editing: false,
        }
    }

    /// Creates a register populated from an `f64` number.
    pub fn from_f64(val: f64) -> Result<Self, CalculatorError> {
        if !val.is_finite() {
            return Err(CalculatorError::Overflow);
        }

        let is_negative = val < 0.0;
        let abs_val = val.abs();

        // Check if value exceeds 8-digit capacity (99,999,999)
        if abs_val >= 100_000_000.0 {
            return Err(CalculatorError::Overflow);
        }

        let formatted = Self::format_f64_to_8digits(abs_val);

        Ok(Self {
            buffer: formatted,
            is_negative,
            is_editing: false,
        })
    }

    /// Appends a digit (0-9) to the register.
    pub fn append_digit(&mut self, digit: u8) {
        debug_assert!(digit <= 9);

        if !self.is_editing {
            self.buffer = digit.to_string();
            self.is_negative = false;
            self.is_editing = true;
            return;
        }

        let digit_count = self.digit_count();
        if digit_count >= MAX_LCD_DIGITS {
            return; // Ignore input when max digits reached
        }

        if self.buffer == "0" {
            self.buffer = digit.to_string();
        } else {
            self.buffer.push_str(&digit.to_string());
        }
    }

    /// Appends a decimal point if one is not already present.
    pub fn append_decimal(&mut self) {
        if !self.is_editing {
            self.buffer = "0.".to_string();
            self.is_negative = false;
            self.is_editing = true;
            return;
        }

        if !self.buffer.contains('.') {
            self.buffer.push('.');
        }
    }

    /// Toggles the sign of the register.
    pub fn toggle_sign(&mut self) {
        if self.to_f64() == 0.0 {
            self.is_negative = false;
            return;
        }
        self.is_negative = !self.is_negative;
    }

    /// Clears the current entry back to "0".
    pub fn clear(&mut self) {
        self.buffer = "0".to_string();
        self.is_negative = false;
        self.is_editing = false;
    }

    /// Returns the numerical `f64` value.
    pub fn to_f64(&self) -> f64 {
        let raw: f64 = self.buffer.parse().unwrap_or(0.0);
        if self.is_negative {
            -raw
        } else {
            raw
        }
    }

    /// Returns the number of numeric digits in the current buffer (excluding decimal points).
    pub fn digit_count(&self) -> usize {
        self.buffer.chars().filter(|c| c.is_ascii_digit()).count()
    }

    /// Returns whether the register is currently in editing mode.
    pub fn is_editing(&self) -> bool {
        self.is_editing
    }

    /// Sets editing mode explicitly.
    pub fn set_editing(&mut self, editing: bool) {
        self.is_editing = editing;
    }

    /// Returns whether the value is negative.
    pub fn is_negative(&self) -> bool {
        self.is_negative && self.to_f64() != 0.0
    }

    /// Returns the string representation formatted for LCD display (digits + decimal point).
    pub fn display_string(&self) -> &str {
        &self.buffer
    }

    /// Formats an `f64` number to fit within the 8-digit display constraint.
    fn format_f64_to_8digits(val: f64) -> String {
        // Round very close values to eliminate floating point inaccuracies (e.g. 0.1 + 0.2)
        let rounded = (val * 10_000_000.0).round() / 10_000_000.0;

        if rounded.fract() == 0.0 {
            // Integer value
            return format!("{:.0}", rounded);
        }

        // Decimal formatting: try to fit within 8 total digits
        let integer_part = rounded.trunc();
        let integer_digits = if integer_part == 0.0 {
            1
        } else {
            format!("{:.0}", integer_part).len()
        };

        if integer_digits >= MAX_LCD_DIGITS {
            return format!("{:.0}", integer_part);
        }

        let decimal_places = MAX_LCD_DIGITS.saturating_sub(integer_digits);
        let formatted = format!("{:.*}", decimal_places, rounded);

        // Trim trailing zeros and unnecessary decimal point
        let trimmed = formatted.trim_end_matches('0');
        let trimmed = trimmed.trim_end_matches('.');
        trimmed.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_register() {
        let reg = Register::new();
        assert_eq!(reg.display_string(), "0");
        assert_eq!(reg.to_f64(), 0.0);
        assert!(!reg.is_negative());
    }

    #[test]
    fn test_digit_entry() {
        let mut reg = Register::new();
        reg.append_digit(4);
        assert_eq!(reg.display_string(), "4");
        reg.append_digit(2);
        assert_eq!(reg.display_string(), "42");
        assert_eq!(reg.to_f64(), 42.0);
    }

    #[test]
    fn test_decimal_entry() {
        let mut reg = Register::new();
        reg.append_decimal();
        assert_eq!(reg.display_string(), "0.");
        reg.append_digit(5);
        assert_eq!(reg.display_string(), "0.5");
        assert_eq!(reg.to_f64(), 0.5);

        // Second decimal is ignored
        reg.append_decimal();
        assert_eq!(reg.display_string(), "0.5");
    }

    #[test]
    fn test_max_digits_limit() {
        let mut reg = Register::new();
        for _ in 0..12 {
            reg.append_digit(9);
        }
        assert_eq!(reg.display_string(), "99999999");
        assert_eq!(reg.digit_count(), 8);
    }

    #[test]
    fn test_toggle_sign() {
        let mut reg = Register::new();
        reg.append_digit(5);
        reg.toggle_sign();
        assert!(reg.is_negative());
        assert_eq!(reg.to_f64(), -5.0);

        reg.toggle_sign();
        assert!(!reg.is_negative());
        assert_eq!(reg.to_f64(), 5.0);
    }

    #[test]
    fn test_from_f64() {
        let reg = Register::from_f64(12.3456).unwrap();
        assert_eq!(reg.display_string(), "12.3456");

        let reg_neg = Register::from_f64(-42.0).unwrap();
        assert_eq!(reg_neg.display_string(), "42");
        assert!(reg_neg.is_negative());

        // Overflow
        assert!(Register::from_f64(100_000_000.0).is_err());
        assert!(Register::from_f64(f64::INFINITY).is_err());
    }
}
