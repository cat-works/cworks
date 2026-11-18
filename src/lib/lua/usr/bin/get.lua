local path = require("path")

local args = env.cmdline:match("^get%s+(.*)$")
if args == nil or args == "" then
  io.write("Usage: get <path>\n")
  os.exit()
end

local result_file = path.join(env.cwd, args)
local data = path.resolve_object(session:fs_root(), result_file)
if not data then
  io.write("No such file or directory: " .. result_file .. "\n")
  os.exit()
end

local kind = data:type()

local value
if kind == "String" then
  value = data:string()
elseif kind == "Int" then
  value = tostring(data:int())
elseif kind == "Boolean" then
  value = tostring(data:boolean())
elseif kind == "Float" then
  value = tostring(data:float())
elseif kind == "Double" then
  value = tostring(data:double())
elseif kind == "Null" then
  value = "null"
elseif kind == "Func" then
  value = "<channel>"
elseif kind == "CompoundFSObj" then
  value = "<directory>"
else
  value = "<" .. kind .. ">"
end

io.write("Content of " .. result_file .. " (" .. kind .. "):\n" .. value .. "\n")
