import type { Handle, Process } from "./session";

function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

export class FileSystem {
  public ipc: Handle | null;
  constructor(p: Process) {
    this.ipc = null;

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
    while (this.ipc === null) {
      await sleep(waits);
    }
  }


  public async list(path: string): Promise<string[]> {
    await this.ipc.send(`List?${path}`);
    let ret = await this.ipc.recv();

    this.handle_error(ret);

    return ret.split("?");
  }

  public async stat(path: string): Promise<string> {
    await this.ipc.send(`Stat?${path}`);
    let ret = await this.ipc.recv();
    this.handle_error(ret);

    return ret;
  }
  public async get(p: string): Promise<[string, string]> {
    await this.ipc.send(`Get?${p}`);
    let ret = await this.ipc.recv();

    this.handle_error(ret);

    let [kind, ...parts] = ret.split("?");
    return [kind, parts.join("?")];
  }
  public async set_raw(p: string, obj: string): Promise<void> {
    await this.ipc.send(`Set?${p}?${obj}`);
    let ret = await this.ipc.recv();

    this.handle_error(ret);
  }
  public async mkdir(path: string, name: string): Promise<void> {
    await this.ipc.send(`Mkdir?${path}?${name}`);
    let ret = await this.ipc.recv();

    this.handle_error(ret);
  }
}