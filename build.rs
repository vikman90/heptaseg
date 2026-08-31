fn main() {
    println!("cargo:rerun-if-changed=ui");
    slint_build::compile("ui/appwindow.slint").expect("Slint build failed");
}
