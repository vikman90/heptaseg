//! Domain types and events for the Heptaseg calculator.

use std::fmt;

/// Binary arithmetic operators supported by the calculator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Subtract,
    Multiply,
    Divide,
}

impl fmt::Display for BinaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BinaryOp::Add => write!(f, "+"),
            BinaryOp::Subtract => write!(f, "-"),
            BinaryOp::Multiply => write!(f, "×"),
            BinaryOp::Divide => write!(f, "÷"),
        }
    }
}

/// Unary operations applied immediately to the current register.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    SquareRoot,
    Percentage,
    Negate,
}

/// Memory register operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryOp {
    Add,      // M+
    Subtract, // M-
    Recall,   // MR
    Clear,    // MC
}

/// A key press event originating from the UI keypad or physical keyboard.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    Digit(u8),
    DecimalPoint,
    BinaryOp(BinaryOp),
    UnaryOp(UnaryOp),
    MemoryOp(MemoryOp),
    Equals,
    Clear,      // AC / All Clear
    ClearEntry, // CE
}

/// Status flags indicating auxiliary annunciators on the LCD screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct StatusFlags {
    /// True when an arithmetic error or overflow has occurred.
    pub has_error: bool,
    /// True when the memory register contains a non-zero value.
    pub memory_active: bool,
    /// True when the displayed value is negative.
    pub negative: bool,
    /// True when the calculator is in RPN stack mode.
    pub is_rpn: bool,
    /// Currently pending binary operator, if any.
    pub active_operator: Option<BinaryOp>,
}

/// Operational calculation mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CalculatorMode {
    #[default]
    Standard,
    Rpn,
}

/// Domain errors produced during arithmetic execution or LCD register formatting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CalculatorError {
    DivisionByZero,
    Overflow,
    NegativeSquareRoot,
    InvalidInput,
}

impl fmt::Display for CalculatorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CalculatorError::DivisionByZero => write!(f, "Division by zero"),
            CalculatorError::Overflow => write!(f, "Numeric overflow"),
            CalculatorError::NegativeSquareRoot => write!(f, "Square root of negative number"),
            CalculatorError::InvalidInput => write!(f, "Invalid numeric input"),
        }
    }
}

impl std::error::Error for CalculatorError {}

/// Retro display palette themes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DisplayTheme {
    #[default]
    ClassicLcd,
    QuartzLcd,
    VfdCyan,
    SinclairLed,
}

impl DisplayTheme {
    /// Cycles to the next available display theme.
    pub fn next(&self) -> Self {
        match self {
            DisplayTheme::ClassicLcd => DisplayTheme::QuartzLcd,
            DisplayTheme::QuartzLcd => DisplayTheme::VfdCyan,
            DisplayTheme::VfdCyan => DisplayTheme::SinclairLed,
            DisplayTheme::SinclairLed => DisplayTheme::ClassicLcd,
        }
    }

    /// User-facing label for the theme.
    pub fn name(&self) -> &'static str {
        match self {
            DisplayTheme::ClassicLcd => "LCD OLIVE",
            DisplayTheme::QuartzLcd => "LCD QUARTZ",
            DisplayTheme::VfdCyan => "VFD CYAN",
            DisplayTheme::SinclairLed => "RUBY LED",
        }
    }

    /// Returns the (background, active_ink, ghost_ink, bezel_border) hex color strings.
    pub fn colors(&self) -> (&'static str, &'static str, &'static str, &'static str) {
        match self {
            DisplayTheme::ClassicLcd => ("#859870", "#050a04", "#748760", "#3a4430"),
            DisplayTheme::QuartzLcd => ("#9ba8a6", "#020405", "#849290", "#3d4645"),
            DisplayTheme::VfdCyan => ("#050c10", "#00ffd5", "#082b26", "#0e1a20"),
            DisplayTheme::SinclairLed => ("#140204", "#ff1e2e", "#38060a", "#2e060a"),
        }
    }
}
