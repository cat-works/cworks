import type { Process } from "./session";

export class FileSystem {
  constructor(private proc: Process) { }

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