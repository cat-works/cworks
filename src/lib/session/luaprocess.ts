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
    const dataString = JSON.stringify(data, (key, value) => {
      if (typeof value === "bigint") {
        return value.toString();
      }
      return value;
    });
    const result = this.thread.yield(dataString);
    const parsed_obj = JSON.parse(result);

    // transform string back to bigint for handles
    const transformHandles = (obj: any): any => {
      if (Array.isArray(obj)) {
        return obj.map(transformHandles);
      } else if (obj && typeof obj === "object") {
        const transformedObj: any = {};
        for (const key in obj) {
          transformedObj[key] = transformHandles(obj[key]);
        }
        return transformedObj;
      } else if (typeof obj === "string" && obj.startsWith("$$bi:")) {
        return BigInt(obj.slice(5));
      }
      return obj;
    };

    return transformHandles(parsed_obj);
  }
}
