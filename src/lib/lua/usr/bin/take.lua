local cworks = require("cworks").new(session, env)
local editor = require("editor").new(
  cworks,
  "/run/debug-app/ta-load-back",
  "/run/debug-app/ta-take",
  "/run/debug-app/ta-push"
)
local cmdline = env.cmdline -- mkdir ...

local path_join = require("path").join

local args = cmdline:match("^take%s+(.*)$")
if args == nil or args == "" then
  io.write("Usage: take <path> <content>\n")
  cworks.exit()
end

local result_file = path_join(env.cwd, args)
local buffer = editor.take()
cworks.set(result_file, { String = buffer })

cworks.exit()
