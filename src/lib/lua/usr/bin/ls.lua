local cworks = env.cworks

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

--- parse command line arguments (env.cmdline -> following)
local path = ""
local recursive = false
local color = false

for arg in env.cmdline:gmatch("%S+") do
  if arg == "-r" then
    recursive = true
  elseif arg == "--color" then
    color = true
  else
    path = arg
  end
end

if path:sub(1, 1) ~= "/" then
  path = path_join(env.cwd, path)
end

--- parse command line arguments
if not cworks.stat(path) then
  io.write("No such file or directory: " .. path .. "\n")
  cworks.exit()
end

local formatter_color = {
  Channel = function(path)
    return "\x1b[1;36m" .. path .. "\x1b[m"
  end,
  File = function(path)
    return "\x1b[1;32m" .. path .. "\x1b[m"
  end,
  Directory = function(path)
    return "\x1b[2;33m" .. path .. "\x1b[m"
  end,
}

local formatter_plain = {
  Channel = function(path)
    return "C " .. path
  end,
  File = function(path)
    return "F " .. path
  end,
  Directory = function(path)
    return "D " .. path
  end,
}

local formatter = color and formatter_color or formatter_plain

local function ls_rec(cworks, dir, depth)
  local list = cworks.list(dir)
  table.sort(list)

  for _, item in ipairs(list) do
    if item == "." or item == ".." then
      goto continue
    end
    local st = cworks.stat(path_join(dir, item))
    local kind = st["kind"] ---@type string
    local display_name = recursive and path_join(dir, item) or item
    if kind == "Channel" then
      print(formatter.Channel(display_name))
    elseif kind == "File" then
      print(formatter.File(display_name))
    elseif kind == "Directory" then
      print(formatter.Directory(display_name))
      if recursive then
        ls_rec(cworks, path_join(dir, item), depth + 1)
      end
    else
      print("Unknown kind: " .. kind .. " for " .. path_join(dir, item) .. "")
    end

    ::continue::
  end
end

ls_rec(cworks, path, 0)
