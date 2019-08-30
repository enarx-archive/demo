use wasmtime_embed::{wasm_import_wrapper, InstanceToken, RuntimeValue};
use wasmtime_embed_macro::wasm_import;

mod app {
    use super::*;

    /// This trait defines an example interface for the implementations which we
    /// want to import to WASM.
    #[wasm_import]
    pub trait Calc {
        fn add(&self, a: u32, b: u32) -> u32;
        fn sub(&self, a: u32, b: u32) -> u32;
        fn mul(&self, a: u32, b: u32) -> u32;
    }

    pub struct PrintingCalc;

    /// An example implementation of the `Calc` trait
    impl Calc for PrintingCalc {
        fn add(&self, a: u32, b: u32) -> u32 {
            let result = a + b;
            print!("{} + {}", a, b);
            result
        }

        fn sub(&self, a: u32, b: u32) -> u32 {
            let result = a - b;
            print!("{} - {}", a, b);
            result
        }

        fn mul(&self, a: u32, b: u32) -> u32 {
            let result = a * b;
            print!("{} * {}", a, b);
            result
        }
    }
}

fn main() {
    use app::{Calc, PrintingCalc};

    // We need an instance of the implementing struct to import it to WASM
    let printing_calc = PrintingCalc;

    // Call a macro which wraps the struct in a WASM InstanceHandle
    let printing_calc_wasm = wasm_import_wrapper!(printing_calc for <PrintingCalc as Calc>);

    // Run all trait methods.
    // Trait methods are exported by their stringified name in the WASM instance.
    for op in &["add", "sub", "mul"] {
        // Get the invokable function by its exported name
        let op_export = printing_calc_wasm
            .get_export(op)
            .unwrap_or_else(|| panic!("couldn't find {}", op));

        // Invoke the exported function
        let op_result = op_export
            .invoke(&[RuntimeValue::I32(7), RuntimeValue::I32(3)])
            .unwrap_or_else(|e| panic!("couldn't invoke {}: {}", op, e));

        println!(" = {:?}", op_result);
    }
}
