import MainModuleFactory from "./cworks-rs.js";

export type KernelEvent =
  | { kind: "none" }
  | { kind: "success" }
  | { kind: "fail"; error: string }
  | { kind: "fsRoot"; obj: FSObj }
  | { kind: "fsGet"; obj: FSObj }
  | { kind: "getPid"; pid: number }
  | { kind: "invoke"; callerPid: number; obj: FSObj; arg: FSObj };

export interface FSObj {
  id(): number;
  getObj(name: string): FSObj;
  list(): string[];
  hasChild(name: string): boolean;
  addChild(name: string, obj: FSObj): FSObj;
  removeChild(name: string): void;
  type(): string;
  string(): string;
  int(): number;
  boolean(): boolean;
  float(): number;
  double(): number;
  bytes(): Uint8Array;
  createInt(name: string, value: number): FSObj;
  createString(name: string, value: string): FSObj;
  createBoolean(name: string, value: boolean): FSObj;
  createFloat(name: string, value: number): FSObj;
  createDouble(name: string, value: number): FSObj;
  createBytes(name: string, value: Uint8Array): FSObj;
  createNull(name: string): FSObj;
  createCompound(name: string): FSObj;
  createFunc(name: string): FSObj;
  ensureInt(name: string, value: number): FSObj;
  ensureString(name: string, value: string): FSObj;
  ensureBoolean(name: string, value: boolean): FSObj;
  ensureFloat(name: string, value: number): FSObj;
  ensureDouble(name: string, value: number): FSObj;
  ensureBytes(name: string, value: Uint8Array): FSObj;
  ensureNull(name: string): FSObj;
  ensureCompound(name: string): FSObj;
  ensureFunc(name: string): FSObj;
  subscribe(callback: (obj: FSObj) => void): () => void;
  publish(data: FSObj): Promise<undefined>;
}

export interface FSObjClass {
  newInt(value: number): FSObj;
  newString(value: string): FSObj;
  newBoolean(value: boolean): FSObj;
  newFloat(value: number): FSObj;
  newDouble(value: number): FSObj;
  newBytes(value: Uint8Array): FSObj;
  newNull(): FSObj;
  newCompound(): FSObj;
  newFunc(): FSObj;
}

export interface JsProcessHandle {
  done(): void;
  root(): void;
  waitForEvent(): void;
  sleep(seconds: number): void;
  getPid(): void;
}

export interface Session {
  step(): void;
  addProcess(callback: (data: KernelEvent) => void): JsProcessHandle;
}

export interface SessionClass {
  new(): Session;
}

export interface LuaEnv {
  run(code: string): void;
  createProcess(name: string, code: string): bigint;
}

export interface LuaEnvClass {
  new(): LuaEnv;
}

const mod: any = await MainModuleFactory();

export const Session: SessionClass = mod.Session;
export const FSObj: FSObjClass = mod.FSObj;
export const LuaEnv: LuaEnvClass = mod.LuaEnv;

export function demangle_str(x: string): string {
  return mod.demangleStr(x);
}
