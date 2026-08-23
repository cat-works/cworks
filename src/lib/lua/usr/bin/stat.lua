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

local args = cmdline:match("^stat%s+(.*)$")
if args == nil or args == "" then
  io.write("Usage: stat <path>\n")
  cworks.exit()
end

if args == "" then
  io.write("Usage: stat <path>\n")
else
  local st = cworks.stat(path_join(cwd, args))
  if st then
    io.write("Kind: " .. st["kind"] .. "\n")
  else
    io.write("No such file or directory: " .. path_join(cwd, args) .. "\n")
  end
end
