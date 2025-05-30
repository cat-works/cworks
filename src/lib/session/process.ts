import { EventEmitter } from "../event_emitter";
import { Handle } from "./handle";
import type { PollResult, RawHandle, SyscallData, SyscallError } from "./raw_types";

export class Process {
  public emitter = new EventEmitter();
  private result_queue: PollResult[] = [];

  constructor(process: (p: Process) => Promise<bigint>) {

    this.emitter.mark_can_be_unused("callback");

    process(this).then((n) => {
      this.result_queue.push({ Done: n });
    }).catch((e) => {
      this.result_queue.push({ Done: -1n });
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
    this.result_queue.push({ Syscall: { IpcCreate: name } });
    return this.get_syscall_handle();
  }
  public ipc_connect(name: string): Promise<Handle> {
    this.result_queue.push({ Syscall: { IpcConnect: name } });
    return this.get_syscall_handle();
  }

  public send(handle: RawHandle, data: string): Promise<void> {
    this.result_queue.push({ Syscall: { Send: [handle, data] } });
    return new Promise((resolve, reject) => {
      this.emitter.once("callback", (s: SyscallData) => {
        if (Object.hasOwnProperty.call(s, "Fail")) {
          reject(s["Fail"]);
          return true;
        } else if (s === "None") {
          resolve();
          return true;
        } else {
          return false;
        }
      })
    })
  }

  public sleep(time: number): Promise<void> {
    this.result_queue.push({ Syscall: { Sleep: time } });
    return this.pending();
  }

  kernel_callback(data: SyscallData): PollResult {
    // if (data !== "None") {
    //   console.log("Kernel callback received:", data);
    // }
    let callback_handled = this.emitter.emit("callback", data);
    if (callback_handled === false) {
      if (Object.hasOwnProperty.call(data, "Handle")) {
        this.emitter.emit("handle", data["Handle"]);
      } else if (Object.hasOwnProperty.call(data, "Fail")) {
        this.emitter.emit("fail", data["Fail"]);
      } else if (Object.hasOwnProperty.call(data, "Connection")) {
        this.emitter.emit("connection", data["Connection"]);
      } else if (Object.hasOwnProperty.call(data, "ReceivingData")) {
        this.emitter.emit("receiving_data", data["ReceivingData"]);
      }
    }

    const result = this.result_queue.shift();
    // if (result) {
    //   console.log("Returning result:", result);
    // }
    return result || "Pending";

  }
}