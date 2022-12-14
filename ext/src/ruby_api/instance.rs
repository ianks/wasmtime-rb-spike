use super::{
    export::Export, module::Module, params::Params, root, store::Store, to_ruby_value::ToRubyValue,
};
use crate::{err, error, rtyped_data};
use magnus;
use magnus::{
    function, gc, method, DataTypeFunctions, Error, Module as _, Object, RArray, RHash, TypedData,
    Value,
};
use wasmtime::{AsContextMut, Func, Instance as InstanceImpl, StoreContextMut, Val};

#[derive(Clone, Debug, TypedData)]
#[magnus(class = "Wasmtime::Instance", mark)]
pub struct Instance {
    inner: InstanceImpl,
    store: Value,
}

unsafe impl Send for Instance {}

impl DataTypeFunctions for Instance {
    fn mark(&self) {
        gc::mark(&self.store);
    }
}

impl Instance {
    pub fn new(s: Value, module: &Module) -> Result<Self, Error> {
        let rtd = rtyped_data!(s)?;
        let store = rtd.get::<Store>()?;
        let module = module.get();
        let mut store = store.borrow_mut();
        let context = store.as_context_mut();
        let inner = InstanceImpl::new(context, &module, &[]).map_err(|e| error!("{}", e))?;

        Ok(Self { inner, store: s })
    }

    pub fn exports(&self) -> Result<RHash, Error> {
        let rtd = rtyped_data!(self.store)?;
        let store = rtd.get::<Store>()?;
        let mut borrowed_store = store.borrow_mut();
        let mut ctx = borrowed_store.as_context_mut();
        let hash = RHash::new();
        let exports = self
            .inner
            .exports(&mut ctx)
            .map(|export| Export::new(store, export));

        for export in exports {
            let name = export.name();
            hash.aset(name, export)?;
        }

        Ok(hash)
    }

    pub fn invoke(&self, name: String, args: RArray) -> Result<RArray, Error> {
        let rtd = rtyped_data!(self.store)?;
        let store = rtd.get::<Store>()?;
        let mut store = store.borrow_mut();
        let func = self.get_func(store.as_context_mut(), &name)?;
        let param_types = func.ty(store.as_context_mut()).params().collect::<Vec<_>>();
        let params_slice = unsafe { args.as_slice() };
        let params = Params::new(params_slice, param_types)?.to_vec()?;

        let results_len = func.ty(store.as_context_mut()).results().len();
        let mut results = vec![Val::null(); results_len];
        let ctx = store.as_context_mut();
        let results = self.invoke_func(ctx, &func, &params, results.as_mut_slice())?;

        Ok(RArray::from_vec(results))
    }

    fn get_func(&self, context: StoreContextMut<'_, Value>, name: &str) -> Result<Func, Error> {
        let instance = self.inner;

        if let Some(func) = instance.get_func(context, &name) {
            Ok(func)
        } else {
            err!("function \"{}\" not found", name)
        }
    }

    fn invoke_func(
        &self,
        context: StoreContextMut<'_, Value>,
        func: &Func,
        params: &[Val],
        results: &mut [Val],
    ) -> Result<Vec<Value>, Error> {
        func.call(context, params, results)
            .map_err(|e| error!("Could not invoke function: {}", e))?;

        let mut final_result = Vec::with_capacity(results.len());

        for result in results {
            final_result.push(result.to_ruby_value()?);
        }

        Ok(final_result)
    }
}

pub fn init() -> Result<(), Error> {
    let class = root().define_class("Instance", Default::default())?;

    class.define_singleton_method("new", function!(Instance::new, 2))?;
    class.define_method("invoke", method!(Instance::invoke, 2))?;
    class.define_method("exports", method!(Instance::exports, 0))?;

    Ok(())
}
