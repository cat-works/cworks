local path = require("path")

--- parse command line arguments (env.cmdline -> following)
local ls_path = ""
local recursive = false
local color = false

local first = true
for arg in env.cmdline:gmatch("%S+") do
  if first then
    first = false -- skip command name
  elseif arg == "-r" then
    recursive = true
  elseif arg == "--color" then
    color = true
  else
    ls_path = arg
  end
end

if path:sub(1, 1) ~= "/" then
  ls_path = path.join(env.cwd, ls_path)
end

local ls_obj = path.resolve_object(session:fs_root(), ls_path)
if not ls_obj then
  io.write("No such file or directory: " .. ls_path .. "\n")
  os.exit()
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

---List Recursive
---@param dir FSObj
---@param depth integer
local function ls_rec(dir, depth)
  local list = dir:list()
  table.sort(list)

  for _, item in ipairs(list) do
    if item == "." or item == ".." then
      goto continue
    end
    local obj = dir:get_obj(item)
    if not obj then
      print("Failed to get object for " .. item)
      goto continue
    end
    local kind = obj:type()
    if kind == "Func" then
      print(("  "):rep(depth) .. formatter.Channel(item))
    elseif kind == "CompoundFSObj" then
      print(("  "):rep(depth) .. formatter.Directory(item))
      if recursive then
        ls_rec(obj, depth + 1)
      end
    else
      print(("  "):rep(depth) .. formatter.File(item))
    end

    ::continue::
  end
end

ls_rec(ls_obj, 0)
