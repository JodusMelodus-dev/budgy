fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").expect("Failed to get os") == "windows" {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/favicon.ico");
        res.compile().expect("Failed to compile icon");
    }
}
