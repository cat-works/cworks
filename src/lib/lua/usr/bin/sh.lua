local cworks = env.cworks
local json = require("json")

local function path_join(base, name)
  if base == "" then
    return name
  end
  if name == "" then
    return base
  end

  if base:sub(-1) ~= "/" then
    base = base .. "/"
  end
  local result = base .. name
  if result:sub(-1) == "/" then
    result = result:sub(1, -2)
  end

  return result
end

print("\x1b[1;32mCat OS Shell\x1b[m")

--- child lua process spawner

local lua_process_spawner = {}
lua_process_spawner.last_pid = nil

cworks.subscribe("/run/debug-app/lua-sp", function(caller, data)
  if data["String"] == nil then
    print("Invalid data for lua process spawner: " .. json.stringify(data))
    return
  end
  local pid_str = data["String"]
  local pid = tonumber(pid_str)
  if pid == nil then
    print("Invalid pid received: " .. pid_str)
    return
  end

  lua_process_spawner.last_pid = pid
end)

function lua_process_spawner.spawn_lua_process(lua_path, cwd, cmd_line)
  local pid_reply_to = "/run/debug-app/lua-sp"
  cworks.publish("/run/sys/exec-lua", {
    CompoundFSObj = {
      children = {
        path = { String = lua_path },
        cwd = { String = cwd },
        cmd_line = { String = cmd_line },
        stdout = { String = env.stdout },
        stdin = { String = env.stdin },
        pid_reply_to = { String = pid_reply_to },
      }
    }
  })

  while lua_process_spawner.last_pid == nil do
    cworks.wait_for_event()
  end

  local pid = lua_process_spawner.last_pid
  lua_process_spawner.last_pid = nil

  return pid
end

--- end of child lua process spawner

local pwd = "/"
while true do
  print("\n\x1b[1;32m" .. pwd .. "\x1b[m")
  io.write("\x1b[2m$\x1b[m ")
  local line = io.read("*l")
  local command, args = line:match("^(%S+)%s*(.*)$")

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
  else
    local lua_path = "/usr/bin/" .. command .. ".lua"
    if cworks.stat(lua_path) then
      local pid = lua_process_spawner.spawn_lua_process(lua_path, pwd, line)
      cworks.wait_for_process(pid)
    else
      print("Unknown command: " .. command .. "")
    end
  end
end
