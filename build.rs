fn main() {
    unsafe {
        std::env::set_var("SLINT_STYLE", "fluent");
    }
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap() == "windows" {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/icon.ico");
        res.compile().unwrap();
    }
    slint_build::compile("ui/app-window.slint").expect("Slint build failed");
}
