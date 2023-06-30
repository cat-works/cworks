<script lang="ts">
  import { Matcher, Pattern } from "./libs/BPMatcher";
  import { patch_console_log } from "./libs/utils";
  let skip = 0;
  let src =
    "21A0C31801C17EEA9BDE72CDE0DD18FBCD6738F0B3CB77282FCB4B280235C93EC3BC20183EB3BD2008CD9CDE21F4C418D43809CD9CDE0155010918C9CD9CDE3EEC853801256F18BDCB7F2831CB4B280234C93EC4BC381120183EF4BD2008CD9CDE21B3C3189F3009CD9CDE01ABFE091894CD9CDE3E14853001246F1888CB672823CB4B28053E108677C93EC5BC200E3E07BD2009CD9CDE21A0C3C3D5DDCD9CDE23C3D5DDCB6F282ECB4B28053EF08677C93EC3BC200E3EA0BD2009CD9CDE2107C5C3D5DDCD9CDE2BC3D5DD0056CB432004FA9BDE77C9CB5F2802C1C9CB57C8FAE3D25FFAE2D2FE0121EFDE2816BB200436AC180236AE21F4DE20043654180236521814BB280436DC180236DE21F4DE28043624180236222107C51E121614010001097E0100FE09321520F3E521EFDE3EFC867721F4DE3E048677E11D20DE23C3D5DD";
  let dest = "";

  let addr = 0;

  function pattern_trait(def: string) {
    const map = {};
    for (const token of def.split(",")) {
      const match = token.match(/\s*([01]+)\s*:\s*(.+)\s*/);
      if (!match) {
        console.error("Invalid pattern trait definition: " + token);
        throw new Error("Invalid pattern trait definition: " + token);
      }

      map[match[1]] = match[2];
    }
    console.log(map);
    return (y: string) => map[y] ?? null;
  }

  let patterns = new Matcher({
    traits: {
      Imm: (x: string, o: string) => {
        const offset = o === "" ? 0 : parseInt(o, 16);
        const h = x
          .split(/(.{8})/)
          .reverse()
          .join("");
        return "0x" + (offset + parseInt(h, 2)).toString(16).toUpperCase();
      },
      S8: (x: string) => {
        let n = parseInt(x, 2);
        if (n > 127) {
          n = n - 256;
        }
        return (
          (n < 0 ? "-" : "") +
          "0x" +
          Math.abs(n).toString(16).padStart(2, "0").toUpperCase()
        );
      },
      RA: (x: string, offset: string) => {
        let n = parseInt(x, 2);
        if (n > 127) {
          n = n - 256;
        }
        return (
          "0x" +
          (addr + n + parseInt(offset === "" ? "0" : offset))
            .toString(16)
            .padStart(2, "0")
            .toUpperCase()
        );
      },
      R: pattern_trait("000:B, 001:C, 010:D, 011:E, 100:H, 101:L, 111:A"),
      PR: pattern_trait("00:BC, 01:DE, 10:AL, 11:AF"),
      RegHL: pattern_trait(
        "000:B, 001:C, 010:D, 011:E, 100:H, 101:L, 110:(HL), 111:A"
      ),
      MathO: pattern_trait(
        "000:add, 001:adc, 010:sub, 011:sbc, 100:and, 101:xor, 110:or, 111:cp"
      ),
      RegO: pattern_trait("01:toggle, 10:set, 11:unset"),
      MathO2: pattern_trait("100:inc, 101:dec"),
      Cond1: pattern_trait(
        "000:NZ, 001:Z, 010:NC, 011:C, 100:PO, 101:PE, 110:P, 111:M"
      ),
      Cond2: pattern_trait("100:NZ, 101:Z, 110:NC, 111:C"),
    },
    dynamic_datas: {
      addr: () => addr.toString(16),
    },
    patterns: [
      // 8bit register incoming transfer
      new Pattern("[r:R,R:RegHL] 01rrrRRR: ${r} <- ${R}"),
      new Pattern("[r:R]         01110rrr: *(HL) <- ${r}"),
      new Pattern("[r:R,d:Imm]   00rrr110 d{8}: ${r} <- ${d}"),
      new Pattern("[x:Imm(ff00)] 11110000 x{8}: A <- ${x}"),
      // 16bit reg in trans
      new Pattern("[r:PR, x:Imm] 00rr0001 x{16}: ${r} <- ${x}"),
      new Pattern("[x:Imm]       00101010 x{16}: HL <- *(${x})"),
      new Pattern("[x:Imm]       11101010 x{16}: *(${x}) <- a"),
      // stack
      new Pattern("[r:PR] 11rr0101: push ${r}"),
      new Pattern("[r:PR] 11rr0001: pop ${r}"),
      // operations
      new Pattern("[r:RegHL, t:MathO2] 00rrrttt: ${t} ${r}"),
      new Pattern("[s:PR]              00ss1001: HL += ${s}"),
      new Pattern("[s:PR,t:MathO2]     00ss0011: inc ${s}"),
      new Pattern("[s:PR,t:MathO2]     00ss1011: dec ${s}"),
      new Pattern("[r:RegHL, t:MathO]  10tttrrr: ${t} A, ${r}"),
      new Pattern("[a:RegO,b:Imm,r:R]  11001011 aabbbrrr: ${a} ${r}@${b}"),
      // flow
      new Pattern("[p:RA(2)]          00011000 p{8}: jump ${p} (relative)"),
      new Pattern("[c:Cond2, x:RA(2)] 00ccc000 x{8}: jump[${c}] ${x}"),
      new Pattern("[x:Imm]            11001101 x{16}: call ${x}"),
      new Pattern("[p:Imm]            11000011 p{16}: jp ${p}"),
      new Pattern("[c:Cond1]          11ccc000: ret[${c}]"),
      new Pattern("[:]                11001001: ret"),
      // extended
      new Pattern("[r:R,b:Imm] 11001011 00bbbrrr: swap ${r} (b=${b})"),
      new Pattern("[:]         00110010: ldd (hl), a"),
      new Pattern("[p:Imm]     11111010 p{16}: A <- ${p}"),
      new Pattern("[x:Imm]     11111110 x{8}: cp A, ${x}"),
      // original
      new Pattern("[:] 00000000: nop"),
      new Pattern("[x:Imm] 00110110 x{8}: *(HL) <- ${x}"),
    ],
  });

  $: {
    let buf = src
      .replace(/[\s\n,]/g, "")
      .trim()
      .split(/(..)/)
      .filter((x) => x)
      .map((x) => parseInt(x, 16))
      .map((x) => x.toString(2).padStart(8, "0"));

    patterns.set_buffer(buf);
    dest = "";
    addr = 0xddd0 + skip;
    for (let _ = 0; _ < skip; _++) {
      patterns.pop();
    }
    while (patterns.has_data()) {
      const match = patterns.match();
      if (match === null) {
        dest += `Unknown[${patterns.pop()}@${addr.toString(16)}]\n`;
        addr += 1;
      } else {
        dest +=
          addr.toString(16).toUpperCase().padStart(4, "0") +
          ": " +
          match[1]
            .map((x) => parseInt(x, 2).toString(16).padStart(2, "0"))
            .join("")
            .toUpperCase()
            .padEnd(2 * 3) +
          " | " +
          match[2] +
          "\n";
        addr += match[0];
      }
    }
  }

  let buf = patch_console_log();
</script>

<textarea bind:value={src} />
<textarea bind:value={dest} />

<pre><div class="a" style="font-family: 'Fira Code'">{$buf}</div></pre>

<style>
  textarea {
    width: 100%;
    height: 25%;
    font-family: "Fira Code";
    font-size: 16px;
  }
  .a {
    font-family: "Fira Code Light";
    font-size: 14px;
  }
</style>
