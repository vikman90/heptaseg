use heptaseg::core::fsm::CalculatorFsm;
use heptaseg::core::types::{BinaryOp, CalculatorMode, Key, UnaryOp};

#[test]
fn test_rpn_basic_addition() {
    let mut fsm = CalculatorFsm::new();
    fsm.set_mode(CalculatorMode::Rpn);

    // 3 Enter 4 + -> 7
    fsm.process_key(Key::Digit(3));
    fsm.process_key(Key::Equals); // Enter in RPN
    fsm.process_key(Key::Digit(4));
    fsm.process_key(Key::BinaryOp(BinaryOp::Add));

    assert_eq!(fsm.display_string(), "7");
    assert!(fsm.status_flags().is_rpn);
}

#[test]
fn test_rpn_complex_expression() {
    let mut fsm = CalculatorFsm::new();
    fsm.set_mode(CalculatorMode::Rpn);

    // (5 + 3) * 2 = 16
    fsm.process_key(Key::Digit(5));
    fsm.process_key(Key::Equals);
    fsm.process_key(Key::Digit(3));
    fsm.process_key(Key::BinaryOp(BinaryOp::Add)); // 8
    fsm.process_key(Key::Equals);
    fsm.process_key(Key::Digit(2));
    fsm.process_key(Key::BinaryOp(BinaryOp::Multiply)); // 16

    assert_eq!(fsm.display_string(), "16");
}

#[test]
fn test_rpn_stack_cascade() {
    let mut fsm = CalculatorFsm::new();
    fsm.set_mode(CalculatorMode::Rpn);

    // 1 Enter 2 Enter 3 Enter 4 + + + -> 10
    fsm.process_key(Key::Digit(1));
    fsm.process_key(Key::Equals);
    fsm.process_key(Key::Digit(2));
    fsm.process_key(Key::Equals);
    fsm.process_key(Key::Digit(3));
    fsm.process_key(Key::Equals);
    fsm.process_key(Key::Digit(4));

    fsm.process_key(Key::BinaryOp(BinaryOp::Add)); // 3 + 4 = 7
    assert_eq!(fsm.display_string(), "7");

    fsm.process_key(Key::BinaryOp(BinaryOp::Add)); // 2 + 7 = 9
    assert_eq!(fsm.display_string(), "9");

    fsm.process_key(Key::BinaryOp(BinaryOp::Add)); // 1 + 9 = 10
    assert_eq!(fsm.display_string(), "10");
}

#[test]
fn test_rpn_unary_operations() {
    let mut fsm = CalculatorFsm::new();
    fsm.set_mode(CalculatorMode::Rpn);

    // 25 sqrt -> 5
    fsm.process_key(Key::Digit(2));
    fsm.process_key(Key::Digit(5));
    fsm.process_key(Key::UnaryOp(UnaryOp::SquareRoot));

    assert_eq!(fsm.display_string(), "5");
}
