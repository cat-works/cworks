import { EventEmitter } from "./event_emitter";

export class Handle extends EventEmitter {
  constructor(public handle: RawHandle, private process: Process) {
    super();

    let this_handle = this;
    this.process.emitter.on("receiving_data", (x: { focus: RawHandle; data: string }) => {
      if (x.focus.id !== this_handle.handle.id) {
        return false;
      }

      this_handle.emit("message", x.data);
      return true;
    })
    this.process.emitter.on(
      "connection",
      (c: { client: RawHandle; server: RawHandle }) => {
        if (c.server.id !== this_handle.handle.id) {
          return false;
        }
        let client = new Handle(c.client, this.process);
        this.emit("connection", client);
        this.process.emitter.on("receiving_data", (x: { focus: RawHandle; data: string }) => {
          if (x.focus.id !== client.handle.id) {
            return false;
          }

          client.emit("message", x.data);
          return true;
        })
        return true;

      },
    );
  }

  public send(data: string): Promise<void> {
    return this.process.send(this.handle, data);
  }
}

type RawHandle = {
  pid: bigint;
  id: bigint;
};

export type Syscall =
  | { Sleep: number }
  | { IpcCreate: string }
  | { IpcConnect: string }
  | { Send: [RawHandle, string] };

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
  | { ReceivingData: { focus: RawHandle; data: string } }
  | "None";

export type PollResult = "Pending" | { Syscall: Syscall } | { Done: bigint };

export class Process {
  public emitter = new EventEmitter();
  private result_queue: PollResult[] = [];

  constructor(process: (p: Process) => Promise<bigint>) {

    this.emitter.mark_can_be_unused("callback");

    process(this).then((n) => {
      this.result_queue.push({ Done: n });
    }).catch((e) => {
      this.result_queue.push({ Done: -1n });
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

    return this.result_queue.shift() || "Pending";

  }
}