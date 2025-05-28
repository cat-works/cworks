import init, { demangle_str } from "../wasm/pkg/wasm";

export function ghs_demangle(str: string): string {
  return demangle_str(str);
}

await init();