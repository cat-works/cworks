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

local args = env.cmdline:match("^take%s+(.*)$")
if args == nil or args == "" then
  io.write("Usage: take <path>\n")
  os.exit()
end

local parent_path, filename = path.split_parent(path.join(env.cwd, args))

local result_file = path.resolve_object(session:fs_root(), parent_path)
if not result_file then
  io.write("No such file or directory: " .. parent_path .. "\n")
  os.exit()
end

result_file:create_string(filename, editor.take())

os.exit()
