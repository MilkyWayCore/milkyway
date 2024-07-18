/* Module used for loading dynamic modules */
/* WARNING: Unsafe code ahead */
#[allow(unsafe_code)]
use libloading::{Library, Symbol};
use std::sync::Arc;
use crate::module::MilkywayModule;

pub struct DynamicModule {
    pub instance: Box<dyn MilkywayModule>,
    _library: Library,
}

impl DynamicModule {
    pub unsafe fn load(path: &str) -> Result<DynamicModule, Box<dyn std::error::Error>> {
        let library =Library::new(path).unwrap();
        type Constructor = unsafe fn() -> *mut dyn MilkywayModule;
        let instance: Box<dyn MilkywayModule>;
        unsafe {
            let create: Symbol<Constructor> = library
                .get(b"create")
                .unwrap();

            instance = Box::from_raw(create());
        }
        Ok(DynamicModule {
            instance,
            _library: library,
        })
    }
}
