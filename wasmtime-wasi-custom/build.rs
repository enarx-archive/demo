use std::process::Command;

fn main() {
    let out = std::env::var("OUT_DIR").unwrap();

    if cfg!(feature = "c") {
        println!("Compiling C app to WASM...");
        let cc = std::env::var("CC").unwrap_or("clang".to_owned());
        assert!(Command::new(cc)
            .args(&["-o", &format!("{}/app.wasm", out)])
            .arg("--target=wasm32-unknown-wasi")
            .arg("-Wl,--export-all")
            .arg("-Wl,--no-entry")
            .arg("-nostdlib")
            .arg("-O3")
            .arg("src/app.c")
            .status()
            .expect("failed to compile WASM module")
            .success());
    }

    if cfg!(feature = "rust") {
        println!("Compiling Rust app to WASM...");
        assert!(Command::new("rustc")
            // .args(&["-C", "lto"])
            // .args(&["-C", "opt-level=3"])
            // .args(&["--target", "wasm32-unknown-unknown"])
            .args(&["--target", "wasm32-wasi"])
            .args(&["-o", &format!("{}/app.wasm", out)])
            .arg("src/app.rs")
            .status()
            .expect("failed to compile WASM module")
            .success());
    }
}
