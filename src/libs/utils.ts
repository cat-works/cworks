export function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

export function my_stringify(x: object) {
  return JSON.stringify(x, (_, v) =>
    typeof v === "bigint" ? v.toString() : v
  );
}