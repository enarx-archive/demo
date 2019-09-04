use log::{info, trace};
use std::collections::HashMap;
use std::error::Error;
use std::fs::read;
use wasmtime_api;

fn main() -> Result<(), Box<dyn Error>> {
    let _ = env_logger::try_init_from_env(env_logger::Env::default());

    // Before we begin, it'll be heplful to know which source language we're
    // running, so let's print it out.
    if cfg!(feature = "c") {
        info!("WASM application binary has been compiled from C.");
    }
    if cfg!(feature = "rust") {
        info!("WASM appliation binary has been compiled from Rust.");
    }

    // Load and instnatiate the WASM app
    info!("Loading WASM application binary...");
    let app_wasm = read(concat!(env!("OUT_DIR"), "/app.wasm")).unwrap();
    info!("WASM application binary loaded.");

    // wasmtime-api variables
    let engine = wasmtime_api::HostRef::new(wasmtime_api::Engine::default());
    let store = wasmtime_api::HostRef::new(wasmtime_api::Store::new(engine));
    let mut module_registry: HashMap<String, (wasmtime_api::Instance, HashMap<String, usize>)> =
        HashMap::new();

    // Instantiate WASI
    let custom_wasi = wasmtime_wasi::instantiate_wasi(
        // prefix: &str,
        "",
        // global_exports: Rc<RefCell<HashMap<String, Option<wasmtime_runtime::Export>>>>,
        store.borrow().global_exports().clone(),
        // preopened_dirs: &[(String, File)],
        Default::default(),
        // argv: &[String],
        Default::default(),
        // environ: &[(String, String)],
        Default::default(),
    )
    .expect("could nt instantiate WASI");
    module_registry.insert(
        "wasi_unstable".to_string(),
        wasmtime_api::Instance::from_handle(store.clone(), custom_wasi)?,
    );

    let app_module = wasmtime_api::HostRef::new(wasmtime_api::Module::new(store.clone(), &app_wasm)?);

    // Resolve import using module_registry.
    let imports = {
        // let module_borrow: std::cell::Ref<'_, wasmtime_api::Module> = app_module.borrow();

        app_module
            .borrow()
            .imports()
            .iter()
            .map(|i| {
                let module_name = i.module().to_string();
                if let Some((instance, map)) = module_registry.get(&module_name) {
                    let field_name = i.name().to_string();
                    if let Some(export_index) = map.get(&field_name) {
                        trace!("found export: {}/{}", field_name, module_name);
                        Ok(instance.exports()[*export_index].clone())
                    } else {
                        Err(format!(
                            "Import {} was not found in module {}",
                            field_name, module_name
                        ))
                    }
                } else {
                    Err(format!("Import module {} was not found", module_name))
                }
            })
            .collect::<Result<Vec<_>, _>>()?
    };

    let instance = wasmtime_api::HostRef::new(wasmtime_api::Instance::new(
        store.clone(),
        app_module.clone(),
        &imports,
    )?);

    let mut handle = instance.borrow().handle().clone();
    let mut context = store.borrow().engine().borrow().create_wasmtime_context();
    let module_data = wasmtime_interface_types::ModuleData::new(&app_wasm)?;

    // Invoke the function
    module_data.invoke(&mut context, &mut handle, "hello_world", Default::default())?;

    Ok(())
}
