local cworks = env.cworks;
local json = require("json");

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

local editor = {}
editor.take_buffer = "";
cworks.subscribe("/run/debug-app/ta-load-back", function(caller, data)
  if data["String"] == nil then
    print("Invalid textarea data: " .. json.stringify(data))
    return
  end

  editor.take_buffer = data["String"]
end)
function editor.take()
  editor.take_buffer = "";
  cworks.publish("/run/debug-app/ta-take", { String = "/run/debug-app/ta-load-back" })
  while editor.take_buffer == "" do
    cworks.wait_for_event()
  end

  local buffer = editor.take_buffer
  editor.take_buffer = ""
  return buffer
end

function editor.push(data)
  cworks.publish("/run/debug-app/ta-push", { String = data })
end

local function ls_rec(dir, depth)
  local list = cworks.list(dir)
  table.sort(list)

  for _, item in ipairs(list) do
    if item == "." or item == ".." then
      goto continue
    end
    local st = cworks.stat(dir .. "/" .. item)
    local kind = st["kind"] ---@type string

    if kind == "Channel" then
      io.write("\x1b[36m" .. path_join(dir, item) .. "\x1b[m\n")
    elseif kind == "File" then
      io.write("\x1b[33m" .. path_join(dir, item) .. "\x1b[m\n")
    elseif kind == "Directory" then
      ls_rec(dir .. "/" .. item, depth + 1)
    else
      io.write("Unknown kind: " .. kind .. " for " .. path_join(dir, item) .. "\n")
    end
    ::continue::
  end
end
io.write("\x1b[1;32mCat OS Shell\x1b[m\n")

ls_rec("", 0)

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

function lua_process_spawner.spawn_lua_process(lua_path, cmd_line)
  local pid_reply_to = "/run/debug-app/lua-sp"
  cworks.publish("/run/sys/exec-lua", {
    CompoundFSObj = {
      children = {
        path = { String = lua_path },
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

local args = "ls"

local pid = lua_process_spawner.spawn_lua_process("/usr/bin/ls.lua", "ls")
cworks.wait_for_process(pid)


local pwd = "/"
while true do
  io.write("\n\x1b[1;32m" .. pwd .. "\x1b[m\n");
  io.write("  \x1b[2mcmdline = " .. tostring(env.cmdline) .. "\x1b[m\n")
  io.write("\x1b[2m$\x1b[m ");
  local line = io.read("*l")
  local command, args = line:match("^(%S+)%s*(.*)$")
  if command == "ls" then
    local path = args ~= "" and args or pwd
    local list = cworks.list(path)
    for _, item in ipairs(list) do
      local st = cworks.stat(path .. "/" .. item)
      local kind = st["kind"] ---@type string
      io.write(kind:sub(0, 1) .. " " .. item .. "\n")
    end
  elseif command == "cd" then
    if args == "" then
      pwd = "/"
    else
      local new_path = path_join(pwd, args)
      local st = cworks.stat(new_path)
      if st and st["kind"] == "Directory" then
        pwd = new_path
      else
        io.write("No such directory: " .. new_path .. "\n")
      end
    end
  elseif command == "mkdir" then
    local name = args
    if name ~= "" then
      cworks.mkdir(pwd, name)
    else
      io.write("Usage: mkdir <name>\n")
    end
  elseif command == "stat" then
    if args == "" then
      io.write("Usage: stat <path>\n")
    else
      local st = cworks.stat(path_join(pwd, args))
      if st then
        io.write("Kind: " .. st["kind"] .. "\n")
      else
        io.write("No such file or directory: " .. path_join(pwd, args) .. "\n")
      end
    end
  elseif command == "get" then
    local result_file = path_join(pwd, args)
    local data = cworks.get(result_file)
    io.write("Content of " .. result_file .. ":\n" .. json.stringify(data) .. "\n")
  elseif command == "cat" then
    local result_file = path_join(pwd, args)
    local data = cworks.get(result_file)
    if data["String"] == nil then
      io.write("File is not a string: " .. result_file .. "\n")
    else
      io.write(data["String"] .. "\n")
    end
  elseif command == "set" then
    local file_name, content = args:match("^(%S+)%s+(.+)$")
    if file_name and content then
      local result_file = path_join(pwd, file_name)
      cworks.set(result_file, json.parse(content))
      io.write("Set content of " .. result_file .. "\n")
    else
      io.write("Usage: set <file_name> <content>\n")
    end
  elseif command == "take" then
    local result_file = path_join(pwd, args)
    local buffer = editor.take()
    cworks.set(result_file, { String = buffer })
  elseif command == "push" then
    local src_file = path_join(pwd, args)
    local src = cworks.get(src_file)
    if src["String"] == nil then
      io.write("File is not a string: " .. src_file .. "\n")
    else
      editor.push(src["String"])
    end
  elseif command == "clear" then
    io.write("\x1b[2J\x1b[H")
  elseif command == "exec" then
    -- args = <rel_path> ...
    local rel_path = args:match("^(%S+)%s*")
    local lua_path = path_join(pwd, rel_path)
    local cmd_line = line
    local pid = lua_process_spawner.spawn_lua_process(lua_path, cmd_line)
    cworks.wait_for_process(pid)
  elseif command == "repl" then
    io.write("Entering REPL mode. Type 'exit' to leave.\n")
    -- 1 ==> 1
    -- print ==> print
    while true do
      io.write("\x1b[2m>>\x1b[m ")
      local repl_line = io.read("*l")
      if repl_line == "exit" then
        io.write("Exiting REPL mode.\n")
        break
      end
      local func, err = load(repl_line)
      if not func then
        io.write("Error: " .. err .. "\n")
      else
        local success, result = pcall(func)
        if not success then
          io.write("Error: " .. result .. "\n")
        else
          if result ~= nil then
            if type(result) == "table" then
              io.write("Result: " .. json.stringify(result) .. "\n")
            else
              io.write("Result: " .. tostring(result) .. "\n")
            end
          end
        end
      end
    end
  else
    io.write("Unknown command: " .. command .. "\n")
  end
end
