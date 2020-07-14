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

mod wasm {
    include!("../../common/src/wasm.rs");
}

use wasm::enarx_syscalls;

fn main() -> ! {
    enarx_syscalls::socket_udp_bind("127.0.0.1:34254");
    loop {
        let mut data_buf = [0; 1024];
        let mut src_buf = [0; 1024];

        let len = enarx_syscalls::socket_udp_receive(&mut data_buf, &mut src_buf);
        let data = {
            let data_str = std::str::from_utf8(&data_buf).unwrap();
            let newline_index = data_str.find("\n").unwrap_or(data_str.len());
            data_str.split_at(newline_index).0
        };
        let src = std::str::from_utf8(&src_buf)
            .unwrap()
            .split("\0")
            .nth(0)
            .unwrap();
        enarx_syscalls::println(&format!("received {} bytes: '{}' from {}", len, data, src));
        enarx_syscalls::println("sending back...");
        enarx_syscalls::socket_udp_send_to(format!("{}\n", data).as_bytes(), src.as_bytes());
    }
}
