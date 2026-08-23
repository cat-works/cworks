import { EventEmitter } from "../event_emitter";
export class Process {
  public emitter = new EventEmitter("Process");
  private result_queue: any[] = [];

  constructor(process: (p: Process) => Promise<void>) {

    this.emitter.mark_can_be_unused("callback");

    process(this).then(() => {
      this.result_queue.unshift("Done");
    }).catch((e) => {
      this.result_queue.unshift("Done");
      throw e;
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

  public sleep(time: number): Promise<void> {
    this.result_queue.unshift({ Sleep: time });

    return this.pending();
  }

  public wait_for_process(pid: bigint): Promise<void> {
    return new Promise((resolve) => {
      this.emitter.once("callback", (x) => {
        if (x === "Done") {
          resolve();
          return true;
        }
      });
    });
  }

  public wait_for_event(): Promise<void> {
    this.result_queue.push("WaitForEvent");

    return new Promise((resolve) => {
      this.emitter.once("callback", (x) => {
        if (x !== "None") {
          const idx = this.result_queue.indexOf("WaitForEvent");
          if (idx !== -1) {
            this.result_queue.splice(idx, 1);
          }
        }
        resolve();
        return x === "None";
      });
    });
  }

  public fs_list(path: string): Promise<string[]> {
    this.result_queue.unshift({ "List": path });

    return new Promise((resolve, reject) => {
      this.emitter.once("callback", (s: any) => {
        if (s.Fail !== undefined) {
          reject(s.Fail);
          return true;
        } else if (s.FSList !== undefined) {
          resolve(s.FSList);
          return true;
        }
      })
    })
  }

  public fs_stat(path: string): Promise<{ kind: "Directory" | "File" }> {
    this.result_queue.unshift({ "Stat": path });

    return new Promise((resolve, reject) => {
      this.emitter.once("callback", (s: any) => {
        if (s.Fail !== undefined) {
          reject(s.Fail);
          return true;
        } else if (s.FSStat !== undefined) {
          resolve(s.FSStat);
          return true;
        }
      })
    })
  }

  public fs_set(path: string, data: any): Promise<void> {
    this.result_queue.unshift({ "Set": [path, data] });

    return new Promise((resolve, reject) => {
      this.emitter.once("callback", (s: any) => {
        if (s.Fail !== undefined) {
          reject(s.Fail);
          return true;
        } else if (s === "FSSuccess") {
          resolve();
          return true;
        }
      })
    })
  }

  public fs_get(path: string): Promise<any> {
    this.result_queue.unshift({ "Get": path });

    return new Promise((resolve, reject) => {
      this.emitter.once("callback", (s: any) => {
        if (s.Fail !== undefined) {
          reject(s.Fail);
          return true;
        }
        if (s.FSGet !== undefined) {
          resolve(s.FSGet);
          return true;
        }
      })
    })
  }

  public fs_mkdir(path: string, name: any): Promise<void> {
    this.result_queue.unshift({ "Mkdir": [path, name] });

    return new Promise((resolve, reject) => {
      this.emitter.once("callback", (s: any) => {
        if (s.Fail !== undefined) {
          reject(s.Fail);
          return true;
        } else if (s === "FSSuccess") {
          resolve();
          return true;
        }
      })
    })
  }

  public fs_subscribe(path: string, callback: (caller_pid: bigint, data: any) => void): Promise<void> {
    this.result_queue.unshift({ "Subscribe": path });

    this.emitter.on("invoke", (data: { caller_pid: bigint, path: string, arg: any }) => {
      if (data.path === path) {
        callback(data.caller_pid, data.arg);
        return true;
      }
    });

    return new Promise((resolve, reject) => {
      this.emitter.once("callback", (s: any) => {
        if (s.Fail !== undefined) {
          reject(s.Fail);
          return true;
        } else if (s === "FSSuccess") {
          resolve();
          return true;
        } else {
          return false;
        }
      })
    })
  }

  public fs_unsubscribe(path: string): Promise<void> {
    this.result_queue.unshift({ "Unsubscribe": path });

    return new Promise((resolve, reject) => {
      this.emitter.once("callback", (s: any) => {
        if (s.Fail !== undefined) {
          reject(s.Fail);
          return true;
        } else if (s === "FSSuccess") {
          resolve();
          return true;
        } else {
          return false;
        }
      })
    })
  }

  public fs_publish(path: string, data: any): Promise<void> {
    this.result_queue.unshift({ "Publish": [path, data] });

    return new Promise((resolve, reject) => {
      this.emitter.once("callback", (s: any) => {
        if (s.Fail !== undefined) {
          reject(s.Fail);
          return true;
        } else if (s === "FSSuccess") {
          resolve();
          return true;
        } else {
          return false;
        }
      })
    })
  }


  kernel_callback(data: any): any {
    if (data !== "None") {
      let callback_handled = this.emitter.emit("callback", data);
      if (callback_handled === false) {
        if (data.Fail !== undefined) {
          this.emitter.emit("fail", data.Fail);
        } else if (data.Invoke !== undefined) {
          let caller_pid = data.Invoke.caller_pid;
          let path: string = data.Invoke.path;
          let arg = data.Invoke.arg;
          this.emitter.emit("invoke", {
            caller_pid: caller_pid,
            path: path,
            arg: arg
          });

        } else {
          console.error("Unhandled kernel callback:", data);
          return "Done";
        }
      }
    }

    const result = this.result_queue.shift();
    // if (result) { console.debug("Kernel callback result", result) };

    return result || "Pending";

  }
}