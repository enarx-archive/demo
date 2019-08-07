//! This is an example crate demonstrating how Enarx may use Wasmtime, a
//! Rust-powered JIT, to natively run programs from several different source
//! languages (Rust/C/C++) compiled to WASI-compliant WASM.

use cranelift_codegen::settings;
use cranelift_native;
use std::fs::File;
use std::io::Read;
use wasmtime_jit::{ActionOutcome, Context, RuntimeValue};

fn main() {
    // Before we begin, it'll be heplful to know which source language we're
    // running, so let's print it out.
    if cfg!(feature = "c") {
        println!("WASM binary has been compiled from C.");
    }
    if cfg!(feature = "rust") {
        println!("WASM binary has been compiled from Rust.");
    }

    println!("Loading WASM binary...");
    let mut binary_file = File::open(concat!(env!("OUT_DIR"), "/add.wasm")).unwrap();
    let mut binary: Vec<u8> = Vec::new();
    binary_file.read_to_end(&mut binary).unwrap();
    println!("WASM binary loaded.");

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
    println!("Invoking function.");
    let args = [RuntimeValue::I32(5), RuntimeValue::I32(7)];
    let result = context.invoke(&mut instance, "add", &args);
    match result.unwrap() {
        ActionOutcome::Returned { values } => println!("Output: {:#?}", values),
        ActionOutcome::Trapped { message } => println!("Trap from within function: {}", message),
    }
    println!("Done.");
}
