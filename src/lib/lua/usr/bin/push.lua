local cworks = require("cworks").new(session, env)
local editor = require("editor").new(
  cworks,
  "/run/debug-app/ta-load-back",
  "/run/debug-app/ta-take",
  "/run/debug-app/ta-push"
)
local cmdline = env.cmdline -- mkdir ...

local path_join = require("path").join

local args = cmdline:match("^push%s+(.*)$")
if args == nil or args == "" then
  io.write("Usage: push <path> <content>\n")
  cworks.exit()
end

local src_file = path_join(env.cwd, args)
local src = cworks.get(src_file)
if src:type() ~= "String" then
  io.write("File is not a string: " .. src_file .. "\n")
else
  editor.push(src:string())
end
