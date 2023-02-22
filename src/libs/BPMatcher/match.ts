export class Match {
  variables: { [key: string]: string };

  constructor() {
    this.variables = {};
  }

  append_variable(key: string, data: string): void {
    if (!Object.hasOwn(this.variables, key)) {
      this.variables[key] = data;
    } else {
      this.variables[key] += data;
    }
  }

  format(mnemonic: string): string {
    let r = mnemonic;
    for (const key in this.variables) {
      r = r.replace(new RegExp(`\\$\\{${key}\\}`, "g"), this.variables[key]);
    }

    return r;
  }
}