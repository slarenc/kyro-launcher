fn main() {
    println!("cargo:rerun-if-changed=icons/icon.ico");
    println!("cargo:rerun-if-changed=build.rs");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        winresource::WindowsResource::new()
            .set_icon("icons/icon.ico")
            .set("ProductName", "KYRO Launcher")
            .set("FileDescription", "KYRO Launcher Native")
            .compile()
            .expect("Windows application resources");
    }
}
