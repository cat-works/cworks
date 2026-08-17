local cworks = require("cworks");
local json = require("json");

local stdio = {}
stdio.in_buf = "";
stdio.in_line = "";

cworks.subscribe("/run/debug-app/shell-in", function(caller, data)
  if data["String"] == nil then
    print("Invalid stdin data")
    return
  end

  if data["String"] == "\n" then
    stdio.in_line = stdio.in_buf
    stdio.in_buf = ""
  else
    stdio.in_buf = stdio.in_buf .. data["String"]
  end
end)

function stdio.readline()
  while stdio.in_line == "" do
    cworks.wait_for_event()
  end

  local line = stdio.in_line
  stdio.in_line = ""
  return line
end

function stdio.write(data)
  cworks.publish("/run/debug-app/shell-out", { String = data })
end

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

cworks.mkdir("/", "run")
cworks.mkdir("/run", "debug-app")

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
      stdio.write("\x1b[36m" .. path_join(dir, item) .. "\x1b[m\n")
    elseif kind == "File" then
      stdio.write("\x1b[33m" .. path_join(dir, item) .. "\x1b[m\n")
    elseif kind == "Directory" then
      ls_rec(dir .. "/" .. item, depth + 1)
    else
      stdio.write("Unknown kind: " .. kind .. " for " .. path_join(dir, item) .. "\n")
    end
    ::continue::
  end
end
stdio.write("\x1b[1;32mCat OS Shell\x1b[m\n")

ls_rec("", 0)


local pwd = "/"
while true do
  stdio.write("\n\x1b[1;32m" .. pwd .. "\x1b[m\n");
  stdio.write("\x1b[2m$\x1b[m ");
  local line = stdio.readline()
  local command, args = line:match("^(%S+)%s*(.*)$")
  if command == "ls" then
    local path = args ~= "" and args or pwd
    local list = cworks.list(path)
    for _, item in ipairs(list) do
      local st = cworks.stat(path .. "/" .. item)
      local kind = st["kind"] ---@type string
      stdio.write(kind:sub(0, 1) .. " " .. item .. "\n")
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
        stdio.write("No such directory: " .. new_path .. "\n")
      end
    end
  elseif command == "mkdir" then
    local name = args
    if name ~= "" then
      cworks.mkdir(pwd, name)
    else
      stdio.write("Usage: mkdir <name>\n")
    end
  elseif command == "stat" then
    if args == "" then
      stdio.write("Usage: stat <path>\n")
    else
      local st = cworks.stat(path_join(pwd, args))
      if st then
        stdio.write("Kind: " .. st["kind"] .. "\n")
      else
        stdio.write("No such file or directory: " .. path_join(pwd, args) .. "\n")
      end
    end
  elseif command == "get" then
    local result_file = path_join(pwd, args)
    local data = cworks.get(result_file)
    stdio.write("Content of " .. result_file .. ":\n" .. json.stringify(data) .. "\n")
  elseif command == "cat" then
    local result_file = path_join(pwd, args)
    local data = cworks.get(result_file)
    if data["String"] == nil then
      stdio.write("File is not a string: " .. result_file .. "\n")
    else
      stdio.write(data["String"] .. "\n")
    end
  elseif command == "set" then
    local file_name, content = args:match("^(%S+)%s+(.+)$")
    if file_name and content then
      local result_file = path_join(pwd, file_name)
      cworks.set(result_file, json.parse(content))
      stdio.write("Set content of " .. result_file .. "\n")
    else
      stdio.write("Usage: set <file_name> <content>\n")
    end
  elseif command == "take" then
    local result_file = path_join(pwd, args)
    local buffer = editor.take()
    cworks.set(result_file, { String = buffer })
  elseif command == "push" then
    local src_file = path_join(pwd, args)
    local src = cworks.get(src_file)
    if src["String"] == nil then
      stdio.write("File is not a string: " .. src_file .. "\n")
    else
      editor.push(src["String"])
    end
  elseif command == "clear" then
    stdio.write("\x1b[2J\x1b[H")
  elseif command == "exec" then
    local lua_path = path_join(pwd, args)
    local lua_code = cworks.get(lua_path)
    if lua_code["String"] == nil then
      stdio.write("File is not a string: " .. lua_path .. "\n")
    else
      cworks.publish("/run/sys/exec-lua", { String = lua_code["String"] })
    end
  else
    stdio.write("Unknown command: " .. command .. "\n")
  end
end
