export type TraitDefine = {
  check?: (x: string, arg: string) => boolean;
  filter?: (x: string, arg: string) => string;
};

export class Pattern {
  traits: { [key: string]: [string, string][] };
  binary: string[];
  mnemonic: string;

  constructor(s: string) {
    // s: [r: r, R: r + R] 01rrrRRR dddddddd: ${r} <- ${R}
    let match = s.match(/\[(.*)\] (.*): (.*)/);
    if (match === null) throw new Error("invalid pattern");


    this.traits = {};
    match[1].replace(/\s/g, "").split(",").forEach((x) => {
      let [key, value] = x.split(":");
      this.traits[key] = value.split("+").map(x => {
        let m = x.match(/(.*)\((.*)\)/);
        return m ? [m[1], m[2]] : [x, ""]
      });
    });

    this.binary = match[2].split(" ");
    this.mnemonic = match[3];
  }
};

export class BinaryPatternMatcherConfig {
  traits: { [key: string]: TraitDefine };
  dynamic_datas: {
    [key: string]: () => string
  };
  patterns: Pattern[];
}

class Match {
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

function apply_trait(trait: TraitDefine, data: string, trait_arg: string): string | null {
  if (trait.check && !trait.check(data, trait_arg)) {
    return null;
  }
  if (trait.filter) {
    data = trait.filter(data, trait_arg);
  }

  return data;
}

export class BinaryPatternMatcher {
  private buffer: string[];
  constructor(private config: BinaryPatternMatcherConfig) {

  }
  set_buffer(buffer: string[]) {
    this.buffer = buffer;
  }

  public *undefined_patterns(): Generator<number> {
    let buffer_ = this.buffer;

    for (let i = 0; i < 0x100; i++) {
      this.set_buffer([i.toString(2).padStart(8, "0")]);
      if (this.match() === null) {
        yield i;
      }
    }

    this.buffer = buffer_;
  }

  private get_buffer_slice(length: number): string[] | null {
    let r = this.buffer.slice(0, length);
    if (r.length !== length) return null;
    return r;
  }

  private reduce_buffer_head(length: number) {
    this.buffer = this.buffer.slice(length);
  }

  private match_without_trait(pattern: Pattern, part: string[]): Match | null {
    let match: Match = new Match();
    for (let i = 0; i < pattern.binary.length; i++) {
      const sub_binary = pattern.binary[i];
      const sub_part = part[i];
      for (let j = 0; j < 8; j++) {
        if (sub_binary[j] === "0" || sub_binary[j] === "1") {
          if (sub_binary[j] !== sub_part[j]) return null;
          continue;
        }

        match.append_variable(sub_binary[j], sub_part[j]);
      }
    }

    return match;
  }

  public has_data(): boolean {
    return this.buffer.length > 0;
  }

  private match_pattern(pattern: Pattern): Match | null {
    const pattern_length = pattern.binary.length;

    const part = this.get_buffer_slice(pattern_length);
    if (part === null) return null;

    let match = this.match_without_trait(pattern, part);
    if (match === null) return null;

    for (const var_name in match.variables) {
      if (pattern.traits[var_name] === undefined) continue;

      for (const [trait_name, trait_arg] of pattern.traits[var_name]) {
        let trait = this.config.traits[trait_name];
        if (trait === undefined) {
          throw new Error(`trait ${trait_name} is not defined`);
        }

        let data = apply_trait(trait, match.variables[var_name], trait_arg);
        if (data === null) return null;

        match.variables[var_name] = data;
      }
    }

    return match;
  }

  pop(): string {
    let buf = this.buffer[0];
    if (buf === undefined) return "";
    this.reduce_buffer_head(1);
    return buf;
  }

  match(): [number, string] | null {
    for (let pattern of this.config.patterns) {
      let match = this.match_pattern(pattern);
      if (match === null) continue;
      const bin_length = pattern.binary.length
      this.reduce_buffer_head(pattern.binary.length);
      return [bin_length, match.format(pattern.mnemonic)];
    }
    return null;
  }

}