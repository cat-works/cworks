use crate::python::{init_scope, python_enter};
use rustpython_vm::{
    protocol::{PyIter, PyIterReturn},
    PyObjectRef, PyResult,
};

pub struct GeneratorWrapper {
    generator: PyObjectRef,
}

impl GeneratorWrapper {
    pub fn new(source: String) -> PyResult<GeneratorWrapper> {
        python_enter(|vm| {
            let scope = vm.new_scope_with_builtins();
            let scope = init_scope(vm, &scope)?;

            Ok(GeneratorWrapper {
                generator: vm.run_block_expr(scope.clone(), source.as_str())?,
            })
        })
    }

    pub fn step(&self) -> PyResult<bool> {
        python_enter(|vm| {
            let r = PyIter::new(self.generator.clone()).next(vm)?;
            let ret = match r {
                PyIterReturn::Return(_value) => true,
                PyIterReturn::StopIteration(_) => false,
            };
            Ok(ret)
        })
    }
}
