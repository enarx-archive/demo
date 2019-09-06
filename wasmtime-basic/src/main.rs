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

#[cfg(test)]
extern crate test;

use cranelift_codegen::settings;
use cranelift_native;
use std::fs::File;
use std::io::Read;
use wasmtime_jit::{ActionError, ActionOutcome, Context, RuntimeValue};

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
    match result.unwrap() {
        ActionOutcome::Returned { values } => println!("Output: {:#?}", values),
        ActionOutcome::Trapped { message } => println!("Trap from within function: {}", message),
    }
    println!("Done.");
}

// A function to encapsulate the full setup and execution of a single iteration of the WASM demo.
pub fn wasm_add_full() -> Result<ActionOutcome, ActionError> {
    // First, we need to load the WASM binary in.
    let mut binary_file = File::open(concat!(env!("OUT_DIR"), "/add.wasm")).unwrap();
    let mut binary: Vec<u8> = Vec::new();
    binary_file.read_to_end(&mut binary).unwrap();

    // In order to run this binary, we need to prepare a few inputs.

    // First, we need a Wasmtime context. To build one, we need to get an ISA
    // from `cranelift_native.
    let isa_builder = cranelift_native::builder().unwrap();
    let flag_builder = settings::builder();
    let isa = isa_builder.finish(settings::Flags::new(flag_builder));

    // Then, we use the ISA to build the context.
    let mut context = Context::with_isa(isa);

    // Now, we instantiate the WASM module loaded into memory.
    let mut instance = context.instantiate_module(None, &binary).unwrap();

    // And, finally, invoke our function and print the results.
    // For this demo, all we're doing is adding 5 and 7 together.
    let args = [RuntimeValue::I32(5), RuntimeValue::I32(7)];
    context.invoke(&mut instance, "add", &args)
}

// A Rust function that does the same thing as the WASM demo, for benchmarking use.
#[cfg(test)]
pub fn native_add(a: i32, b: i32) -> i32 {
    a + b
}

// Performance benchmarking for the demo.
#[cfg(test)]
mod tests {
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

        // First, we need a Wasmtime context. To build one, we need to get an ISA
        // from `cranelift_native.
        let isa_builder = cranelift_native::builder().unwrap();
        let flag_builder = settings::builder();
        let isa = isa_builder.finish(settings::Flags::new(flag_builder));

        // Then, we use the ISA to build the context.
        let mut context = Context::with_isa(isa);

        // Now, we instantiate the WASM module loaded into memory.
        let mut instance = context.instantiate_module(None, &binary).unwrap();

        // And, finally, invoke our function and print the results.
        // For this demo, all we're doing is adding 5 and 7 together.
        let args = [RuntimeValue::I32(5), RuntimeValue::I32(7)];
        b.iter(|| context.invoke(&mut instance, "add", &args));
    }
}
