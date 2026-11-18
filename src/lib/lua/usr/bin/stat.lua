local path = require("path")

local args = env.cmdline:match("^stat%s+(.*)$")
if args == nil or args == "" then
  io.write("Usage: stat <path>\n")
  os.exit()
end

if args == "" then
  io.write("Usage: stat <path>\n")
  os.exit()
end

local path_ = path.join(env.cwd, args)
local st = path.resolve_object(session:fs_root(), path_)
if not st then
  io.write("No such file or directory: " .. path_ .. "\n")
  os.exit()
end
io.write("Kind: " .. st:type() .. "\n")
