import type {
  FSObj,
  JsProcessHandle,
  KernelEvent,
  Session,
} from "../../../wasm/cworks-wasm/pkg/cworks";

export class Process {
  private handle: JsProcessHandle;
  private pending: {
    resolve: (data: KernelEvent) => void;
    reject: (error: Error) => void;
    acceptNone: boolean;
  } | null = null;
  private queue: Array<() => void> = [];
  private handlers = new Map<number, (arg: FSObj) => void>();
  private done = false;

  constructor(session: Session, run: (p: Process) => Promise<void>) {
    this.handle = session.addProcess(this.kernelCallback.bind(this));
    run(this).then(
      () => this.finish(),
      (error) => {
        this.finish();
        throw error;
      },
    );
  }

  private finish() {
    this.done = true;
  }

  private request(
    send: () => void,
    acceptNone = false,
  ): Promise<KernelEvent> {
    return new Promise<KernelEvent>((resolve, reject) => {
      this.queue.push(() => {
        this.pending = { resolve, reject, acceptNone };
        send();
      });
    });
  }

  private pump() {
    if (this.pending || this.queue.length === 0) {
      return;
    }
    const next = this.queue.shift();
    next?.();
  }

  public kernelCallback(data: KernelEvent): void {
    if (data.kind === "invoke") {
      const handler = this.handlers.get(data.obj.id());
      handler?.(data.arg);
      return;
    }
    if (this.pending) {
      if (data.kind === "none") {
        if (this.pending.acceptNone) {
          const { resolve } = this.pending;
          this.pending = null;
          resolve(data);
        }
      } else {
        const { resolve, reject } = this.pending;
        this.pending = null;
        if (data.kind === "fail") {
          reject(new Error(data.error));
        } else {
          resolve(data);
        }
      }
    }
    if (this.done) {
      this.handle.done();
      return;
    }
    this.pump();
  }

  public async fsRoot(): Promise<FSObj> {
    const data = await this.request(() => this.handle.root());
    if (data.kind !== "fsRoot") {
      throw new Error(`unexpected response for root: ${data.kind}`);
    }
    return data.obj;
  }

  public async waitForEvent(): Promise<void> {
    await this.request(() => this.handle.waitForEvent(), true);
  }

  public async sleep(seconds: number): Promise<void> {
    await this.request(() => this.handle.sleep(seconds), true);
  }

  public async getPid(): Promise<number> {
    const data = await this.request(() => this.handle.getPid());
    return data.kind === "getPid" ? data.pid : 0;
  }
}
