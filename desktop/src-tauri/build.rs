fn main() {
    tauri_build::build();
    // The unit-test executable links the same Windows libraries as the app but, unlike the app,
    // gets no manifest, so it loads an old comctl32 and fails with STATUS_ENTRYPOINT_NOT_FOUND.
    if std::env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc") {
        let dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_default();
        println!("cargo:rustc-link-arg-tests=/MANIFEST:EMBED");
        println!("cargo:rustc-link-arg-tests=/MANIFESTINPUT:{dir}\\windows-test.manifest");
        println!("cargo:rerun-if-changed=windows-test.manifest");
    }
}
