local cworks = require("cworks").new(session, env)
local cwd = env.cwd
local cmdline = env.cmdline -- mkdir ...

local path_join = require("path").join

local args = cmdline:match("^stat%s+(.*)$")
if args == nil or args == "" then
  io.write("Usage: stat <path>\n")
  cworks.exit()
end

if args == "" then
  io.write("Usage: stat <path>\n")
else
  local st = cworks.stat(path_join(cwd, args))
  if st then
    io.write("Kind: " .. st["kind"] .. "\n")
  else
    io.write("No such file or directory: " .. path_join(cwd, args) .. "\n")
  end
end
