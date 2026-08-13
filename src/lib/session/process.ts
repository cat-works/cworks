import { EventEmitter } from "../event_emitter";
import { Handle } from "./handle";
import type { PollResult, RawHandle, SyscallData, SyscallError } from "./raw_types";

export class Process {
  public emitter = new EventEmitter();
  private result_queue: any[] = [];

  constructor(process: (p: Process) => Promise<bigint>) {

    this.emitter.mark_can_be_unused("callback");

    process(this).then((n) => {
      this.result_queue.push({ Done: n });
    }).catch((e) => {
      this.result_queue.push({ Done: -1 });
      throw e;
    });
  }

  public get_syscall_handle(): Promise<Handle> {
    return new Promise((resolve, reject) => {
      this.emitter.once_or(["handle", "fail"], (event: string, x: RawHandle | SyscallError) => {
        if (event === "fail") {
          reject(x as SyscallError);
        } else {
          resolve(new Handle(x as RawHandle, this));
        }
        return true;
      });
    });
  }

  public pending(): Promise<void> {
    return new Promise((resolve) => {
      this.emitter.once("callback", (x) => {
        resolve();
        return x === "None";

      });
    });
  }

  public ipc_create(name: string): Promise<Handle> {
    this.result_queue.push({
      "Syscall": {
        "IpcCreate": name
      }
    });
    return this.get_syscall_handle();
  }
  public ipc_connect(name: string): Promise<Handle> {
    this.result_queue.push({
      "Syscall": {
        "IpcConnect": name
      }
    });
    return this.get_syscall_handle();
  }

  public send(handle: RawHandle, data: string): Promise<void> {
    this.result_queue.push({
      "Syscall": {
        "Send": [handle, data]
      }
    });
    return new Promise((resolve, reject) => {
      this.emitter.once("callback", (s: Uint8Array) => {
        const op = s[0];
        if (0x01 <= op && op <= 0x06) {
          reject(op);
          return true;
        } else if (op == 0x00) {
          resolve();
          return true;
        }

        return false;
      })
    })
  }

  public sleep(time: number): Promise<void> {
    this.result_queue.push({
      "Syscall": {
        Sleep: time
      }
    });

    return this.pending();
  }
  public fs_list(path: string): Promise<string[]> {
    this.result_queue.push({
      "Syscall": {
        "List": path
      }
    });

    return new Promise((resolve, reject) => {
      this.emitter.once("callback", (s: any) => {
        if (s.Fail !== undefined) {
          reject(s.Fail);
          return true;
        } else if (s.FSList !== undefined) {
          resolve(s.FSList);
          return true;
        }
      })
    })
  }
  public fs_stat(path: string): Promise<any> {
    this.result_queue.push({
      "Syscall": {
        "Stat": path
      }
    });

    return new Promise((resolve, reject) => {
      this.emitter.once("callback", (s: any) => {
        if (s.Fail !== undefined) {
          reject(s.Fail);
          return true;
        } else if (s.FSStat !== undefined) {
          resolve(s.FSStat);
          return true;
        }
      })
    })
  }
  public fs_set(path: string, data: any): Promise<void> {
    this.result_queue.push({
      "Syscall": {
        "Set": [path, data]
      }
    });

    return new Promise((resolve, reject) => {
      this.emitter.once("callback", (s: any) => {
        if (s.Fail !== undefined) {
          reject(s.Fail);
          return true;
        } else if (s === "FSSuccess") {
          resolve();
          return true;
        }
      })
    })
  }
  public fs_mkdir(path: string, name: any): Promise<void> {
    this.result_queue.push({
      "Syscall": {
        "Mkdir": [path, name]
      }
    });

    return new Promise((resolve, reject) => {
      this.emitter.once("callback", (s: any) => {
        if (s.Fail !== undefined) {
          reject(s.Fail);
          return true;
        } else if (s === "FSSuccess") {
          resolve();
          return true;
        }
      })
    })
  }

  kernel_callback(data: any): any {
    if (data !== "None") {
      console.debug("Kernel callback data", data);
      let callback_handled = this.emitter.emit("callback", data);
      if (callback_handled === false) {
        if (data.Fail !== undefined) {
          this.emitter.emit("fail", data.Fail);
        } else if (data.Handle !== undefined) {
          this.emitter.emit("handle",
            data.Handle
          );
        } else if (data.Connection !== undefined) {
          let client = data.Connection.client;
          let server = data.Connection.server;
          this.emitter.emit("connection", {
            client: client,
            server: server,
          });
        } else {
          console.error("Unhandled kernel callback:", data);
          return { "Done": -1 };

          const op = data[0];
          if (0x01 <= op && op <= 0x06) {
            this.emitter.emit("fail", op);
          } else if (op == 0x09) { // recv
            let handle_id = data.slice(1, 17).reduce((acc, byte, index) => {
              return acc | (BigInt(byte) << BigInt(120 - index * 8));
            }, 0n);
            let message = new TextDecoder().decode(data.slice(17));
            // console.groupCollapsed("Receive event");
            // console.log("Handle ID:", handle_id);
            // console.log("Message:", message);
            // console.log("data:", data);
            // console.log("  handle array:", data.slice(1, 17));
            // console.log("  message array:", data.slice(17));
            // console.groupEnd();
            this.emitter.emit("receiving_data", {
              focus: {
                id: handle_id
              } as RawHandle,
              data: message
            });
          } else {
            console.error("Unhandled kernel callback:", data);
          }
        }
      }
    }

    const result = this.result_queue.shift();
    if (result) { console.debug("Kernel callback result", result) };

    return result || "Pending";

  }
}