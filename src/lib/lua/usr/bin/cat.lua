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

local args = cmdline:match("^cat%s+(.*)$")
if args == nil or args == "" then
  io.write("Usage: cat <path>\n")
  cworks.exit()
end

local result_file = path_join(cwd, args)
local data = cworks.get(result_file)
if data["String"] == nil then
  io.write("File is not a string: " .. result_file .. "\n")
else
  io.write(data["String"] .. "\n")
end
