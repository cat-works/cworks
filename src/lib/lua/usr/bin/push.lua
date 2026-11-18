local path = require("path")

local editor_loopback = path.resolve_object(session:fs_root(), "/run/debug-app/ta-load-back")
local editor_take = path.resolve_object(session:fs_root(), "/run/debug-app/ta-take")
local editor_push = path.resolve_object(session:fs_root(), "/run/debug-app/ta-push")
if not editor_loopback or not editor_take or not editor_push then
  io.write("Editor channels do not exist. Please run the editor first.\n")
  os.exit()
end
local editor = require("editor").new(
  session,
  editor_loopback,
  editor_take,
  editor_push
)

local args = env.cmdline:match("^push%s+(.*)$")
if args == nil or args == "" then
  io.write("Usage: push <path> <content>\n")
  os.exit()
end

local src_file = path.join(env.cwd, args)
local src = session:fs_root():resolve_path(src_file)
if src:type() ~= "String" then
  io.write("File is not a string: " .. src_file .. "\n")
else
  editor.push(src:string())
end
