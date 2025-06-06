import lua_mod from "./lua-rs";

const mod = await lua_mod();

export class LuaEnv {
  ptr: number;
  constructor() {
    this.ptr = mod.___ffi_lufenv_new();
  }

  run(code: string) {
    mod.ccall("__ffi_luaenv_run", null, ["number", "string"], [this.ptr, code]);
  }

  thread(code: string): LuaThread {
    const ptr = mod.ccall("__ffi_luaenv_thread", "number", ["number", "string"], [this.ptr, code]);
    return new LuaThread(ptr);
  }
};

export class LuaThread {
  ptr: number;

  constructor(ptr: number) {
    this.ptr = ptr;
  }

  yield(arg: string): string {
    const encoded_arg = encode(arg);

    const encoded_value = mod.ccall("__ffi_lua_thread_yield", "string", ["number", "string"], [this.ptr, encoded_arg]);
    const value = decode(encoded_value);

    // console.log(`${JSON.stringify(arg)} -> ${JSON.stringify(encoded_arg)} -> ${JSON.stringify(encoded_value)} -> ${JSON.stringify(value)}`);
    return value;
  }
};

function encode(input: string): string {
  let result = "";
  for (const ch of input) {
    if (ch === "\0") {
      result += "\x01\x02";
    } else if (ch === "\x01") {
      result += "\x01\x01";
    } else {
      result += ch;
    }
  }
  return result;
}

function decode(input: string): string {
  let result = "";
  for (let i = 0; i < input.length; i++) {
    if (input[i] === "\x01") {
      if (input[i + 1] === "\x02") {
        result += "\0";
        i++;
      } else if (input[i + 1] === "\x01") {
        result += "\x01";
        i++;
      } else {
        result += "\x01";
      }
    } else {
      result += input[i];
    }
  }
  return result;
}