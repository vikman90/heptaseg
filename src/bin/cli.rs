//! Heptaseg CLI: Terminal pocket calculator powered by the same decoupled FSM.

use std::io::{self, BufRead, Write};

use heptaseg::core::fsm::CalculatorFsm;
use heptaseg::core::register::MAX_LCD_DIGITS;
use heptaseg::core::types::{BinaryOp, Key, MemoryOp, UnaryOp};

/// Renders a single digit into 3 vertical slices for ASCII 7-segment display.
fn render_digit_slices(ch: char, has_decimal: bool) -> [&'static str; 3] {
    let base: [&'static str; 3] = match ch {
        '0' => [" _ ", "| |", "|_|"],
        '1' => ["   ", "  |", "  |"],
        '2' => [" _ ", " _|", "|_ "],
        '3' => [" _ ", " _|", " _|"],
        '4' => ["   ", "|_|", "  |"],
        '5' => [" _ ", "|_ ", " _|"],
        '6' => [" _ ", "|_ ", "|_|"],
        '7' => [" _ ", "  |", "  |"],
        '8' => [" _ ", "|_|", "|_|"],
        '9' => [" _ ", "|_|", " _|"],
        '-' => ["   ", " _ ", "   "],
        'E' => [" _ ", "|_ ", "|_ "],
        _ => ["   ", "   ", "   "],
    };

    if has_decimal {
        match ch {
            '0' => [" _ ", "| |", "|_|."],
            '1' => ["   ", "  |", "  |."],
            '2' => [" _ ", " _|", "|_ ."],
            '3' => [" _ ", " _|", " _|."],
            '4' => ["   ", "|_|", "  |."],
            '5' => [" _ ", "|_ ", " _|."],
            '6' => [" _ ", "|_ ", "|_|."],
            '7' => [" _ ", "  |", "  |."],
            '8' => [" _ ", "|_|", "|_|."],
            '9' => [" _ ", "|_|", " _|."],
            _ => base,
        }
    } else {
        base
    }
}

/// Formats the raw display string into 8 aligned digit chars and decimal booleans.
fn format_display_slots(raw: &str) -> ([char; MAX_LCD_DIGITS], [bool; MAX_LCD_DIGITS]) {
    let mut digits: Vec<char> = Vec::new();
    let mut decimals: Vec<bool> = Vec::new();

    let mut chars = raw.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch.is_ascii_digit() || ch == '-' || ch == 'E' {
            digits.push(ch);
            if chars.peek() == Some(&'.') {
                chars.next();
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

    let pad_count = MAX_LCD_DIGITS.saturating_sub(digits.len());
    let mut out_digits = [' '; MAX_LCD_DIGITS];
    let mut out_decimals = [false; MAX_LCD_DIGITS];

    for (i, (&d, &dec)) in digits.iter().zip(decimals.iter()).enumerate() {
        let idx = pad_count + i;
        if idx < MAX_LCD_DIGITS {
            out_digits[idx] = d;
            out_decimals[idx] = dec;
        }
    }

    (out_digits, out_decimals)
}

/// Renders the full retro ASCII LCD panel to stdout.
fn print_terminal_lcd(fsm: &CalculatorFsm) {
    let flags = fsm.status_flags();
    let (digits, decimals) = format_display_slots(&fsm.display_string());

    let mem_flag = if flags.memory_active { "[M]" } else { "   " };
    let neg_flag = if flags.negative { "[-]" } else { "   " };
    let err_flag = if flags.has_error { "[ERR]" } else { "     " };
    let op_flag = match flags.active_operator {
        Some(BinaryOp::Add) => "[+]",
        Some(BinaryOp::Subtract) => "[-]",
        Some(BinaryOp::Multiply) => "[*]",
        Some(BinaryOp::Divide) => "[/]",
        None => "   ",
    };

    println!("\x1B[2J\x1B[1;1H"); // Clear screen
    println!("╔══════════════════════════════════════════════════╗");
    println!("║  HEPTASEG TERMINAL CALCULATOR                    ║");
    println!("╠══════════════════════════════════════════════════╣");
    println!(
        "║  {} {} {} {:>28}  ║",
        mem_flag, neg_flag, err_flag, op_flag
    );

    // Render 3 ASCII lines of 7-segment cells
    for line_idx in 0..3 {
        print!("║  ");
        for i in 0..MAX_LCD_DIGITS {
            let slices = render_digit_slices(digits[i], decimals[i]);
            let s = slices[line_idx];
            if s.len() == 4 {
                print!("{} ", s);
            } else {
                print!("{}  ", s);
            }
        }
        println!(" ║");
    }
    println!("╚══════════════════════════════════════════════════╝");
    println!(" Commands: [0-9] [.] [+ - * /] [=] [ac] [ce] [m+ m- mr mc] [sqrt] [%] [q]");
    print!(" > ");
    io::stdout().flush().unwrap();
}

fn parse_cli_token(token: &str) -> Vec<Key> {
    match token.to_lowercase().as_str() {
        "ac" | "c" | "clear" => vec![Key::Clear],
        "ce" => vec![Key::ClearEntry],
        "=" | "enter" => vec![Key::Equals],
        "+" => vec![Key::BinaryOp(BinaryOp::Add)],
        "-" => vec![Key::BinaryOp(BinaryOp::Subtract)],
        "*" | "x" => vec![Key::BinaryOp(BinaryOp::Multiply)],
        "/" => vec![Key::BinaryOp(BinaryOp::Divide)],
        "m+" => vec![Key::MemoryOp(MemoryOp::Add)],
        "m-" => vec![Key::MemoryOp(MemoryOp::Subtract)],
        "mr" => vec![Key::MemoryOp(MemoryOp::Recall)],
        "mc" => vec![Key::MemoryOp(MemoryOp::Clear)],
        "sqrt" | "v" => vec![Key::UnaryOp(UnaryOp::SquareRoot)],
        "%" => vec![Key::UnaryOp(UnaryOp::Percentage)],
        "+/-" | "neg" => vec![Key::UnaryOp(UnaryOp::Negate)],
        other => {
            // Parse sequence of characters (e.g. "123.45")
            let mut keys = Vec::new();
            for ch in other.chars() {
                if let Some(digit) = ch.to_digit(10) {
                    keys.push(Key::Digit(digit as u8));
                } else if ch == '.' || ch == ',' {
                    keys.push(Key::DecimalPoint);
                } else if ch == '+' {
                    keys.push(Key::BinaryOp(BinaryOp::Add));
                } else if ch == '-' {
                    keys.push(Key::BinaryOp(BinaryOp::Subtract));
                } else if ch == '*' {
                    keys.push(Key::BinaryOp(BinaryOp::Multiply));
                } else if ch == '/' {
                    keys.push(Key::BinaryOp(BinaryOp::Divide));
                } else if ch == '=' {
                    keys.push(Key::Equals);
                }
            }
            keys
        }
    }
}

fn main() {
    let mut fsm = CalculatorFsm::new();
    let stdin = io::stdin();
    let mut reader = stdin.lock();

    print_terminal_lcd(&fsm);

    let mut line = String::new();
    while reader.read_line(&mut line).unwrap() > 0 {
        let trimmed = line.trim();
        if trimmed == "q" || trimmed == "quit" || trimmed == "exit" {
            println!("Goodbye!");
            break;
        }

        for token in trimmed.split_whitespace() {
            let keys = parse_cli_token(token);
            for key in keys {
                fsm.process_key(key);
            }
        }

        print_terminal_lcd(&fsm);
        line.clear();
    }
}
