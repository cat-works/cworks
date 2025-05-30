import type { Handle, Process } from "./session";

function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

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
  public async root(): Promise<void> {
    await this.ipc.send("Root");
    let ret = await this.ipc.recv();

    this.handle_error(ret);
  }
  public async get(p: string): Promise<string[]> {
    await this.ipc.send(`Get?${p}`);
    let ret = await this.ipc.recv();

    this.handle_error(ret);

    return ret.split("?");
  }
  public async set_raw(p: string, obj: string): Promise<void> {
    await this.ipc.send(`Set?${p}?${obj}`);
    let ret = await this.ipc.recv();

    this.handle_error(ret);
  }
  public async pwd(): Promise<string> {
    await this.ipc.send(`Pwd`);
    let ret = await this.ipc.recv();

    this.handle_error(ret);

    return ret;
  }
}