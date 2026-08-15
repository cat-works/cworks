local cworks = require("cworks");
local json = require("json");

local io_buf_stdin = "";
local io_buf_stdin_line = "";

cworks.subscribe("/srv/stdio/root/in", function(caller, data)
  if data["String"] == nil then
    print("Invalid stdin data: " .. json.stringify(data))
    return
  end

  if data["String"] == "\n" then
    io_buf_stdin_line = io_buf_stdin
    io_buf_stdin = ""
  else
    io_buf_stdin = io_buf_stdin .. data["String"]
  end
end)

local function readline()
  while true do
    if io_buf_stdin_line == "" then
      cworks.pending()
      goto continue
    end

    break
    ::continue::
  end

  local line = io_buf_stdin_line
  io_buf_stdin_line = ""
  return line
end

local function write_stdout(data)
  cworks.publish("/srv/stdio/root/out", { String = data })
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

local editor_load_buffer = "";
cworks.mkdir("/srv", "shell")
cworks.mkdir("/srv/shell", "root")
cworks.subscribe("/srv/shell/root/push", function(caller, data)
  if data["String"] == nil then
    print("Invalid textarea data: " .. json.stringify(data))
    return
  end

  editor_load_buffer = data["String"]
end)
local function editor_take()
  editor_load_buffer = "";
  cworks.publish("/srv/textarea/root/take", { String = "/srv/shell/root/push" })
  while true do
    if editor_load_buffer == "" then
      cworks.pending()
      goto continue
    end

    break
    ::continue::
  end

  local buffer = editor_load_buffer
  editor_load_buffer = ""
  return buffer
end

local function editor_push(data)
  cworks.publish("/srv/textarea/root/push", { String = data })
end

local function ls_rec(dir, depth)
  local list = cworks.list(dir)

  if depth == 0 then
    write_stdout("/\n")
  else
    local indent = string.rep("  ", depth - 1) .. "- "
    write_stdout(indent .. "" .. dir .. "\n")
  end

  for _, item in ipairs(list) do
    if item == "." or item == ".." then
      goto continue
    end
    local st = cworks.stat(dir .. "/" .. item)
    local kind = st["kind"] ---@type string

    local indent = string.rep("  ", depth) .. "- "

    if kind == "Channel" then
      write_stdout(indent .. "\x1b[36m" .. item .. "\x1b[m\n")
    elseif kind == "File" then
      write_stdout(indent .. "\x1b[33m" .. item .. "\x1b[m\n")
    elseif kind == "Directory" then
      ls_rec(dir .. "/" .. item, depth + 1)
    else
      write_stdout(indent .. item .. "\n")
    end
    ::continue::
  end
end
write_stdout("\x1b[1;32mCat OS Shell\x1b[m\n")

ls_rec("", 0)


local pwd = "/"
while true do
  write_stdout("\n\x1b[1;32m" .. pwd .. "\x1b[m\n");
  write_stdout("\x1b[2m$\x1b[m ");
  local line = readline()
  local command, args = line:match("^(%S+)%s*(.*)$")
  if command == "ls" then
    local path = args ~= "" and args or pwd
    local list = cworks.list(path)
    for _, item in ipairs(list) do
      local st = cworks.stat(path .. "/" .. item)
      local kind = st["kind"] ---@type string
      write_stdout(kind:sub(0, 1) .. " " .. item .. "\n")
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
        write_stdout("No such directory: " .. new_path .. "\n")
      end
    end
  elseif command == "mkdir" then
    local name = args
    if name ~= "" then
      cworks.mkdir(pwd, name)
    else
      write_stdout("Usage: mkdir <name>\n")
    end
  elseif command == "stat" then
    if args == "" then
      write_stdout("Usage: stat <path>\n")
    else
      local st = cworks.stat(path_join(pwd, args))
      if st then
        write_stdout("Kind: " .. st["kind"] .. "\n")
      else
        write_stdout("No such file or directory: " .. path_join(pwd, args) .. "\n")
      end
    end
  elseif command == "get" then
    local result_file = path_join(pwd, args)
    local data = cworks.get(result_file)
    write_stdout("Content of " .. result_file .. ":\n" .. json.stringify(data) .. "\n")
  elseif command == "cat" then
    local result_file = path_join(pwd, args)
    local data = cworks.get(result_file)
    if data["String"] == nil then
      write_stdout("File is not a string: " .. result_file .. "\n")
    else
      write_stdout(data["String"] .. "\n")
    end
  elseif command == "set" then
    local file_name, content = args:match("^(%S+)%s+(.+)$")
    if file_name and content then
      local result_file = path_join(pwd, file_name)
      cworks.set(result_file, json.parse(content))
      write_stdout("Set content of " .. result_file .. "\n")
    else
      write_stdout("Usage: set <file_name> <content>\n")
    end
  elseif command == "take" then
    local result_file = path_join(pwd, args)
    local buffer = editor_take()
    cworks.set(result_file, { String = buffer })
  elseif command == "push" then
    local src_file = path_join(pwd, args)
    local src = cworks.get(src_file)
    if src["String"] == nil then
      write_stdout("File is not a string: " .. src_file .. "\n")
    else
      editor_push(src["String"])
    end
  elseif command == "clear" then
    write_stdout("\x1b[2J\x1b[H")
  elseif command == "exec" then
    local lua_path = path_join(pwd, args)
    local lua_code = cworks.get(lua_path)
    if lua_code["String"] == nil then
      write_stdout("File is not a string: " .. lua_path .. "\n")
    else
      cworks.publish("/run/sys/exec-lua", { String = lua_code["String"] })
    end
  else
    write_stdout("Unknown command: " .. command .. "\n")
  end
end
