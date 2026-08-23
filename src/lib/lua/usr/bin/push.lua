local cworks = env.cworks
local editor = require("editor").new(
  cworks,
  "/run/debug-app/ta-load-back",
  "/run/debug-app/ta-take",
  "/run/debug-app/ta-push"
)
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

local args = cmdline:match("^push%s+(.*)$")
if args == nil or args == "" then
  io.write("Usage: push <path> <content>\n")
  cworks.exit()
end

local src_file = path_join(env.cwd, args)
local src = cworks.get(src_file)
if src["String"] == nil then
  io.write("File is not a string: " .. src_file .. "\n")
else
  editor.push(src["String"])
end
