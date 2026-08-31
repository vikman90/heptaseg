use heptaseg::core::fsm::CalculatorFsm;
use heptaseg::core::types::{BinaryOp, Key, MemoryOp, UnaryOp};

fn feed_keys(fsm: &mut CalculatorFsm, keys: &[Key]) {
    for key in keys {
        fsm.process_key(*key);
    }
}

#[test]
fn test_standard_operations() {
    let mut fsm = CalculatorFsm::new();

    // 12 + 34 = 46
    feed_keys(
        &mut fsm,
        &[
            Key::Digit(1),
            Key::Digit(2),
            Key::BinaryOp(BinaryOp::Add),
            Key::Digit(3),
            Key::Digit(4),
            Key::Equals,
        ],
    );
    assert_eq!(fsm.display_string(), "46");
    assert!(!fsm.status_flags().negative);

    // Negative result: 10 - 25 = -15
    fsm.process_key(Key::Clear);
    feed_keys(
        &mut fsm,
        &[
            Key::Digit(1),
            Key::Digit(0),
            Key::BinaryOp(BinaryOp::Subtract),
            Key::Digit(2),
            Key::Digit(5),
            Key::Equals,
        ],
    );
    assert_eq!(fsm.display_string(), "15");
    assert!(fsm.status_flags().negative);
}

#[test]
fn test_chained_operations_without_equals() {
    let mut fsm = CalculatorFsm::new();
    // 2 + 3 + 4 + 5 = 14
    feed_keys(
        &mut fsm,
        &[
            Key::Digit(2),
            Key::BinaryOp(BinaryOp::Add),
            Key::Digit(3),
            Key::BinaryOp(BinaryOp::Add),
            Key::Digit(4),
            Key::BinaryOp(BinaryOp::Add),
            Key::Digit(5),
            Key::Equals,
        ],
    );
    assert_eq!(fsm.display_string(), "14");
}

#[test]
fn test_pocket_calc_sequential_precedence() {
    let mut fsm = CalculatorFsm::new();
    // 2 + 3 * 4 = 20 (Immediate sequential execution)
    feed_keys(
        &mut fsm,
        &[
            Key::Digit(2),
            Key::BinaryOp(BinaryOp::Add),
            Key::Digit(3),
            Key::BinaryOp(BinaryOp::Multiply),
            Key::Digit(4),
            Key::Equals,
        ],
    );
    assert_eq!(fsm.display_string(), "20");
}

#[test]
fn test_replacing_pending_operator() {
    let mut fsm = CalculatorFsm::new();
    // 10 + then * then / 2 = 5
    feed_keys(
        &mut fsm,
        &[
            Key::Digit(1),
            Key::Digit(0),
            Key::BinaryOp(BinaryOp::Add),
            Key::BinaryOp(BinaryOp::Multiply),
            Key::BinaryOp(BinaryOp::Divide),
            Key::Digit(2),
            Key::Equals,
        ],
    );
    assert_eq!(fsm.display_string(), "5");
}

#[test]
fn test_decimal_precision() {
    let mut fsm = CalculatorFsm::new();
    // .5 + .25 = 0.75
    feed_keys(
        &mut fsm,
        &[
            Key::DecimalPoint,
            Key::Digit(5),
            Key::BinaryOp(BinaryOp::Add),
            Key::DecimalPoint,
            Key::Digit(2),
            Key::Digit(5),
            Key::Equals,
        ],
    );
    assert_eq!(fsm.display_string(), "0.75");
}

#[test]
fn test_square_root() {
    let mut fsm = CalculatorFsm::new();
    // 144 sqrt = 12
    feed_keys(
        &mut fsm,
        &[
            Key::Digit(1),
            Key::Digit(4),
            Key::Digit(4),
            Key::UnaryOp(UnaryOp::SquareRoot),
        ],
    );
    assert_eq!(fsm.display_string(), "12");

    // sqrt of negative number triggers error
    fsm.process_key(Key::Clear);
    feed_keys(
        &mut fsm,
        &[
            Key::Digit(9),
            Key::UnaryOp(UnaryOp::Negate),
            Key::UnaryOp(UnaryOp::SquareRoot),
        ],
    );
    assert!(fsm.status_flags().has_error);
}

#[test]
fn test_percentage_operations() {
    let mut fsm = CalculatorFsm::new();
    // 200 + 10 % = 220
    feed_keys(
        &mut fsm,
        &[
            Key::Digit(2),
            Key::Digit(0),
            Key::Digit(0),
            Key::BinaryOp(BinaryOp::Add),
            Key::Digit(1),
            Key::Digit(0),
            Key::UnaryOp(UnaryOp::Percentage),
            Key::Equals,
        ],
    );
    assert_eq!(fsm.display_string(), "220");
}

#[test]
fn test_memory_workflow() {
    let mut fsm = CalculatorFsm::new();
    assert!(!fsm.status_flags().memory_active);

    // 10 M+
    feed_keys(
        &mut fsm,
        &[Key::Digit(1), Key::Digit(0), Key::MemoryOp(MemoryOp::Add)],
    );
    assert!(fsm.status_flags().memory_active);

    // 25 M+
    fsm.process_key(Key::Clear);
    feed_keys(
        &mut fsm,
        &[Key::Digit(2), Key::Digit(5), Key::MemoryOp(MemoryOp::Add)],
    );

    // MR -> 35
    fsm.process_key(Key::Clear);
    fsm.process_key(Key::MemoryOp(MemoryOp::Recall));
    assert_eq!(fsm.display_string(), "35");

    // 5 M- -> Memory becomes 30
    feed_keys(
        &mut fsm,
        &[Key::Digit(5), Key::MemoryOp(MemoryOp::Subtract)],
    );

    // MR -> 30
    fsm.process_key(Key::Clear);
    fsm.process_key(Key::MemoryOp(MemoryOp::Recall));
    assert_eq!(fsm.display_string(), "30");

    // MC
    fsm.process_key(Key::MemoryOp(MemoryOp::Clear));
    assert!(!fsm.status_flags().memory_active);
}

