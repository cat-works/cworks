import { EventEmitter } from "./event_emitter";

export class Handle extends EventEmitter {
  constructor(public handle: RawHandle, private process: Process) {
    super();

    let this_handle = this;
    this.process.emitter.on(
      "connection",
      (c: { client: RawHandle; server: RawHandle }) => {
        if (c.server.id === this_handle.handle.id) {
          this.emit("connection", new Handle(c.client, this.process));
          return true;
        }
        return false;
      },
    );
  }

  public send(data: string): Promise<void> {
    this.process.send(this, data);
    return this.process.pending();
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
    })
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
      this.emitter.once("callback", () => {
        resolve();
        return true;
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

  public send(handle: Handle, data: string): void {
    this.result_queue.push({ Syscall: { Send: [handle.handle, data] } });
  }

  public sleep(time: number): Promise<void> {
    this.result_queue.push({ Syscall: { Sleep: time } });
    return this.pending();
  }

  kernel_callback(data: SyscallData): PollResult {
    this.emitter.emit("callback", data);

    if (Object.hasOwnProperty.call(data, "Handle")) {
      this.emitter.emit("handle", data["Handle"]);
    } else if (Object.hasOwnProperty.call(data, "Fail")) {
      this.emitter.emit("fail", data["Fail"]);
    } else if (Object.hasOwnProperty.call(data, "Connection")) {
      this.emitter.emit("connection", data["Connection"]);
    } else if (Object.hasOwnProperty.call(data, "ReceivingData")) {
      this.emitter.emit("receiving_data", data["ReceivingData"]);
    }

    return this.result_queue.shift() || "Pending";

  }
}