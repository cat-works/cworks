use kernel::Handle;

extern crate kernel;
extern crate python;

mod processes;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut k = kernel::Kernel::default();
    let p = processes::IPCMaster::default();
    k.register_process(Box::new(p));

    let p = processes::IPCSlave::default();
    k.register_process(Box::new(p));
    k.start();
    Ok(())
}
/*
#[cfg(test)]
mod test {
    use python::PythonProcess;

    #[test]
    fn python_based_process() {
        let mut k = kernel::Kernel::default();
        let p = PythonProcess::new(
            r#"async def proc():
        print("print")
        print("/")

        print("step")
        await step()

        print("pending")
        await pending()"#
                .to_string(),
        )
        .unwrap();
        k.register_process(Box::new(p));
        k.start();
    }
}

*/
