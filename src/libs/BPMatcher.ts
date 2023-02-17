export type TraitDefine = (string) => boolean;

export type Pattern = {
  traits: { [key: string]: string },
  binary: string[],
  mnemonic: string
};

export class BinaryPatternMatcherConfig {
  traits: { [key: string]: TraitDefine };
  patterns: Pattern[];
}

export class BinaryPatternMatcher {
  buffer: string[];
  constructor(private config: BinaryPatternMatcherConfig) {

  }
  set_buffer(buffer: string[]) {
    this.buffer = buffer;
  }

  private get_buffer_length(): number {
    return this.buffer.length;
  }

  private get_buffer_slice(length: number): string[] | null {
    let r = this.buffer.slice(0, length);
    if (r.length !== length) return null;
    return r;
  }

  private reduce_buffer_head(length: number) {
    this.buffer = this.buffer.slice(length);
  }

  match(): string {
    for (let pattern of this.config.patterns) {
      let part = this.get_buffer_slice(pattern.binary.length);

      if (part === null) continue;

      for (let i = 0; i < pattern.binary.length; i++) {
        console.log(`TM: ${pattern.binary[i]} ${part[i]}`);
      }
    }
    return "^^";
  }

}