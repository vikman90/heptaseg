//! Heptaseg: Vintage 7-Segment LCD Pocket Calculator.

mod audio;

use std::cell::RefCell;
use std::rc::Rc;

use audio::SoundManager;
use heptaseg::core::fsm::CalculatorFsm;

use heptaseg::core::register::MAX_LCD_DIGITS;
use heptaseg::core::types::{BinaryOp, DisplayTheme, Key, MemoryOp, UnaryOp};
use slint::{ModelRc, SharedString, VecModel};

slint::include_modules!();

/// Converts a hex color string (e.g. "#899975") to a Slint Color.
fn hex_to_color(hex: &str) -> slint::Color {
    let hex = hex.trim_start_matches('#');
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
    slint::Color::from_argb_u8(255, r, g, b)
}

/// Applies the active palette theme to the Slint LCD screen.
fn apply_theme(window: &AppWindow, theme: DisplayTheme) {
    let (bg, active, ghost, border) = theme.colors();
    window.set_lcd_bg(hex_to_color(bg));
    window.set_active_ink(hex_to_color(active));
    window.set_ghost_ink(hex_to_color(ghost));
    window.set_border_color(hex_to_color(border));
    window.set_theme_name(SharedString::from(theme.name()));
}

/// Parses the raw display string from the FSM into 8 aligned LCD digit cells and decimal flags.
fn parse_lcd_cells(raw: &str) -> ([SharedString; MAX_LCD_DIGITS], [bool; MAX_LCD_DIGITS]) {
    let mut digits: Vec<char> = Vec::new();
    let mut decimals: Vec<bool> = Vec::new();

    let mut chars = raw.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch.is_ascii_digit() || ch == '-' || ch == 'E' || ch == 'r' || ch == 'o' {
            digits.push(ch);
            if chars.peek() == Some(&'.') {
                chars.next(); // Consume decimal point
                decimals.push(true);
            } else {
                decimals.push(false);
            }
        }
    }

    if digits.is_empty() {
        digits.push('0');
        decimals.push(false);
    }

    // Right-align to 8 cells by padding with spaces on the left
    let total_digits = digits.len();
    let pad_count = MAX_LCD_DIGITS.saturating_sub(total_digits);

    let mut out_digits: [SharedString; MAX_LCD_DIGITS] = Default::default();
    let mut out_decimals: [bool; MAX_LCD_DIGITS] = [false; MAX_LCD_DIGITS];

    for i in 0..pad_count {
        out_digits[i] = SharedString::from(" ");
        out_decimals[i] = false;
    }

    for (i, (&d, &dec)) in digits.iter().zip(decimals.iter()).enumerate() {
        let target_idx = pad_count + i;
        if target_idx < MAX_LCD_DIGITS {
            out_digits[target_idx] = SharedString::from(d.to_string());
            out_decimals[target_idx] = dec;
        }
    }

    (out_digits, out_decimals)
}

/// Updates the Slint UI properties to match the current FSM state.
fn sync_ui(window: &AppWindow, fsm: &CalculatorFsm) {
    let raw_display = fsm.display_string();
    let (digits, decimals) = parse_lcd_cells(&raw_display);

    let digits_model = Rc::new(VecModel::from(digits.to_vec()));
    let decimals_model = Rc::new(VecModel::from(decimals.to_vec()));

    window.set_digits(ModelRc::from(digits_model));
    window.set_decimals(ModelRc::from(decimals_model));

    let flags = fsm.status_flags();
    window.set_has_error(flags.has_error);
    window.set_memory_active(flags.memory_active);
    window.set_is_negative(flags.negative);
    window.set_is_rpn(flags.is_rpn);

    let op_str = match flags.active_operator {
        Some(BinaryOp::Add) => "+",
        Some(BinaryOp::Subtract) => "−",
        Some(BinaryOp::Multiply) => "×",
        Some(BinaryOp::Divide) => "÷",
        None => "",
    };
    window.set_active_op(SharedString::from(op_str));

    // Synchronize paper tape history lines
    let tape_entries: Vec<SharedString> = fsm
        .history()
        .entries()
        .iter()
        .map(|entry| SharedString::from(format!("[#{:03}] {}", entry.id, entry.expression)))
        .collect();
    window.set_tape_lines(ModelRc::from(Rc::new(VecModel::from(tape_entries))));
}

