local path = require("path")

local args = env.cmdline:match("^cat%s+(.*)$")
if args == nil or args == "" then
  io.write("Usage: cat <path>\n")
  os.exit()
end

local result_file = path.resolve_object(session:fs_root(), path.join(env.cwd, args))
if not result_file then
  io.write("No such file or directory: " .. path.join(env.cwd, args) .. "\n")
  os.exit()
end

if result_file:type() ~= "String" then
  io.write("File is not a string: " .. result_file .. "\n")
  os.exit()
end

io.write(result_file:string() .. "\n")
