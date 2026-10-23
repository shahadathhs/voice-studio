//! whisper.cpp's Metal code uses `@available(macOS ...)` checks, which clang compiles into calls to
//! `___isPlatformVersionAtLeast`. That helper lives in clang's runtime library, and Rust links
//! without the default libraries, so on macOS the runtime is linked explicitly. Without this the
//! desktop app fails to link in release builds.

use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("macos") {
        return;
    }
    let Ok(output) = Command::new("cc").arg("--print-resource-dir").output() else {
        return;
    };
    let resource_dir = String::from_utf8_lossy(&output.stdout);
    println!(
        "cargo:rustc-link-search=native={}/lib/darwin",
        resource_dir.trim()
    );
    println!("cargo:rustc-link-lib=static=clang_rt.osx");
}
