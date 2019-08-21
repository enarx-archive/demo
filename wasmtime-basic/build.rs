use std::process::Command;

fn main() {
    let out = std::env::var("OUT_DIR").unwrap();

    if cfg!(feature = "c") {
        println!("Compiling C source to WASM...");
        let cc = std::env::var("CC").unwrap_or("clang".to_owned());
        assert!(Command::new(cc)
            .arg("-o")
            .arg(format!("{}/add.wasm", out))
            .arg("--target=wasm32")
            .arg("-Wl,--export-all")
            .arg("-Wl,--no-entry")
            .arg("-nostdlib")
            .arg("-O3")
            .arg("src/add.c")
            .status()
            .expect("failed to compile WASM module")
            .success());
    }

    if cfg!(feature = "rust") {
        println!("Compiling Rust source to WASM...");
        assert!(Command::new("rustc")
            .arg("-C")
            .arg("lto")
            .arg("-C")
            .arg("opt-level=3")
            .arg("--target")
            .arg("wasm32-unknown-unknown")
            .arg("-o")
            .arg(format!("{}/add.wasm", out))
            .arg("src/add.rs")
            .status()
            .expect("failed to compile WASM module")
            .success());
    }
}
