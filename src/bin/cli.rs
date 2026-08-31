//! Heptaseg CLI: Terminal pocket calculator with Standard & RPN modes.

use std::env;
use std::io::{self, BufRead, Write};

use heptaseg::core::fsm::CalculatorFsm;
use heptaseg::core::register::MAX_LCD_DIGITS;
use heptaseg::core::rpn::RpnCalculator;
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
fn print_terminal_lcd(
    mode_name: &str,
    display_str: &str,
    mem_active: bool,
    neg: bool,
    err: bool,
    op: Option<BinaryOp>,
) {
    let (digits, decimals) = format_display_slots(display_str);

    let mem_flag = if mem_active { "[M]" } else { "   " };
    let neg_flag = if neg { "[-]" } else { "   " };
    let err_flag = if err { "[ERR]" } else { "     " };
    let op_flag = match op {
        Some(BinaryOp::Add) => "[+]",
        Some(BinaryOp::Subtract) => "[-]",
        Some(BinaryOp::Multiply) => "[*]",
        Some(BinaryOp::Divide) => "[/]",
        None => "   ",
    };

    println!("\x1B[2J\x1B[1;1H"); // Clear screen
    println!("╔══════════════════════════════════════════════════╗");
    println!(
        "║  HEPTASEG TERMINAL CALCULATOR ({:>8})         ║",
        mode_name
    );
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
    println!(
        " Commands: [0-9] [.] [+ - * /] [=] [ac] [ce] [m+ m- mr mc] [sqrt] [%] [tape] [mode] [q]"
    );
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
    let mut use_rpn = env::args().any(|arg| arg == "--rpn");
    let mut standard_fsm = CalculatorFsm::new();
    let mut rpn_calc = RpnCalculator::new();

    let stdin = io::stdin();
    let mut reader = stdin.lock();

    let draw_screen = |is_rpn: bool, std: &CalculatorFsm, rpn: &RpnCalculator| {
        if is_rpn {
            let flags = rpn.status_flags();
            print_terminal_lcd(
                "RPN MODE",
                &rpn.display_string(),
                flags.memory_active,
                flags.negative,
                flags.has_error,
                flags.active_operator,
            );
        } else {
            let flags = std.status_flags();
            print_terminal_lcd(
                "STANDARD",
                &std.display_string(),
                flags.memory_active,
                flags.negative,
                flags.has_error,
                flags.active_operator,
            );
        }
    };

    draw_screen(use_rpn, &standard_fsm, &rpn_calc);

    let mut line = String::new();
    while reader.read_line(&mut line).unwrap() > 0 {
        let trimmed = line.trim();
        if trimmed == "q" || trimmed == "quit" || trimmed == "exit" {
            println!("Goodbye!");
            break;
        }

        if trimmed == "mode" || trimmed == "rpn" || trimmed == "std" {
            use_rpn = !use_rpn;
            draw_screen(use_rpn, &standard_fsm, &rpn_calc);
            line.clear();
            continue;
        }

        if trimmed == "tape" || trimmed == "history" {
            println!("\n--- 📜 PAPER TAPE AUDIT TRAIL ---");
            let entries = if use_rpn {
                rpn_calc.history()
            } else {
                standard_fsm.history()
            };
            if entries.is_empty() {
                println!("(Tape is empty)");
            } else {
                for entry in entries {
                    println!(
                        "#{:02} {:<24} = {:>10}",
                        entry.id, entry.expression, entry.result
                    );
                }
            }
            println!("--------------------------------\nPress Enter to continue...");
            let mut dummy = String::new();
            let _ = reader.read_line(&mut dummy);
            draw_screen(use_rpn, &standard_fsm, &rpn_calc);
            line.clear();
            continue;
        }

        for token in trimmed.split_whitespace() {
            let keys = parse_cli_token(token);
            for key in keys {
                if use_rpn {
                    rpn_calc.process_key(key);
                } else {
                    standard_fsm.process_key(key);
                }
            }
        }

        draw_screen(use_rpn, &standard_fsm, &rpn_calc);
        line.clear();
    }
}
