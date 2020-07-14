# Wasmtime UDP Echo-Server Demo
This demo shows the extension of the previously introduced _stdio_ syscall
module, and adds a _net_ syscall module, which implements rudimentary
UDP functionality.
The WASM guest application uses these syscalls to build an UDP echo
server.

The WASM application lives in _app.rs_, and is compiled as per the
instructions in _build.rs_.

## Running the demo
The demo runs the UDP echo-server with `cargo run` out of the box.

To send something to it from the host, `nc` can be used as in the following example:

```console
$ nc -4u localhost 34254
Hello, WASM guest!
Hello, WASM guest!
^C
```

For the above example, the server logs should look something like this:

```console
[RUNTIME] App imports: {"enarx_syscalls_net/__socket_udp_bind": 0, "enarx_syscalls_net/__socket_udp_receive": 1, "enarx_syscalls_stdio/__print_str": 2, "enarx_syscalls_net/__socket_udp_send_to": 3}
[RUNTIME] Including export 'enarx_syscalls_stdio/__print_str' at index 2
[RUNTIME] Including export 'enarx_syscalls_net/__socket_udp_bind' at index 0
[RUNTIME] Including export 'enarx_syscalls_net/__socket_udp_receive' at index 1
[RUNTIME] Including export 'enarx_syscalls_net/__socket_udp_send_to' at index 3
[RUNTIME] receiving on UdpSocket { addr: V4(127.0.0.1:34254), fd: 3 }
[RUNTIME] received 19 bytes from 127.0.0.1:58520
[WASM] received 19 bytes: 'Hello, WASM guest!' from 127.0.0.1:58520
[WASM] sending back...
[RUNTIME] sent 19 bytes to 127.0.0.1:58520
[RUNTIME] receiving on UdpSocket { addr: V4(127.0.0.1:34254), fd: 3 }
```