export class Pattern {
  traits: { [key: string]: [string, string][] };
  binary: string[];
  mnemonic: string;

  constructor(s: string) {
    // s: [r: r, R: r + R] 01rrrRRR dddddddd: ${r} <- ${R}
    let match = s.match(/\[(.*)\]\s+(.*)\s*:\s+(.*)/);
    if (match === null) throw new Error("invalid pattern");


    this.traits = {};
    match[1].replace(/\s/g, "").split(",").forEach((x) => {
      let [key, value] = x.split(":");
      this.traits[key] = value.split("+").map(x => {
        let m = x.match(/(.*)\((.*)\)/);
        return m ? [m[1], m[2]] : [x, ""]
      });
    });

    this.binary = match[2].split(" ").map(x => {
      const m1 = x.match(/\[[0-9a-fA-F]{2}\]/);
      const m2 = x.match(/(.)\{(\d+)\}/);

      if (m1) {
        return parseInt(m1[1], 16).toString(2).padStart(8, "0");
      } else if (m2) {
        return m2[1].repeat(parseInt(m2[2], 10));
      } else {
        return x;
      }
    });
    this.mnemonic = match[3];
  }
};