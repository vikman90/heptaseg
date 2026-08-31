use slint::{ModelRc, SharedString, VecModel};
use std::rc::Rc;

slint::include_modules!();

fn main() {
    // Set headless backend for testing if display server isn't running or use default
    let window = AppWindow::new().unwrap();
    let digits = vec![
        SharedString::from("0"),
        SharedString::from("1"),
        SharedString::from("2"),
        SharedString::from("3"),
        SharedString::from("4"),
        SharedString::from("5"),
        SharedString::from("8"),
        SharedString::from("0"),
    ];
    let decimals = vec![false, false, false, true, false, false, false, false];

    window.set_digits(ModelRc::from(Rc::new(VecModel::from(digits))));
    window.set_decimals(ModelRc::from(Rc::new(VecModel::from(decimals))));
    window.set_memory_active(true);
    window.set_has_error(false);
    window.set_active_op(SharedString::from("+"));

    window.show().unwrap();
    let w = window.window();
    w.set_size(slint::PhysicalSize::new(330, 490));
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
