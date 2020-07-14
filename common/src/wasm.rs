pub mod enarx_syscalls {
    use std::convert::TryInto;

    #[link(wasm_import_module = "enarx_syscalls_stdio")]
    extern "C" {
        fn __print_str(ptr: *const u8, len: u64);
    }

    #[link(wasm_import_module = "enarx_syscalls_net")]
    extern "C" {
        fn __socket_udp_bind(ptr: *const u8, len: u64);

        fn __socket_udp_receive(
            data_ptr: *mut u8,
            data_len: u64,
            src_ptr: *mut u8,
            src_len: u64,
        ) -> u64;

        fn __socket_udp_send_to(
            data_ptr: *const u8,
            data_len: u64,
            dst_ptr: *const u8,
            dst_len: u64,
        ) -> u64;
    }

    fn _print_str(s: &str) {
        unsafe { __print_str(s.as_ptr(), s.len().try_into().unwrap()) }
    }

    pub fn print(s: &str) {
        _print_str(s)
    }

    pub fn println(s: &str) {
        _print_str(&format!("{}\n", s));
    }

    pub fn socket_udp_bind(addr: &str) {
        unsafe { __socket_udp_bind(addr.as_ptr(), addr.len().try_into().unwrap()) }
    }

    pub fn socket_udp_receive(content_buf: &mut [u8], source_buf: &mut [u8]) -> u64 {
        unsafe {
            __socket_udp_receive(
                content_buf.as_mut_ptr(),
                content_buf.len().try_into().unwrap(),
                source_buf.as_mut_ptr(),
                source_buf.len().try_into().unwrap(),
            )
        }
    }

    pub fn socket_udp_send_to(data: &[u8], dst: &[u8]) -> u64 {
        unsafe {
            __socket_udp_send_to(
                data.as_ptr(),
                data.len().try_into().unwrap(),
                dst.as_ptr(),
                dst.len().try_into().unwrap(),
            )
        }
    }
}

pub mod wasmstr {
    use std::convert::TryInto;

    pub struct WasmStr(pub *const u8, pub u64);

    impl WasmStr {
        pub fn to_str(&self) -> &str {
            self.into()
        }
    }

    impl std::convert::From<&WasmStr> for &str {
        fn from(wasm_string: &WasmStr) -> Self {
            std::str::from_utf8(unsafe {
                std::slice::from_raw_parts(wasm_string.0, wasm_string.1.try_into().unwrap())
            })
            .unwrap()
        }
    }
}
