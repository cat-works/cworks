export type TraitDefine = {
  check?: (x: string, arg: string) => boolean;
  filter?: (x: string, arg: string) => string;
};