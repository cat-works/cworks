import { EventEmitter } from "../event_emitter";
import { Handle } from "./handle";
import type { PollResult, RawHandle, SyscallData, SyscallError } from "./raw_types";

export class Process {
  public emitter = new EventEmitter();
  private result_queue: Uint8Array[] = [];

  constructor(process: (p: Process) => Promise<bigint>) {

    this.emitter.mark_can_be_unused("callback");

    process(this).then((n) => {
      this.result_queue.push(new Uint8Array([
        0x01, // PollResult.Done
        Number((n >> 56n) & 0xFFn),
        Number((n >> 48n) & 0xFFn),
        Number((n >> 40n) & 0xFFn),
        Number((n >> 32n) & 0xFFn),
        Number((n >> 24n) & 0xFFn),
        Number((n >> 16n) & 0xFFn),
        Number((n >> 8n) & 0xFFn),
        Number((n >> 0n) & 0xFFn),
      ]));
    }).catch((e) => {
      this.result_queue.push(new Uint8Array([
        0x01, // PollResult.Fail
        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff
      ]));
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
    this.result_queue.push(new Uint8Array([
      0x03, // IPC Create
      ...new TextEncoder().encode(name)
    ]));
    return this.get_syscall_handle();
  }
  public ipc_connect(name: string): Promise<Handle> {
    this.result_queue.push(new Uint8Array([
      0x04, // IPC Connect
      ...new TextEncoder().encode(name)
    ]));
    return this.get_syscall_handle();
  }

  public send(handle: RawHandle, data: string): Promise<void> {
    this.result_queue.push(new Uint8Array([
      0x05, // IPC Create
      Number(handle.id >> 120n & 0xFFn),
      Number(handle.id >> 112n & 0xFFn),
      Number(handle.id >> 104n & 0xFFn),
      Number(handle.id >> 96n & 0xFFn),
      Number(handle.id >> 88n & 0xFFn),
      Number(handle.id >> 80n & 0xFFn),
      Number(handle.id >> 72n & 0xFFn),
      Number(handle.id >> 64n & 0xFFn),
      Number(handle.id >> 56n & 0xFFn),
      Number(handle.id >> 48n & 0xFFn),
      Number(handle.id >> 40n & 0xFFn),
      Number(handle.id >> 32n & 0xFFn),
      Number(handle.id >> 24n & 0xFFn),
      Number(handle.id >> 16n & 0xFFn),
      Number(handle.id >> 8n & 0xFFn),
      Number(handle.id >> 0n & 0xFFn),
      ...new TextEncoder().encode(data)
    ]));
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
    // encode 'time' as f32
    // pack into a Uint8Array as follows:
    // - 1. 0x02 // Sleep
    // - 2. 4 bytes of the float value
    const buffer = new ArrayBuffer(5);
    const view = new DataView(buffer);
    view.setUint8(0, 0x02); // Sleep
    view.setFloat32(1, time, false); // Little-endian float
    this.result_queue.push(new Uint8Array(buffer));

    return this.pending();
  }

  kernel_callback(data: Uint8Array): Uint8Array {
    // if (data !== "None") {
    //   console.log("Kernel callback received:", data);
    // }
    let callback_handled = this.emitter.emit("callback", data);
    if (callback_handled === false) {
      const op = data[0];

      if (op == 0x00) {

      } else if (0x01 <= op && op <= 0x06) {
        this.emitter.emit("fail", op);
      } else if (op == 0x07) { // handle
        let handle_id = data.slice(1, 17).reduce((acc, byte, index) => {
          return acc | (BigInt(byte) << BigInt(120 - index * 8));
        }, 0n);
        // console.groupCollapsed("Handle event");
        // console.log("Handle ID:", handle_id);
        // console.log("data:", data);
        // console.log("  handle array:", data.slice(1, 17));
        // console.groupEnd();
        this.emitter.emit("handle", {
          id: handle_id
        } as RawHandle);

      } else if (op == 0x08) { // connection
        let client = data.slice(1, 17).reduce((acc, byte, index) => {
          return acc | (BigInt(byte) << BigInt(120 - index * 8));
        }, 0n);
        let server = data.slice(17, 33).reduce((acc, byte, index) => {
          return acc | (BigInt(byte) << BigInt(120 - index * 8));
        }, 0n);
        // console.groupCollapsed("Connection event");
        // console.log("Client ID:", client);
        // console.log("Server ID:", server);
        // console.log("data:", data);
        // console.log("  client array:", data.slice(1, 17));
        // console.log("  server array:", data.slice(18, 33));
        // console.groupEnd();
        this.emitter.emit("connection", {
          client: {
            id: client
          } as RawHandle,
          server: {
            id: server
          } as RawHandle,
        });
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

    const result = this.result_queue.shift();
    // if (result) {
    //   console.log("Returning result:", result);
    // }
    return result || new Uint8Array([0]);

  }
}