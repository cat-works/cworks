import type { Handle, Process } from "./session";

function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

export class FileSystem {
  constructor(private proc: Process) { }

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


  public async list(path: string): Promise<string[]> {
    return await this.proc.fs_list(path);
  }

  public async stat(path: string): Promise<{ kind: "Directory" | "File" }> {
    return await this.proc.fs_stat(path);
  }
  public async set(path: string, data: any): Promise<void> {
    return await this.proc.fs_set(path, data);
  }
  public async get(p: string): Promise<any> {
    return await this.proc.fs_get(p);
  }
  public async mkdir(path: string, name: string): Promise<void> {
    return await this.proc.fs_mkdir(path, name);
  }
}