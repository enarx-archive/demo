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

//! This is an example crate demonstrating how Enarx may use Wasmtime, a
//! Rust-powered JIT, to natively run programs from several different source
//! languages (Rust/C/C++) compiled to WASI-compliant WASM.

#![cfg_attr(feature = "benchmark", feature(test))]

use std::fs::File;
use std::io::Read;
use wasmtime::{Engine, Store, Val, Module, Instance, Trap};

// The basic WASM demo itself.
fn main() {
    // Before we begin, it'll be heplful to know which source language we're
    // running, so let's print it out.
    if cfg!(feature = "c") {
        println!("WASM binary has been compiled from C.");
    }
    if cfg!(feature = "rust") {
        println!("WASM binary has been compiled from Rust.");
    }

    println!("Loading and running WASM binary...");
    let result = wasm_add_full();
    println!("Finished. Results:");
    match result {
        Ok(values) => println!("Output: {:?}", values[0]),
        Err(e) => println!("Trap from within function: {}", e.message()),
    }
    println!("Done.");
}

// A function to encapsulate the full setup and execution of a single iteration of the WASM demo.
pub fn wasm_add_full() -> Result<Box<[Val]>, Trap> {
    // First, we need to load the WASM binary in.
    let mut binary_file = File::open(concat!(env!("OUT_DIR"), "/add.wasm")).unwrap();
    let mut binary: Vec<u8> = Vec::new();
    binary_file.read_to_end(&mut binary).unwrap();

    // In order to run this binary, we need to prepare a few inputs.
    // First, we need to create an Engine which is a global context for
    // compilation and management of wasm modules.
    let engine = Engine::default();
    // Next we create a shared cache for wasm modules.
    let store = Store::new(&engine);
    // Next we create and compile our wasm module.
    let module = Module::new(&store, binary).unwrap();

    // Next we create a new instance of our Module.
    let instance = Instance::new(&module, &[]).unwrap();
    // And finally we get the function that we want to call.
    let func = instance.get_export("add").unwrap().func().unwrap();

    // And, finally, invoke our function and print the results.
    // For this demo, all we're doing is adding 5 and 7 together.
    let args = [Val::I32(5), Val::I32(7)];
    func.call(&args)
}

// A Rust function that does the same thing as the WASM demo, for benchmarking use.
#[cfg(test)]
pub fn native_add(a: i32, b: i32) -> i32 {
    a + b
}

// Performance benchmarking for the demo.
#[cfg(all(feature = "benchmark", test))]
mod tests {
    extern crate test;
    use super::*;
    use test::Bencher;

    // Baseline benchmark: Simple Rust adition.
    #[bench]
    fn bench_native_add(b: &mut Bencher) {
        b.iter(|| native_add(5, 7));
    }

    // Benchmark: WASM addition from start to finish, including
    // loading/instantiating WASM.
    #[bench]
    fn bench_wasm_add_unloaded(b: &mut Bencher) {
        b.iter(|| wasm_add_full());
    }

    // Benchmark: WASM addition once everything is set up.
    #[bench]
    fn bench_wasm_add_loaded(b: &mut Bencher) {
        let mut binary_file = File::open(concat!(env!("OUT_DIR"), "/add.wasm")).unwrap();
        let mut binary: Vec<u8> = Vec::new();
        binary_file.read_to_end(&mut binary).unwrap();

        // In order to run this binary, we need to prepare a few inputs.
        // First, we need to create an Engine which is a global context for
        // compilation and management of wasm modules.
        let engine = Engine::default();
        // Next we create a shared cache for wasm modules.
        let store = Store::new(&engine);
        // Next we create and compile our wasm module.
        let module = Module::new(&store, binary).unwrap();

        // Next we create a new instance of our Module.
        let instance = Instance::new(&module, &[]).unwrap();
        // And finally we get the function that we want to call.
        let func = instance.get_export("add").unwrap().func().unwrap();

        // And, finally, invoke our function and print the results.
        // For this demo, all we're doing is adding 5 and 7 together.
        let args = [Val::I32(5), Val::I32(7)];
        b.iter(|| func.call(&args));
    }
}
