import { EventEmitter } from "./event_emitter";

class Handle extends EventEmitter {
  private emitter: EventEmitter;
  constructor(public handle: RawHandle, private process: Process) {
    super();
    this.emitter = new EventEmitter();
  }

  public send(data: string): Promise<void> {
    this.process.send(this.handle, data);
    return this.emitter.pending();
  }


}

type RawHandle = {
  pid: bigint;
  id: bigint;
};

export type Syscall =
  | { Sleep: number }
  | { IpcCreate: String }
  | { IpcConnect: String }
  | { Send: [RawHandle, String] };

export type SyscallError =
  | "NoSuchEntry"
  | "AlreadyExists"
  | "UnknownHandle"
  | "NotAllowedHandle"
  | "NotImplemented"
  | "UnreachableEntry";

export type SyscallData =
  | { Fail: SyscallError }
  | { Handle: RawHandle }
  | { Connection: { client: RawHandle; server: RawHandle } }
  | { ReceivingData: { focus: RawHandle; data: String } }
  | "None";

export type PollResult = "Pending" | { Syscall: Syscall } | { Done: bigint };

export class Process {
  private emitter = new EventEmitter();
  private result_queue: PollResult[] = [];
  private data_buffer: SyscallData[] = [];

  constructor(process: (ABCProcess) => Promise<bigint>) {
    process(this).then((n) => {
      this.result_queue.push({ Done: n });
    })
  }

  public get_syscall_handle(): Promise<Handle> {
    return new Promise((resolve, reject) => {

      this.emitter.once_or(["handle", "fail"], (event: string, x: RawHandle | SyscallError) => {
        if (event === "fail") {
          reject(x as SyscallError);
        } else {
          resolve(x as RawHandle);
        }
      });
    });
  }

  public pending(): Promise<void> {
    return new Promise((resolve) => {
      this.emitter.once("callback", resolve);
    });
  }

  public ipc_create(name: string): Promise<RawHandle> {
    this.result_queue.push({ Syscall: { IpcCreate: name } });

  }

  public sleep(time: number): Promise<void> {
    this.result_queue.push({ Syscall: { Sleep: time } });
    return this.pending();
  }

  kernel_callback(data: SyscallData): PollResult {
    this.emitter.emit("callback", data);

    if (data !== "None") {
      window.console.log("Got", data);
    }

    if (Object.hasOwnProperty.call(data, "Handle")) {
      this.emitter.emit("handle", data["Handle"]);
    } else if (Object.hasOwnProperty.call(data, "Fail")) {
      this.emitter.emit("fail", data["Fail"]);
    }

    if (this.result_queue) {
      return this.result_queue.shift() || "Pending";
    } else {
      return "Pending";
    }
  }
}