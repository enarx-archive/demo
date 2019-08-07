# Wasmtime Basic Demo

This is an example crate demonstrating how Enarx may use Wasmtime, a Rust-powered JIT, to natively run programs from several different source languages (Rust/C/C++) compiled to WASI-compliant WASM.

## Running the demo

By default, the Rust demo should run with `cargo run` out of the box.

If you wish to compile the C demo, you'll need to install `lld` . On Fedora, this can be accomplished with `sudo dnf install lld`.

### Switching Languages

The WASM binary used for the demo can be compiled from either Rust or C. Compilation of this binary happens at build time, and the source language can be controlled via Cargo features.

Rust is compiled by default. If C is desired, the appropriate feature can be invoked via the `cargo` command:

```
cargo run --no-default-features --features c
```

Or, alternatively, by changing the `default` feature in `Cargo.toml` from `["rust"]` to `["c"]`.