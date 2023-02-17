
export class Parser {
  constructor(private buffer: number[] = []) {
  }

  private get_register_name(no: number): string {
    switch (no) {
      case 0: return "B";
      case 1: return "C";
      case 2: return "D";
      case 3: return "E";
      case 4: return "H";
      case 5: return "L";
      case 6: return "!";
      case 7: return "A";
    }
  }

  private get_mathematics_operator_mnemonic_name(no: number): string {
    switch (no) {
      case 0: return "add";
      case 1: return "adc";
      case 2: return "sub";
      case 3: return "sbc";
      case 4: return "and";
      case 5: return "xor";
      case 6: return "or";
      case 7: return "cp";
    }
  }

  private get_pair_register_name(a: number): string {
    switch (a) {
      case 0: return "BC";
      case 1: return "DE";
      case 2: return "HL";
      case 3: return "SP";
    }
  }

  private get_buffer0(): number {
    let elem = this.buffer[0];
    this.buffer = this.buffer.slice(1);

    return elem;
  }

  private parse_instruction(): string {
    let opcode = this.get_buffer0();
    if (opcode === undefined) {
      return "<Error> buffer[0] == undefined";
    }

    let mode = (opcode & 0xc0) >> 6;
    let reg1 = (opcode & 0x38) >> 3;
    let reg2 = (opcode & 0x07);

    if (mode === 0x01 && reg2 !== 0x06) {
      let src = this.get_register_name(reg2);
      let dest = this.get_register_name(reg1);
      return `${dest} = ${src}`;
    } else if (mode === 0x01 && reg2 === 0x06) {
      let dest = this.get_register_name(reg1);
      return `${dest} = *(HL)`;
    } else if (mode === 0 && reg2 === 0x06) {
      let dest = this.get_register_name(reg1);
      let value = this.get_buffer0();
      return `${dest} = 0x${value.toString(16)}`;
    } else if (opcode === 0x00) {
      return "nop";
    } else if (mode === 0x00 && (opcode & 0x0f) === 1) {
      let pair_register_name = this.get_pair_register_name((opcode & 0x30) >> 8);
      let low = this.get_buffer0();
      let high = this.get_buffer0();

      return `${pair_register_name} <- ${(high * 0x100 + low).toString(16)}`
    } else if (opcode === 0xcb) {
      let mode = this.get_buffer0();
      let a = (mode & 0xc0) >> 6;
      let b = (mode & 0x38) >> 3;
      let c = (mode & 0x7);

      if (a === 0) {
        let reg = this.get_register_name(c);
        return `BIT ${b} ${reg}`
      } else {
        return `Unimplemented BIT Instruction: CB ${mode.toString(16)}`
      }
    } else if (mode === 0x03 && (opcode & 0x0f) === 0x05) {
      let q2 = this.get_pair_register_name((opcode & 0x3) >> 4);
      return `push ${q2}`;
    } else if (mode === 0x03 && (opcode & 0x0f) === 0x01) {
      let q2 = this.get_pair_register_name((opcode & 0x3) >> 4);
      return `pop ${q2}`;
    } else if (mode === 0x02 && reg2 !== 0x06) {
      if (reg2 === 0x07) {
        return `${this.get_mathematics_operator_mnemonic_name(reg1)} A`
      } else {
        return `${this.get_mathematics_operator_mnemonic_name(reg1)} A, ${this.get_register_name(reg2)}`
      }
    } else if (opcode === 0xc9) {
      return "ret";
    } else {
      return `Unimplemented instruction: ${opcode.toString(16)}`;
    }
  }

  parse(_src: string) {
    this.buffer = _src
      .replace(/[ \n]+/g, "")
      .split(/(..)/g)
      .filter(Boolean)
      .map((x) => parseInt(x, 16));

    let instructions = [];
    while (this.buffer.length !== 0) {
      instructions.push(this.parse_instruction());
    }

    return instructions.join("\n");
  }
}