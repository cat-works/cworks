local cworks = require("cworks").new(session, env)
local cwd = env.cwd
local cmdline = env.cmdline -- mkdir ...

local path_join = require("path").join

local args = cmdline:match("^cat%s+(.*)$")
if args == nil or args == "" then
  io.write("Usage: cat <path>\n")
  cworks.exit()
end

local result_file = path_join(cwd, args)
local data = cworks.get(result_file)
if data:type() ~= "String" then
  io.write("File is not a string: " .. result_file .. "\n")
else
  io.write(data:string() .. "\n")
end
