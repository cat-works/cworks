use std::sync::Mutex;

use kernel::{PollResult, SyscallData};
use once_cell::sync::Lazy;
use rustpython_vm::pymodule;

#[derive(Default)]
pub struct State {
    pub syscall_result: SyscallData,
    pub poll_result: PollResult<i64>,
}

pub static STATE: Lazy<Mutex<State>> = Lazy::new(|| Mutex::new(State::default()));

#[pymodule]
pub mod cworks_mod {
    use kernel::{PollResult, SyscallData};
    use rustpython_vm::{builtins::PyIntRef, PyObjectRef, PyResult, VirtualMachine};

    use super::STATE;

    #[pyfunction]
    fn get_syscall_result(vm: &VirtualMachine) -> PyResult<PyObjectRef> {
        let r = vm.ctx.new_dict();

        let res = &STATE.lock().unwrap().syscall_result;

        match res {
            SyscallData::None => {
                r.set_item("kind", vm.ctx.none(), vm)?;
            }
            SyscallData::Handle(Err(e)) => {
                r.set_item("kind", vm.ctx.new_str("Handle").into(), vm)?;
                r.set_item("status", vm.ctx.new_str("Failed").into(), vm)?;
                r.set_item("error", vm.ctx.new_str(e.to_string()).into(), vm)?;
            }
            SyscallData::Handle(Ok(h)) => {
                r.set_item("kind", vm.ctx.new_str("Handle").into(), vm)?;
                r.set_item("status", vm.ctx.new_str("Successed").into(), vm)?;
                r.set_item("handle", vm.ctx.new_int(h.id).into(), vm)?;
            }
            SyscallData::Connection { client, server } => {
                r.set_item("kind", vm.ctx.new_str("Connection").into(), vm)?;
                r.set_item("client", vm.ctx.new_int(client.id).into(), vm)?;
                r.set_item("server", vm.ctx.new_int(server.id).into(), vm)?;
            }
            SyscallData::ReceivingData { focus, data } => {
                r.set_item("kind", vm.ctx.new_str("ReceivingData").into(), vm)?;
                r.set_item("handle", vm.ctx.new_int(client.id).into(), vm)?;
                r.set_item("data", vm.ctx.new_str(data.clone()).into(), vm)?;
            }
        }

        Ok(r.into())
    }

    #[pyfunction]
    fn pending(vm: &VirtualMachine) -> PyResult<PyObjectRef> {
        STATE.lock().unwrap().poll_result = PollResult::Pending;
        Ok(vm.ctx.none())
    }

    #[pyfunction]
    fn done(res: PyObjectRef, vm: &VirtualMachine) -> PyResult<PyObjectRef> {
        let r = res.try_into_value::<PyIntRef>(vm)?;

        STATE.lock().unwrap().poll_result =
            PollResult::Done(r.try_to_primitive::<i128>(vm)?.try_into().unwrap());
        Ok(vm.ctx.none())
    }

    #[pyfunction]
    fn print(c: PyObjectRef, vm: &VirtualMachine) -> PyResult<PyObjectRef> {
        let a = c.str(vm)?;
        let s = a.to_string();
        print!("P: {}", s);
        Ok(vm.ctx.none())
    }
}
