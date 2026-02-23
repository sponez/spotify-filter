fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap() == "windows" {
        let mut res = winres::WindowsResource::new();
        res.set_icon("resources/icon.ico");
        res.set("ProductName", "Spotify Filter");
        res.set("FileDescription", "Spotify Filter");
        res.set("LegalCopyright", "");
        res.compile().expect("Failed to compile Windows resources");
    }
}
