//! wasmi-backed loader: runs a plugin wasm module as a cworks `Process`.

use cworks_sdk::Session;
use kernel::{PollResult, Process, SyscallData};

pub struct WasmPluginProcess {
    instance: wasmi::Instance,
    store: wasmi::Store<()>,
    session: Session,
}

impl WasmPluginProcess {
    pub fn load(wasm_bytes: &[u8]) -> Result<Self, wasmi::Error> {
        let engine = wasmi::Engine::default();
        let module = wasmi::Module::new(&engine, wasm_bytes)?;
        let mut store = wasmi::Store::new(&engine, ());
        let linker = wasmi::Linker::<()>::new(&engine);
        let instance = linker.instantiate_and_start(&mut store, &module)?;

        Ok(Self {
            instance,
            store,
            session: Session::default(),
        })
    }

    fn poll_export(&mut self, input: &[u8]) -> Result<Vec<u8>, wasmi::Error> {
        let alloc = self
            .instance
            .get_typed_func::<u32, u32>(&self.store, "cworks_alloc")?;
        let poll = self
            .instance
            .get_typed_func::<(u32, u32), u32>(&self.store, "cworks_plugin_poll")?;
        let memory = self
            .instance
            .get_memory(&self.store, "memory")
            .ok_or(wasmi::Error::new("memory export not found"))?;

        let in_ptr = alloc.call(&mut self.store, input.len() as u32)?;
        memory.write(&mut self.store, in_ptr as usize, input)?;

        let out_ptr = poll.call(&mut self.store, (in_ptr, input.len() as u32))?;

        let mut len_buf = [0u8; 4];
        memory.read(&self.store, out_ptr as usize, &mut len_buf)?;
        let out_len = u32::from_le_bytes(len_buf) as usize;

        let mut out = vec![0u8; out_len];
        memory.read(&self.store, out_ptr as usize + 4, &mut out)?;
        Ok(out)
    }
}

impl Process for WasmPluginProcess {
    fn poll(&mut self, data: &SyscallData) -> PollResult {
        let input = cworks_sdk::encode_request(data);

        let output = match self.poll_export(&input) {
            Ok(out) => out,
            Err(e) => {
                log::error!("plugin poll failed: {e}");
                return PollResult::Done;
            }
        };

        let res = cworks_sdk::decode_response(&output);
        if matches!(res, PollResult::Pending) && output.len() > 4 {
            log::error!("plugin returned undecodable response");
        }
        res
    }
}
