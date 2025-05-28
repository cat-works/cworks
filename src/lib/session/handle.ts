import { EventEmitter } from "../event_emitter";
import type { Process } from "./process";
import type { RawHandle } from "./raw_types";

function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

export class Handle extends EventEmitter {
  private receive_buffer: string[] = [];
  private receiving_mode: "recv" | "event" = "event";

  constructor(public handle: RawHandle, private process: Process) {
    super();
    this.receive_buffer = [];
    this.receiving_mode = "event";

    this.set_connection_handler();
    this.set_data_hander();
  }
  private set_data_hander() {
    let this_handle = this;
    this.process.emitter.on("receiving_data", (x: { focus: RawHandle; data: string }) => {
      if (x.focus.id !== this_handle.handle.id) {
        return false;
      }


      if (this.receiving_mode === "event") {
        this_handle.emit("message", x.data);
      } else if (this.receiving_mode === "recv") {
        this.receive_buffer.push(x.data);
      }
      return true;
    })
  }
  private set_connection_handler() {
    let this_handle = this;
    this.process.emitter.on(
      "connection",
      (c: { client: RawHandle; server: RawHandle }) => {
        if (c.server.id !== this_handle.handle.id) {
          return false;
        }
        let client = new Handle(c.client, this.process);
        this.emit("connection", client);
        return true;

      },
    );
  }

  public async recv(): Promise<string> {
    if (this.receive_buffer.length > 0) {
      return this.receive_buffer.shift();
    }

    this.receiving_mode = "recv";
    while (this.receive_buffer.length === 0) {
      this.process.pending();
      await sleep(10);
    }
    this.receiving_mode = "event";

    return this.recv();
  }

  public send(data: string): Promise<void> {
    return this.process.send(this.handle, data);
  }
}