#[test]
fn test_clear_entry_recovery() {
    let mut fsm = CalculatorFsm::new();
    // 10 + 999 [CE] 5 = 15
    feed_keys(
        &mut fsm,
        &[
            Key::Digit(1),
            Key::Digit(0),
            Key::BinaryOp(BinaryOp::Add),
            Key::Digit(9),
            Key::Digit(9),
            Key::Digit(9),
            Key::ClearEntry,
            Key::Digit(5),
            Key::Equals,
        ],
    );
    assert_eq!(fsm.display_string(), "15");
}

#[test]
fn test_error_locking_and_clearing() {
    let mut fsm = CalculatorFsm::new();
    // 50 / 0 = ERR
    feed_keys(
        &mut fsm,
        &[
            Key::Digit(5),
            Key::Digit(0),
            Key::BinaryOp(BinaryOp::Divide),
            Key::Digit(0),
            Key::Equals,
        ],
    );
    assert!(fsm.status_flags().has_error);

    // Pressing other keys has no effect
    feed_keys(
        &mut fsm,
        &[
            Key::Digit(1),
            Key::Digit(2),
            Key::BinaryOp(BinaryOp::Add),
            Key::Equals,
        ],
    );
    assert!(fsm.status_flags().has_error);

    // AC unlocks
    fsm.process_key(Key::Clear);
    assert!(!fsm.status_flags().has_error);
    assert_eq!(fsm.display_string(), "0");
}

#[test]
fn test_repeated_equals_subtraction_and_multiplication() {
    let mut fsm = CalculatorFsm::new();

    // 10 - 2 = 8 = 6 = 4
    feed_keys(
        &mut fsm,
        &[
            Key::Digit(1),
            Key::Digit(0),
            Key::BinaryOp(BinaryOp::Subtract),
            Key::Digit(2),
            Key::Equals,
        ],
    );
    assert_eq!(fsm.display_string(), "8");

    fsm.process_key(Key::Equals);
    assert_eq!(fsm.display_string(), "6");

    fsm.process_key(Key::Equals);
    assert_eq!(fsm.display_string(), "4");

    // 3 * 2 = 6 = 12 = 24
    fsm.process_key(Key::Clear);
    feed_keys(
        &mut fsm,
        &[
            Key::Digit(3),
            Key::BinaryOp(BinaryOp::Multiply),
            Key::Digit(2),
            Key::Equals,
        ],
    );
    assert_eq!(fsm.display_string(), "6");

    fsm.process_key(Key::Equals);
    assert_eq!(fsm.display_string(), "12");

    fsm.process_key(Key::Equals);
    assert_eq!(fsm.display_string(), "24");
}

#[test]
fn test_multiple_decimal_points_ignored() {
    let mut fsm = CalculatorFsm::new();
    // 1 . 2 . 3 . 4
    feed_keys(
        &mut fsm,
        &[
            Key::Digit(1),
            Key::DecimalPoint,
            Key::Digit(2),
            Key::DecimalPoint,
            Key::Digit(3),
            Key::DecimalPoint,
            Key::Digit(4),
        ],
    );
    assert_eq!(fsm.display_string(), "1.234");
}

#[test]
fn test_leading_zeros_handling() {
    let mut fsm = CalculatorFsm::new();
    // 0 0 0 7 -> 7
    feed_keys(
        &mut fsm,
        &[Key::Digit(0), Key::Digit(0), Key::Digit(0), Key::Digit(7)],
    );
    assert_eq!(fsm.display_string(), "7");

    // 0 . 0 5 -> 0.05
    fsm.process_key(Key::Clear);
    feed_keys(
        &mut fsm,
        &[
            Key::Digit(0),
            Key::DecimalPoint,
            Key::Digit(0),
            Key::Digit(5),
        ],
    );
    assert_eq!(fsm.display_string(), "0.05");
}

#[test]
fn test_negate_workflow() {
    let mut fsm = CalculatorFsm::new();
    // 50 +/- -> -50 + 20 = -30
    feed_keys(
        &mut fsm,
        &[
            Key::Digit(5),
            Key::Digit(0),
            Key::UnaryOp(UnaryOp::Negate),
            Key::BinaryOp(BinaryOp::Add),
            Key::Digit(2),
            Key::Digit(0),
            Key::Equals,
        ],
    );
    assert_eq!(fsm.display_string(), "30");
    assert!(fsm.status_flags().negative);

    // Negate the result -> 30 (positive)
    fsm.process_key(Key::UnaryOp(UnaryOp::Negate));
    assert_eq!(fsm.display_string(), "30");
    assert!(!fsm.status_flags().negative);
}

#[test]
fn test_overflow_detection() {
    let mut fsm = CalculatorFsm::new();
    // 99999999 * 2 = ERR
    feed_keys(
        &mut fsm,
        &[
            Key::Digit(9),
            Key::Digit(9),
            Key::Digit(9),
            Key::Digit(9),
            Key::Digit(9),
            Key::Digit(9),
            Key::Digit(9),
            Key::Digit(9),
            Key::BinaryOp(BinaryOp::Multiply),
            Key::Digit(2),
            Key::Equals,
        ],
    );
    assert!(fsm.status_flags().has_error);
}
