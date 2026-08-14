local cworks = require("cworks");
local json = require("json");

local io_buf_stdin = "";
local io_buf_stdin_line = "";
local io_handle = cworks.ipc_connect("system/stdio/root", function(s)
  if s:find("\n") then
    io_buf_stdin_line = io_buf_stdin_line .. io_buf_stdin;
    io_buf_stdin = "";
  else
    io_buf_stdin = io_buf_stdin .. s;
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
local editor = cworks.ipc_connect("system/textarea/root", function(s)
  if s:sub(0, 1) == "l" then
    editor_load_buffer = editor_load_buffer .. s:sub(2)
  else
    print("Unknown editor data: " .. s)
  end
end)
local function editor_take()
  editor_load_buffer = "";
  cworks.send(editor, "a")
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

local function editor_push(path, data)
  cworks.send(editor, "l" .. data)
end

cworks.send(io_handle, "\x1b[1;32mCat OS Shell\x1b[m\n")
cworks.send(io_handle, "Type 'man commands' to see available commands.\n\n")

local list = cworks.list("/")
for _, item in ipairs(list) do
  local st = cworks.stat("/" .. item)
  local kind = st["kind"] ---@type string
  cworks.send(io_handle, kind:sub(0, 1) .. " " .. item .. "\n")
end

local pwd = "/"
while true do
  cworks.send(io_handle, "\x1b[1;32m" .. pwd .. "\x1b[m\n");
  cworks.send(io_handle, "\x1b[2m$\x1b[m ");
  local line = readline()
  local command, args = line:match("^(%S+)%s*(.*)$")
  if command == "ls" then
    local path = args ~= "" and args or pwd
    local list = cworks.list(path)
    for _, item in ipairs(list) do
      local st = cworks.stat(path .. "/" .. item)
      local kind = st["kind"] ---@type string
      cworks.send(io_handle, kind:sub(0, 1) .. " " .. item .. "\n")
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
        cworks.send(io_handle, "No such directory: " .. new_path .. "\n")
      end
    end
  elseif command == "mkdir" then
    local name = args
    if name ~= "" then
      cworks.mkdir(pwd, name)
    else
      cworks.send(io_handle, "Usage: mkdir <name>\n")
    end
  elseif command == "stat" then
    if args == "" then
      cworks.send(io_handle, "Usage: stat <path>\n")
    else
      local st = cworks.stat(path_join(pwd, args))
      if st then
        cworks.send(io_handle, "Kind: " .. st["kind"] .. "\n")
      else
        cworks.send(io_handle, "No such file or directory: " .. path_join(pwd, args) .. "\n")
      end
    end
  elseif command == "get" then
    local result_file = path_join(pwd, args)
    local data = cworks.get(result_file)
    cworks.send(io_handle, "Content of " .. result_file .. ":\n" .. json.stringify(data) .. "\n")
  elseif command == "cat" then
    local result_file = path_join(pwd, args)
    local data = cworks.get(result_file)
    if data["String"] == nil then
      cworks.send(io_handle, "File is not a string: " .. result_file .. "\n")
    else
      cworks.send(io_handle, data["String"] .. "\n")
    end
  elseif command == "set" then
    local file_name, content = args:match("^(%S+)%s+(.+)$")
    if file_name and content then
      local result_file = path_join(pwd, file_name)
      cworks.set(result_file, json.parse(content))
      cworks.send(io_handle, "Set content of " .. result_file .. "\n")
    else
      cworks.send(io_handle, "Usage: set <file_name> <content>\n")
    end
  elseif command == "take" then
    local result_file = path_join(pwd, args)
    local buffer = editor_take()
    cworks.set(result_file, { String = buffer })
  elseif command == "push" then
    local src_file = path_join(pwd, args)
    local src = cworks.get(src_file)
        if src["String"] == nil then
            cworks.send(io_handle, "File is not a string: " .. src_file .. "\n")
        else
            editor_push(src_file, src["String"])
        end
      elseif command=="clear" then
        cworks.send(io_handle, "\x1b[2J\x1b[H")
  else
    cworks.send(io_handle, "Unknown command: " .. command .. "\n")
  end
end
