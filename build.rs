fn main() {
    println!("cargo:rerun-if-changed=assets/icon.ico");
    // Embeds the icon and file details in the Windows executable.
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/icon.ico");
        res.set("ProductName", "Swarf");
        res.set("FileDescription", "Swarf speeds and feeds calculator");
        if let Err(e) = res.compile() {
            println!("cargo:warning=icon not embedded in the executable: {e}");
        }
    }
}
