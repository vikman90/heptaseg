use slint::{ModelRc, SharedString, VecModel};
use std::rc::Rc;

slint::include_modules!();

fn parse_lcd_cells(raw: &str) -> ([SharedString; 8], [bool; 8]) {
    let mut digits: Vec<char> = Vec::new();
    let mut decimals: Vec<bool> = Vec::new();

    let mut chars = raw.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch.is_ascii_digit() || ch == '-' || ch == 'E' || ch == 'r' || ch == 'o' {
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

    let total_digits = digits.len();
    let pad_count = 8usize.saturating_sub(total_digits);

    let mut out_digits: [SharedString; 8] = Default::default();
    let mut out_decimals: [bool; 8] = [false; 8];

    for i in 0..pad_count {
        out_digits[i] = SharedString::from(" ");
        out_decimals[i] = false;
    }

    for (i, (&d, &dec)) in digits.iter().zip(decimals.iter()).enumerate() {
        let target_idx = pad_count + i;
        if target_idx < 8 {
            out_digits[target_idx] = SharedString::from(d.to_string());
            out_decimals[target_idx] = dec;
        }
    }

    (out_digits, out_decimals)
}

fn main() {
    let window = AppWindow::new().unwrap();
    let (digits, decimals) = parse_lcd_cells("155");

    let tape_items = vec![
        TapeItem {
            id: 1,
            expression: SharedString::from("120 + 35 ="),
            result: SharedString::from("155"),
        },
        TapeItem {
            id: 2,
            expression: SharedString::from("155 × 2 ="),
            result: SharedString::from("310"),
        },
        TapeItem {
            id: 3,
            expression: SharedString::from("310 ÷ 2 ="),
            result: SharedString::from("155"),
        },
    ];

    window.set_digits(ModelRc::from(Rc::new(VecModel::from(digits.to_vec()))));
    window.set_decimals(ModelRc::from(Rc::new(VecModel::from(decimals.to_vec()))));
    window.set_tape_entries(ModelRc::from(Rc::new(VecModel::from(tape_items))));
    window.set_show_tape(true);
    window.set_memory_active(true);
    window.set_has_error(false);
    window.set_active_op(SharedString::from("+"));

    window.show().unwrap();
    let w = window.window();
    w.set_size(slint::PhysicalSize::new(570, 490));
    let snapshot = w.take_snapshot().expect("Failed to take snapshot");

    let width = snapshot.width();
    let height = snapshot.height();
    let pixels = snapshot.as_slice();

    // Write simple PPM
    let mut header = format!("P6\n{} {}\n255\n", width, height).into_bytes();
    for pixel in pixels {
        header.push(pixel.r);
        header.push(pixel.g);
        header.push(pixel.b);
    }
    std::fs::write("screenshot.ppm", header).unwrap();
    println!("Saved screenshot.ppm ({}x{})", width, height);
}
