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

fn main() {
    let (a, b) = (5, 7);
    let result = a + b;

    enarx_syscalls::println(&format!("{} + {} = {}", a, b, result));
}
