use super::{root, store::Store};
use magnus::{
    memoize, method, r_typed_data::DataTypeBuilder, DataTypeFunctions, Error, Module, RClass,
    Symbol, TypedData,
};
use wasmtime::{AsContext, Export as ExportImpl, ExternType};

pub struct Export<'instance> {
    export: ExportImpl<'instance>,
    store: &'instance Store,
}

unsafe impl Send for Export<'_> {}

unsafe impl<'instance> TypedData for Export<'instance> {
    fn class() -> magnus::RClass {
        *memoize!(RClass: root().define_class("Export", Default::default()).unwrap())
    }

    fn data_type() -> &'static magnus::DataType {
        magnus::memoize!(magnus::DataType: {
            let mut builder = DataTypeBuilder::<Export>::new("Wasmtime::Export");
            builder.free_immediatly();
            builder.build()
        })
    }
}

impl DataTypeFunctions for Export<'_> {}

impl<'instance> Export<'instance> {
    pub fn new(store: &'instance Store, export: ExportImpl<'instance>) -> Self {
        Self { store, export }
    }

    pub fn get(&self) -> &ExportImpl {
        &self.export
    }

    pub fn name(&self) -> Symbol {
        self.get().name().into()
    }

    pub fn type_name(&self) -> Symbol {
        let s = self.store.borrow();

        match self.get().ty(s.as_context()) {
            ExternType::Func(_) => "func".into(),
            ExternType::Global(_) => "global".into(),
            ExternType::Table(_) => "table".into(),
            ExternType::Memory(_) => "memory".into(),
        }
    }
}

pub fn init() -> Result<(), Error> {
    let class = root().define_class("Export", Default::default())?;
    class.define_method("name", method!(Export::name, 0))?;
    class.define_method("type_name", method!(Export::type_name, 0))?;

    Ok(())
}
