fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        let mut res = winres::WindowsResource::new();
        res.set_icon(&format!("{manifest_dir}/../assets/icon-dark.ico"));
        if let Err(e) = res.compile() {
            eprintln!("cargo:warning=winres failed to embed icon: {e}");
        }
    }
}