/// Translates a UI key action string to a typed `Key` enum.
fn parse_key_action(action: &str) -> Option<Key> {
    match action {
        "0" => Some(Key::Digit(0)),
        "1" => Some(Key::Digit(1)),
        "2" => Some(Key::Digit(2)),
        "3" => Some(Key::Digit(3)),
        "4" => Some(Key::Digit(4)),
        "5" => Some(Key::Digit(5)),
        "6" => Some(Key::Digit(6)),
        "7" => Some(Key::Digit(7)),
        "8" => Some(Key::Digit(8)),
        "9" => Some(Key::Digit(9)),
        "." => Some(Key::DecimalPoint),
        "+" => Some(Key::BinaryOp(BinaryOp::Add)),
        "-" => Some(Key::BinaryOp(BinaryOp::Subtract)),
        "*" => Some(Key::BinaryOp(BinaryOp::Multiply)),
        "/" => Some(Key::BinaryOp(BinaryOp::Divide)),
        "+/-" => Some(Key::UnaryOp(UnaryOp::Negate)),
        "sqrt" => Some(Key::UnaryOp(UnaryOp::SquareRoot)),
        "%" => Some(Key::UnaryOp(UnaryOp::Percentage)),
        "M+" => Some(Key::MemoryOp(MemoryOp::Add)),
        "M-" => Some(Key::MemoryOp(MemoryOp::Subtract)),
        "MR" => Some(Key::MemoryOp(MemoryOp::Recall)),
        "MC" => Some(Key::MemoryOp(MemoryOp::Clear)),
        "=" => Some(Key::Equals),
        "AC" => Some(Key::Clear),
        "CE" => Some(Key::ClearEntry),
        _ => None,
    }
}

fn main() -> Result<(), slint::PlatformError> {
    let window = AppWindow::new()?;
    let fsm = Rc::new(RefCell::new(CalculatorFsm::new()));
    let current_theme = Rc::new(RefCell::new(DisplayTheme::default()));
    let sound_mgr = Rc::new(SoundManager::new());

    // Initial UI state synchronization & theme setup
    sync_ui(&window, &fsm.borrow());
    apply_theme(&window, *current_theme.borrow());
    window.set_is_muted(sound_mgr.is_muted());

    // Connect keypad and physical keyboard actions
    let window_weak = window.as_weak();
    let fsm_clone = fsm.clone();
    let sound_mgr_click = sound_mgr.clone();

    window.on_key_action(move |action| {
        sound_mgr_click.play_click();
        if let Some(key) = parse_key_action(action.as_str()) {
            fsm_clone.borrow_mut().process_key(key);
            if let Some(win) = window_weak.upgrade() {
                sync_ui(&win, &fsm_clone.borrow());
            }
        }
    });

    let window_weak_clear = window.as_weak();
    let fsm_clone_clear = fsm.clone();
    window.on_clear_tape(move || {
        fsm_clone_clear.borrow_mut().clear_history();
        if let Some(win) = window_weak_clear.upgrade() {
            sync_ui(&win, &fsm_clone_clear.borrow());
        }
    });

    let window_weak_theme = window.as_weak();
    let current_theme_clone = current_theme.clone();
    window.on_cycle_theme(move || {
        let next_theme = current_theme_clone.borrow().next();
        *current_theme_clone.borrow_mut() = next_theme;
        if let Some(win) = window_weak_theme.upgrade() {
            apply_theme(&win, next_theme);
        }
    });

    let window_weak_sound = window.as_weak();
    let sound_mgr_toggle = sound_mgr.clone();
    window.on_toggle_sound(move || {
        let is_muted = sound_mgr_toggle.toggle_mute();
        if let Some(win) = window_weak_sound.upgrade() {
            win.set_is_muted(is_muted);
        }
    });

    let window_weak_mode = window.as_weak();
    let fsm_clone_mode = fsm.clone();
    window.on_toggle_mode(move || {
        fsm_clone_mode.borrow_mut().toggle_mode();
        if let Some(win) = window_weak_mode.upgrade() {
            sync_ui(&win, &fsm_clone_mode.borrow());
        }
    });

    window.run()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_lcd_cells() {
        let (digits, decimals) = parse_lcd_cells("123.45");
        assert_eq!(digits[0].as_str(), " ");
        assert_eq!(digits[1].as_str(), " ");
        assert_eq!(digits[2].as_str(), " ");
        assert_eq!(digits[3].as_str(), "1");
        assert_eq!(digits[4].as_str(), "2");
        assert_eq!(digits[5].as_str(), "3");
        assert_eq!(digits[6].as_str(), "4");
        assert_eq!(digits[7].as_str(), "5");

        assert!(decimals[5]); // The '3' has the decimal point
        assert!(!decimals[6]);
        assert!(!decimals[7]);
    }
}
