import { LuaEnv, LuaThread } from "../../lua/pkg/lua";
import lib from "./cworks-loader.lua?raw";
import json_lib from "./json.lua?raw";

const env = new LuaEnv();
env.run(json_lib);
env.run(lib);

export class LuaProcess {
  private thread: LuaThread;

  constructor(code: string) {
    this.thread = env.thread(code);
  }

  kernel_callback(data: any): any {
    const dataString = JSON.stringify(data);
    const result = this.thread.yield(dataString);
    const parsed_obj = JSON.parse(result);

    return parsed_obj;
  }
}
