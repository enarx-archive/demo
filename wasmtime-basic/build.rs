// Copyright 2019 Red Hat
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//      http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

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
