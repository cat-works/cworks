local cworks = require("cworks").new(session, env)
local json = require("json")

local path_join = require("path").join

--- child lua process spawner

local lua_process_spawner = {}
lua_process_spawner.last_pid = nil

cworks.subscribe("/run/debug-app/lua-sp", function(_, data)
  local pid
  if data:type() == "Int" then
    pid = data:int()
  elseif data:type() == "String" then
    pid = tonumber(data:string())
  end
  if pid == nil then
    print("Invalid pid received: " .. data:type())
    return
  end

  lua_process_spawner.last_pid = pid
end)

function lua_process_spawner.spawn_lua_process(lua_path, cwd, cmd_line)
  local request = cworks.root.new_compound()
  request:create_string("name", lua_path:match("[^/]+$"))
  request:create_string("code", cworks.get(lua_path):string())
  request:create_string("cwd", cwd)
  request:create_string("cmd_line", cmd_line)
  request:add_child("stdout", env.stdout)
  request:add_child("stdin", env.stdin)
  request:add_child("pid_reply_to", cworks.resolve("/run/debug-app/lua-sp"))

  cworks.publish("/run/sys/exec-lua", request)

  while lua_process_spawner.last_pid == nil do
    cworks.wait_for_event()
  end

  local pid = lua_process_spawner.last_pid
  lua_process_spawner.last_pid = nil

  return pid
end

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
    else
      local new_path = path_join(pwd, args)
      local st = cworks.stat(new_path)
      if st and st["kind"] == "Directory" then
        pwd = new_path
      else
        print("No such directory: " .. new_path .. "")
      end
    end
  elseif command:sub(0, 2) == "./" then
    local lua_path = path_join(pwd, command:sub(3) .. ".lua")
    if cworks.stat(lua_path) then
      local pid = lua_process_spawner.spawn_lua_process(lua_path, pwd, line)
      cworks.wait_for_process(pid)
    else
      print("No such file: " .. lua_path .. "")
    end
  else
    local lua_path = "/usr/bin/" .. command .. ".lua"
    if cworks.stat(lua_path) then
      local pid = lua_process_spawner.spawn_lua_process(lua_path, pwd, line)
      cworks.wait_for_process(pid)
    else
      print("Unknown command: " .. command .. "")
    end
  end

  ::continue::
end
