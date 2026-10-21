fn main() {
    println!("cargo:rerun-if-changed=src/assets/app.ico");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        #[cfg(windows)]
        winresource::WindowsResource::new()
            .set_icon("src/assets/app.ico")
            .compile()
            .expect("failed to embed the Windows icon");
    }
}
