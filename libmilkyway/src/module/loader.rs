/* Module used for loading dynamic modules */
/* WARNING: Unsafe code ahead */
#[allow(unsafe_code)]
use libloading::{Library, Symbol};
use std::sync::Arc;
use crate::module::MilkywayModule;

pub struct DynamicModule {
    instance: *mut dyn MilkywayModule,
}

impl DynamicModule {
    pub unsafe fn load(path: &str) -> Result<DynamicModule, Box<dyn std::error::Error>> {
        let library = Arc::new(Library::new(path)?);

        unsafe {
            let create: Symbol<extern "C" fn() -> *mut dyn MilkywayModule> = library
                .get(b"create")
                .unwrap();

            let instance = create();

            Ok(DynamicModule {
                instance,
            })
        }
    }

    pub fn get_instance(&self) -> &mut dyn MilkywayModule {
        unsafe { &mut *self.instance }
    }
}
