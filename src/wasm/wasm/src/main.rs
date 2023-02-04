extern crate kernel;
extern crate python;

const PYTHON_CODE: &str = r#"async def proc():
    while True:
        code = input("> ")
        if not code:
            continue
        elif code == ".exit":
            break
        else:
            try:
                r = eval(code)
            except Exception as e1:
                try:
                    exec(code)
                except Exception as e2:
                    print("Unexpected error:")
                    print("Eval: ")
                    print(e1)
                    print("Exec: ")
                    print(e2)

            try:
                r = await r
            except Exception as e:
                pass

            print(repr(r))
"#;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut k = kernel::Kernel::default();
    let p = python::PythonProcess::new(PYTHON_CODE.to_string()).unwrap();
    k.register_process(Box::new(p));
    k.start();
    Ok(())
}
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
