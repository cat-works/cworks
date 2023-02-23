import type { Pattern } from "./pattern";
import type { TraitDefine } from "./trait_define";

export class Config {
  traits: { [key: string]: TraitDefine };
  dynamic_datas: {
    [key: string]: () => string
  };
  patterns: Pattern[];
}