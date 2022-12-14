use super::{engine::Engine, root};
use crate::error;
use magnus::{function, Error, Module as _, Object, RString};
use wasmtime::Module as ModuleImpl;

#[derive(Clone)]
#[magnus::wrap(class = "Wasmtime::Module")]
pub struct Module {
    inner: ModuleImpl,
}

impl Module {
    pub fn new(engine: &Engine, wat_or_wasm: RString) -> Self {
        let eng = engine.get();
        // SAFETY: this string is immediately copied and never moved off the stack
        let module = ModuleImpl::new(&eng, unsafe { wat_or_wasm.as_slice() })
            .map_err(|e| error!("Could not build module: {:?}", e.to_string()))
            .unwrap();

        Self { inner: module }
    }

    pub fn get(&self) -> &ModuleImpl {
        &self.inner
    }
}

pub fn init() -> Result<(), Error> {
    let class = root().define_class("Module", Default::default())?;

    class.define_singleton_method("new", function!(Module::new, 2))?;

    Ok(())
}
