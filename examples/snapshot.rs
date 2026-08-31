use heptaseg::core::types::DisplayTheme;
use slint::{ModelRc, SharedString, VecModel};
use std::rc::Rc;

slint::include_modules!();

fn hex_to_color(hex: &str) -> slint::Color {
    let hex = hex.trim_start_matches('#');
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
    slint::Color::from_argb_u8(255, r, g, b)
}

fn apply_theme(window: &AppWindow, theme: DisplayTheme) {
    let (bg, active, ghost, border) = theme.colors();
    window.set_lcd_bg(hex_to_color(bg));
    window.set_active_ink(hex_to_color(active));
    window.set_ghost_ink(hex_to_color(ghost));
    window.set_border_color(hex_to_color(border));
    window.set_theme_name(SharedString::from(theme.name()));
}

fn take_theme_snapshot(theme: DisplayTheme, out_png: &str) {
    let window = AppWindow::new().unwrap();
    let digits = vec![
        SharedString::from("1"),
        SharedString::from("2"),
        SharedString::from("3"),
        SharedString::from("4"),
        SharedString::from("5"),
        SharedString::from("6"),
        SharedString::from("7"),
        SharedString::from("8"),
    ];
    let decimals = vec![false, false, false, false, false, false, false, false];

    window.set_digits(ModelRc::from(Rc::new(VecModel::from(digits))));
    window.set_decimals(ModelRc::from(Rc::new(VecModel::from(decimals))));
    window.set_memory_active(true);
    window.set_has_error(false);
    window.set_active_op(SharedString::from("+"));

    apply_theme(&window, theme);

    window.show().unwrap();
    let w = window.window();
    w.set_size(slint::PhysicalSize::new(330, 490));
    let snapshot = w.take_snapshot().expect("Failed to take snapshot");

    let width = snapshot.width();
    let height = snapshot.height();
    let pixels = snapshot.as_slice();

    let ppm_path = format!("{}.ppm", out_png);
    let mut header = format!("P6\n{} {}\n255\n", width, height).into_bytes();
    for pixel in pixels {
        header.push(pixel.r);
        header.push(pixel.g);
        header.push(pixel.b);
    }
    std::fs::write(&ppm_path, header).unwrap();
}

fn main() {
    take_theme_snapshot(DisplayTheme::ClassicLcd, "classic");
    take_theme_snapshot(DisplayTheme::QuartzLcd, "quartz");
}
