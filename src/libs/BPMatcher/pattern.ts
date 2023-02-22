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