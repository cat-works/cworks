import commands from "./commands.md?raw";
import filesystem from "./filesystem.md?raw";
export const manuals: {
  [key: string]: string | undefined;
} = {
  commands: commands,
  filesystem: filesystem,
}