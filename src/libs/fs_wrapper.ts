import type { Handle, Process } from "./session";
import { sleep } from "./utils";

export class FileSystem {
  public ipc: Handle;
  constructor(p: Process) {
    this.ipc = undefined;

    let fs = this;
    p.ipc_connect("system/file-system").then((ipc) => {
      fs.ipc = ipc;
    });
  }

  private handle_error(ret: string): void {
    if (
      [
        "InvalidCommandFormat",
        "UnsupportedMethod",
        "InvalidHandle",
        "UnknownPath",
        "UnknownError",
      ].includes(ret)
    ) {
      throw ret;
    }
  }

  public async wait_for_ready(waits: number = 100): Promise<void> {
    while (this.ipc === undefined) {
      await sleep(waits);
    }
  }

  public async cd(path: string): Promise<void> {
    await this.ipc.send(`Cd?${path}`);
    let ret = await this.ipc.recv();

    this.handle_error(ret);
  }

  public async list(): Promise<string[]> {
    await this.ipc.send("List");
    let ret = await this.ipc.recv();

    this.handle_error(ret);

    return ret.split("?");
  }
}