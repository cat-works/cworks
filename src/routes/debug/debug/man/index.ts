import commands from "./commands.md?raw";
import filesystem from "./filesystem.md?raw";
import cat from "./cat.md?raw";
import cd from "./cd.md?raw";
import clear from "./clear.md?raw";
import get from "./get.md?raw";
import ipc from "./ipc.md?raw";
import load from "./load.md?raw";
import ls from "./ls.md?raw";
import man from "./man.md?raw";
import mkdir from "./mkdir.md?raw";
import save from "./save.md?raw";
import set_ from "./set.md?raw";
import stat from "./stat.md?raw";

export const manuals: {
  [key: string]: string | undefined;
} = {
  commands: commands,
  filesystem: filesystem,
  cat: cat,
  cd: cd,
  clear: clear,
  get: get,
  ipc: ipc,
  load: load,
  ls: ls,
  man: man,
  mkdir: mkdir,
  save: save,
  set: set_,
  stat: stat,
}