use heptaseg::core::fsm::CalculatorFsm;
use heptaseg::core::types::{BinaryOp, Key, UnaryOp};

#[test]
fn test_fsm_records_history_on_equals() {
    let mut fsm = CalculatorFsm::new();

    // 12 + 34 = 46
    fsm.process_key(Key::Digit(1));
    fsm.process_key(Key::Digit(2));
    fsm.process_key(Key::BinaryOp(BinaryOp::Add));
    fsm.process_key(Key::Digit(3));
    fsm.process_key(Key::Digit(4));
    fsm.process_key(Key::Equals);

    assert_eq!(fsm.display_string(), "46");
    assert_eq!(fsm.history().len(), 1);
    assert_eq!(fsm.history().entries()[0].expression, "12 + 34 = 46");
}

#[test]
fn test_fsm_records_chained_calculations_history() {
    let mut fsm = CalculatorFsm::new();

    // 10 + 20 * 2 = 60
    fsm.process_key(Key::Digit(1));
    fsm.process_key(Key::Digit(0));
    fsm.process_key(Key::BinaryOp(BinaryOp::Add));
    fsm.process_key(Key::Digit(2));
    fsm.process_key(Key::Digit(0));
    fsm.process_key(Key::BinaryOp(BinaryOp::Multiply)); // evaluates 10 + 20 = 30
    fsm.process_key(Key::Digit(2));
    fsm.process_key(Key::Equals); // evaluates 30 * 2 = 60

    assert_eq!(fsm.display_string(), "60");
    assert_eq!(fsm.history().len(), 2);
    assert_eq!(fsm.history().entries()[0].expression, "10 + 20 = 30");
    assert_eq!(fsm.history().entries()[1].expression, "30 × 2 = 60");
}

#[test]
fn test_fsm_records_unary_history() {
    let mut fsm = CalculatorFsm::new();

    // 16 sqrt = 4
    fsm.process_key(Key::Digit(1));
    fsm.process_key(Key::Digit(6));
    fsm.process_key(Key::UnaryOp(UnaryOp::SquareRoot));

    assert_eq!(fsm.display_string(), "4");
    assert_eq!(fsm.history().len(), 1);
    assert_eq!(fsm.history().entries()[0].expression, "√(16) = 4");
}

#[test]
fn test_fsm_clear_history() {
    let mut fsm = CalculatorFsm::new();

    fsm.process_key(Key::Digit(5));
    fsm.process_key(Key::BinaryOp(BinaryOp::Add));
    fsm.process_key(Key::Digit(5));
    fsm.process_key(Key::Equals);

    assert_eq!(fsm.history().len(), 1);
    fsm.clear_history();
    assert!(fsm.history().is_empty());
}
