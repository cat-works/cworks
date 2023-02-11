use python::PythonProcess;

pub struct PythonInterpreter {}
impl PythonInterpreter {
    pub fn new() -> PythonProcess {
        PythonProcess::new(
            r#"async def proc():
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
"#
            .to_string(),
        )
        .unwrap()
    }
}
