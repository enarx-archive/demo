# Wasmtime Custom WASI Demo
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

## Changelog
This demo is expected to evolve over time.

### v0.1.0
Customizes `fd_write` so that all writes to fd=2 (stdin) are redirected to fd=1 (stdout).
Uses an in-tree copy of `wasmtime-wasi` with customizations to the glue code.

Example run:

```log
$ RUST_LOG=wasmtime_wasi=trace cargo run
   Compiling wasmtime-wasi-custom v0.1.0 (/home/steveej/src/job-redhat/enarx/github_enarx_demo/wasmtime-wasi-custom)
    Finished dev [unoptimized + debuginfo] target(s) in 4.80s
     Running `target/debug/wasmtime-wasi-custom`
[2019-09-19T17:31:42Z INFO  wasmtime_wasi_custom] WASM appliation binary has been compiled from Rust.
[2019-09-19T17:31:42Z INFO  wasmtime_wasi_custom] Loading WASM application binary...
[2019-09-19T17:31:42Z INFO  wasmtime_wasi_custom] WASM application binary loaded.
[2019-09-19T17:31:42Z TRACE wasmtime_wasi_custom] found export: fd_prestat_get/wasi_unstable
[2019-09-19T17:31:42Z TRACE wasmtime_wasi_custom] found export: fd_prestat_dir_name/wasi_unstable
[2019-09-19T17:31:42Z TRACE wasmtime_wasi_custom] found export: environ_sizes_get/wasi_unstable
[2019-09-19T17:31:42Z TRACE wasmtime_wasi_custom] found export: environ_get/wasi_unstable
[2019-09-19T17:31:42Z TRACE wasmtime_wasi_custom] found export: args_sizes_get/wasi_unstable
[2019-09-19T17:31:42Z TRACE wasmtime_wasi_custom] found export: args_get/wasi_unstable
[2019-09-19T17:31:42Z TRACE wasmtime_wasi_custom] found export: fd_write/wasi_unstable
[2019-09-19T17:31:42Z TRACE wasmtime_wasi_custom] found export: proc_exit/wasi_unstable
[2019-09-19T17:31:42Z TRACE wasmtime_wasi_custom] found export: fd_fdstat_get/wasi_unstable
[2019-09-19T17:31:46Z TRACE wasmtime_wasi::syscalls] fd_prestat_get(fd=3, buf=0xffff0)
[2019-09-19T17:31:46Z TRACE wasmtime_wasi::syscalls] environ_sizes_get(environ_count=0xffff0, environ_buf_size=0xffffc)
[2019-09-19T17:31:46Z TRACE wasmtime_wasi::syscalls] args_sizes_get(argc=0xffffc, argv_buf_size=0xffff0)
[2019-09-19T17:31:46Z TRACE wasmtime_wasi::syscalls] fd_write(fd=1, iovs=0xffeb0, iovs_len=1, nwritten=0xffeb8)
Hello, world!
[2019-09-19T17:31:46Z TRACE wasmtime_wasi::syscalls] fd_write(fd=2, iovs=0xffed0, iovs_len=1, nwritten=0xffedc)
[2019-09-19T17:31:46Z DEBUG wasmtime_wasi::syscalls] redirecting stderr to stdout
Hello, stderr!
```