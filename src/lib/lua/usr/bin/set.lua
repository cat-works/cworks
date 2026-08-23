local json = require("json")
local cworks = env.cworks
local cwd = env.cwd
local cmdline = env.cmdline -- mkdir ...

local function path_join(base, name)
  if base:sub(-1) ~= "/" then
    base = base .. "/"
  end
  local result = base .. name
  if result:sub(-1) == "/" then
    result = result:sub(1, -2)
  end

  return result
end

local args = cmdline:match("^set%s+(.*)$")
if args == nil or args == "" then
  io.write("Usage: set <path> <content>\n")
  cworks.exit()
end

local file_name, content = args:match("^(%S+)%s+(.+)$")
if file_name and content then
  local result_file = path_join(cwd, file_name)
  cworks.set(result_file, json.parse(content))
  io.write("Set content of " .. result_file .. "\n")
else
  io.write("Usage: set <file_name> <content>\n")
end
