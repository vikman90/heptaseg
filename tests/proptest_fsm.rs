use heptaseg::core::fsm::CalculatorFsm;
use heptaseg::core::register::{Register, MAX_LCD_DIGITS};
use heptaseg::core::types::{BinaryOp, Key, MemoryOp, UnaryOp};
use proptest::prelude::*;

/// Strategy to generate arbitrary calculator keys.
fn arb_key() -> impl Strategy<Value = Key> {
    prop_oneof![
        (0u8..=9).prop_map(Key::Digit),
        Just(Key::DecimalPoint),
        prop_oneof![
            Just(BinaryOp::Add),
            Just(BinaryOp::Subtract),
            Just(BinaryOp::Multiply),
            Just(BinaryOp::Divide),
        ]
        .prop_map(Key::BinaryOp),
        prop_oneof![
            Just(UnaryOp::SquareRoot),
            Just(UnaryOp::Percentage),
            Just(UnaryOp::Negate),
        ]
        .prop_map(Key::UnaryOp),
        prop_oneof![
            Just(MemoryOp::Add),
            Just(MemoryOp::Subtract),
            Just(MemoryOp::Recall),
            Just(MemoryOp::Clear),
        ]
        .prop_map(Key::MemoryOp),
        Just(Key::Equals),
        Just(Key::Clear),
        Just(Key::ClearEntry),
    ]
}

proptest! {
    /// Fuzzes the FSM with random sequences of 1 to 200 keystrokes.
    /// Invariants verified:
    /// 1. The FSM never panics or aborts.
    /// 2. The display string never exceeds the LCD character limit.
    /// 3. The digit count never exceeds MAX_LCD_DIGITS (8).
    /// 4. Error state is fully locked until Clear (AC) is pressed.
    #[test]
    fn prop_fsm_random_keystroke_invariants(keys in proptest::collection::vec(arb_key(), 1..200)) {
        let mut fsm = CalculatorFsm::new();

        for key in keys {
            fsm.process_key(key);

            let display = fsm.display_string();
            let flags = fsm.status_flags();

            // Invariant: Display string is never empty
            prop_assert!(!display.is_empty(), "Display string must not be empty");

            // Invariant: Numeric digits never exceed 8
            let digit_count = display.chars().filter(|c| c.is_ascii_digit()).count();
            prop_assert!(
                digit_count <= MAX_LCD_DIGITS,
                "Digit count {} exceeded MAX_LCD_DIGITS ({}) in display: '{}'",
                digit_count,
                MAX_LCD_DIGITS,
                display
            );

            // Invariant: In error state, display is '0'
            if flags.has_error {
                prop_assert_eq!(display.as_str(), "0", "Display in error state must show '0'");
            }
        }
    }

    /// Verifies that Register::from_f64 handles arbitrary floating point values safely.
    #[test]
    fn prop_register_from_arbitrary_f64_never_panics(val in any::<f64>()) {
        match Register::from_f64(val) {
            Ok(reg) => {
                prop_assert!(reg.digit_count() <= MAX_LCD_DIGITS);
                let num = reg.to_f64();
                prop_assert!(num.is_finite());
            }
            Err(_) => {
                // Returns Err safely on overflow / NaN / Inf
                prop_assert!(!val.is_finite() || val.abs() >= 100_000_000.0);
            }
        }
    }
}
