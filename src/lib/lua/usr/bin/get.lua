local cworks = require("cworks").new(session, env)
local cwd = env.cwd
local cmdline = env.cmdline -- mkdir ...

local path_join = require("path").join

local args = cmdline:match("^get%s+(.*)$")
if args == nil or args == "" then
  io.write("Usage: get <path>\n")
  cworks.exit()
end

local result_file = path_join(cwd, args)
local data = cworks.get(result_file)
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
