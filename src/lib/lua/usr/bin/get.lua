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

local args = cmdline:match("^get%s+(.*)$")
if args == nil or args == "" then
  io.write("Usage: get <path>\n")
  cworks.exit()
end

local result_file = path_join(cwd, args)
local data = cworks.get(result_file)
io.write("Content of " .. result_file .. ":\n" .. json.stringify(data) .. "\n")
