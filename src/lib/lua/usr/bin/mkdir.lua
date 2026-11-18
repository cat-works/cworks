local path = require "path"
local args = env.cmdline:match("^mkdir%s+(.*)$")
if args == nil or args == "" then
  io.write("Usage: mkdir <name>\n")
  os.exit()
end

local current_path = path.resolve_object(session:fs_root(), env.cwd)
if args:sub(1, 1) == "/" then
  current_path = session:fs_root()
end
if not current_path then
  io.write("Current path does not exist: " .. env.cwd .. "\n")
  os.exit()
end

for segment in args:gmatch("[^/]+") do
  if current_path:has_child(segment) then
    if current_path:get_obj(segment):type() ~= "CompoundFSObj" then
      io.write("Cannot create directory '" .. segment .. "' because a non-directory object exists with the same name.\n")
      os.exit()
    end
    current_path = current_path:get_obj(segment)
  else
    current_path = current_path:create_compound(segment)
  end
end
