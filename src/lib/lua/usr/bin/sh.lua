local path = require("path")

local root = session:fs_root()
local lua_process_spawner = require("lua_process").LuaProcessSpawner.new(session, root, env)



--- end of child lua process spawner

print("\x1b[1;32mCat OS Shell\x1b[m")

local pwd = "/"
while true do
  print("\n\x1b[1;32m" .. pwd .. "\x1b[m")
  io.write("\x1b[2m$\x1b[m ")
  local line = io.read("*l")
  local command, args = line:match("^(%S+)%s*(.*)$")

  if command == nil then
    goto continue
  end

  if command == "cd" then
    if args == "" then
      pwd = "/"
      goto continue
    end
    local new_path = path.join(pwd, args)
    local obj = path.resolve_object(root, new_path)
    if obj == nil then
      print("No such directory: " .. new_path .. "obj is nil")
      goto continue
    end
    if obj:type() ~= "CompoundFSObj" then
      print("No such directory: " .. new_path .. "obj is not a directory")
      goto continue
    end

    pwd = new_path
  else
    local lua_path
    if command:sub(0, 2) == "./" then
      lua_path = path.join(pwd, command:sub(3) .. ".lua")
    else
      lua_path = "/usr/bin/" .. command .. ".lua"
    end

    local lua_obj = path.resolve_object(root, lua_path)
    if lua_obj == nil then
      print("No such file: " .. lua_path .. "")
      goto continue
    end
    if lua_obj:type() ~= "String" then
      print("No such file: " .. lua_path .. "obj is not a string")
      goto continue
    end
    local lua_code = lua_obj:string()
    local name = lua_path:match("([^/]+)%.lua$")
    local pid = lua_process_spawner:spawn(name, lua_code, pwd, line)
    session:wait_for_process(pid)
  end

  ::continue::
end
