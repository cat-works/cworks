import { LuaEnv, LuaThread } from "../../lua/pkg/lua";
import lib from "./cworks-loader.lua?raw";

const env = new LuaEnv();
env.run(lib);

export class LuaProcess {
  private thread: LuaThread;

  constructor(code: string) {
    this.thread = env.thread(code);
  }

  kernel_callback(data: Uint8Array): Uint8Array {
    // Convert Uint8Array to string
    let dataString = "";
    for (let i = 0; i < data.length; i++) {
      dataString += String.fromCharCode(data[i]);
    }

    const result = this.thread.yield(dataString);

    // Convert the result string back to a Uint8Array
    const resultArray = new Uint8Array(result.length);
    for (let i = 0; i < result.length; i++) {
      resultArray[i] = result.charCodeAt(i);
    }
    return resultArray;
  }
}
