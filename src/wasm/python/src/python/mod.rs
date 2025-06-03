pub mod cworks;

use rustpython::InterpreterConfig;
use rustpython_vm::{
    builtins::PyBaseException, scope::Scope, Interpreter, PyRef, PyResult, Settings, VirtualMachine,
};

struct Python {
    interp: Interpreter,
}

impl Python {
    pub fn enter<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&VirtualMachine) -> R,
    {
        self.interp.enter(f)
    }
}

pub fn format_exception(e: PyRef<PyBaseException>, vm: &VirtualMachine) -> String {
    let mut s = String::new();

    s += &format!(
        "Error: {:#?}",
        e.args()
            .iter()
            .map(|x| x.str(vm).unwrap())
            .collect::<Vec<_>>()
    );
    if let Some(traceback) = e.traceback() {
        let frames = traceback.iter();
        for x in frames {
            s += &format!(
                "  File \"{}\", line {}, in {}",
                x.frame.code.source_path, x.lineno, x.frame.code.obj_name
            );
        }
    }

    s
}

pub fn panic_py_except(e: PyRef<PyBaseException>, vm: &VirtualMachine) -> ! {
    log::error!(
        "Error: {:#?}",
        e.args()
            .iter()
            .map(|x| x.str(vm).unwrap())
            .collect::<Vec<_>>()
    );
    if let Some(traceback) = e.traceback() {
        let frames = traceback.iter();
        for x in frames {
            log::error!(
                "  File \"{}\", line {}, in {}",
                x.frame.code.source_path,
                x.lineno,
                x.frame.code.obj_name
            );
        }
    }
    std::process::exit(1);
}

pub fn init_scope<'a>(vm: &VirtualMachine, scope: &'a Scope) -> PyResult<&'a Scope> {
    log::info!("Running script.");
    let r = vm.run_block_expr(
        scope.clone(),
        r#"
import cw
import io

async def step():
    import asyncio
    await asyncio.sleep(0)

async def pending():
    cw.pending();
    await step()

async def done(r):
    cw.done(r)
    await step()

print_org=print
def print(*a, **k):
    buf = io.StringIO()
    print_org(*a, **k, file=buf)
    cw.print(buf.getvalue())

def wrapper():
    coro = proc()
    while True:
        try:
            yield coro.send(None)
        except StopIteration:
            cw.done(0)
            yield None
"#,
    );
    log::info!("Ran script.");
    if let Err(e) = r {
        panic_py_except(e, vm);
    }

    Ok(scope)
}

static mut PYTHON: Option<Python> = None;

fn init_python() {
    log::info!("Initializing Python VM...");
    let mut setting = Settings::default();
    setting.debug = 1;

    let interp = InterpreterConfig::new()
        .settings(setting)
        .init_stdlib()
        .init_hook(Box::new(|vm| {
            vm.add_native_module("cw", Box::new(cworks::cworks_mod::make_module));
        }))
        .interpreter();

    log::info!("Initializing Python VM...DONE!");

    #[allow(static_mut_refs)]
    unsafe {
        PYTHON.replace(Python { interp })
    };
}

#[allow(static_mut_refs)]
pub fn python_enter<F, R>(f: F) -> R
where
    F: FnOnce(&VirtualMachine) -> R,
{
    let py = unsafe {
        if PYTHON.is_none() {
            init_python();
        }
        PYTHON.as_ref().unwrap()
    };
    py.enter(f)
}
