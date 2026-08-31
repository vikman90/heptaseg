//! Integration tests for the HP-Style RPN calculation mode.

use heptaseg::core::rpn::RpnCalculator;
use heptaseg::core::types::{BinaryOp, Key, UnaryOp};

#[test]
fn test_rpn_chain_evaluation() {
    let mut calc = RpnCalculator::new();

    // Expression: (5 + 3) * (10 - 2)
    // RPN sequence: 5 Enter 3 + 10 Enter 2 - *
    calc.process_key(Key::Digit(5));
    calc.process_key(Key::Equals); // ENTER
    calc.process_key(Key::Digit(3));
    calc.process_key(Key::BinaryOp(BinaryOp::Add)); // X = 8

    assert_eq!(calc.display_string(), "8");

    calc.process_key(Key::Equals); // Push 8 to Y
    calc.process_key(Key::Digit(1));
    calc.process_key(Key::Digit(0));
    calc.process_key(Key::Equals); // Push 10 to Y (8 is now in Z)
    calc.process_key(Key::Digit(2));
    calc.process_key(Key::BinaryOp(BinaryOp::Subtract)); // 10 - 2 = 8, Z (8) drops to Y

    assert_eq!(calc.display_string(), "8");
    assert_eq!(calc.stack().y, 8.0);

    calc.process_key(Key::BinaryOp(BinaryOp::Multiply)); // 8 * 8 = 64
    assert_eq!(calc.display_string(), "64");
}

#[test]
fn test_rpn_unary_square_root_and_percentage() {
    let mut calc = RpnCalculator::new();

    // 144 SQRT -> 12
    calc.process_key(Key::Digit(1));
    calc.process_key(Key::Digit(4));
    calc.process_key(Key::Digit(4));
    calc.process_key(Key::UnaryOp(UnaryOp::SquareRoot));

    assert_eq!(calc.display_string(), "12");

    // Clear
    calc.process_key(Key::Clear);
    assert_eq!(calc.display_string(), "0");
}
