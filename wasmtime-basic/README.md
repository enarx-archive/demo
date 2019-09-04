# Wasmtime Basic Demo

This is an example crate demonstrating how Enarx may use Wasmtime, a Rust-powered JIT, to natively run programs from several different source languages (Rust/C/C++) compiled to WASM.

## Running the demo

By default, the Rust demo should run with `cargo run` out of the box.

If you wish to compile the C demo, you'll need to install `lld` . On Fedora, this can be accomplished with `sudo dnf install lld`.

### Switching Languages

The WASM binary used for the demo can be compiled from either Rust or C. Compilation of this binary happens at build time, and the source language can be controlled via Cargo features.

Rust is compiled by default. If C is desired, the appropriate feature can be invoked via the `cargo` command:

```
cargo run --no-default-features --features c
```

## Running benchmarks

`main.rs` includes performance benchmarks useful for analyzing the startup cost and runtime of functions in Wasmtime.

These benchmarks use Rust's unstable built-in benchmarking tools, and must be run on Rust nightly. Additionally, the `benchmark` feature must be enabled to run them:

```
cargo +nightly bench --features benchmark
```