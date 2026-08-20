import { LuaEnv, LuaThread, LuaThreadError } from "../../lua/pkg/lua";
import loader from "$lib/lua/cworks-loader.lua?raw";
import json from "$lib/lua/json.lua?raw";

const env = new LuaEnv();
env.run(json);
env.run(loader);

export class LuaProcess {
  private thread: LuaThread;

  constructor(name: string, code: string, cmdline: string = "/") {
    const preamble = 'require("cworks").set_cmdline("' + cmdline + '")';

    this.thread = env.thread(name, preamble + code);
  }

  kernel_callback(data: any): any {
    const dataString = JSON.stringify(data, (key, value) => {
      if (typeof value === "bigint") {
        return value.toString();
      } else if (value instanceof Map) {
        return Object.fromEntries(value);
      }
      return value;
    });

    const result = (() => {
      try {
        return this.thread.yield(dataString)
      } catch (e) {
        if (e instanceof LuaThreadError) {
          return JSON.stringify("Done");
        } else {
          throw e;
        }
      }
    })();
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
