mod generator_wrapper;
mod python;
mod python_interpreter;

extern crate kernel;
use generator_wrapper::GeneratorWrapper;
use kernel::{PollResult, Process, SyscallData};
use python::panic_py_except;
pub use python_interpreter::python_interpreter;
use rustpython_vm::{convert::IntoPyException, PyResult};

pub use python::format_exception;
pub use python::python_enter;

pub struct PythonProcess {
    generator: GeneratorWrapper,
}

impl PythonProcess {
    pub fn new(source: String) -> PyResult<PythonProcess> {
        Ok(PythonProcess {
            generator: GeneratorWrapper::new(source + "\nwrapper()")?,
        })
    }
}

impl Process for PythonProcess {
    fn poll(&mut self, data: &SyscallData) -> PollResult<i64> {
        {
            python::cworks::get_state_mut().syscall_result = data.clone();
        }

        if let Err(e) = self.generator.step() {
            python_enter(|vm| {
                panic_py_except(e.into_pyexception(vm), vm);
            });
        }
        python::cworks::get_state_mut().poll_result.take().unwrap()
    }
}

#[cfg(test)]
mod test {
    use kernel::Kernel;

    use super::*;

    #[test]
    fn main() {
        let mut k = Kernel::default();

        let p1 = PythonProcess::new(
            r#"async def proc():
    print("print")
    print("/")

    print("step")
    await step()

    print("pending")
    await pending()

wrapper()"#
                .to_string(),
        );
        if let Err(p1) = &p1 {
            python::python_enter(|vm| {
                python::panic_py_except(p1.clone(), vm);
            })
        }
        let p1 = p1.unwrap();

        k.register_process(Box::new(p1));

        k.start();
    }
}
