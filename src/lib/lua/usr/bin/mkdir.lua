local cworks = require("cworks").new(session, env)
local cwd = env.cwd
local cmdline = env.cmdline -- mkdir ...

local args = cmdline:match("^mkdir%s+(.*)$")
if args == nil or args == "" then
  io.write("Usage: mkdir <name>\n")
  cworks.exit()
end

local current_path = cwd
if args:sub(1, 1) == "/" then
  current_path = ""
end
for segment in args:gmatch("[^/]+") do
  local new_path = current_path .. "/" .. segment
  local st = cworks.stat(new_path)
  if st == nil then
    cworks.mkdir(current_path, segment)
    io.write("Created directory: " .. new_path .. "\n")
  elseif st["kind"] ~= "Directory" then
    io.write("Cannot create directory: " .. new_path .. " (not a directory)\n")
    cworks.exit()
  end
  current_path = new_path
end
