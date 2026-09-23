--[[ path.lua

Path helpers shared by the usr scripts.
]]

local path = {}

--- Join `name` onto `base`. An absolute `name` is returned as-is.
--- @param base string
--- @param name string
--- @return string
function path.join(base, name)
  if name:sub(1, 1) == "/" then
    return name
  end
  if base == nil or base == "" then
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

--- Split `path` into its parts, separated by `/`.
--- @param path_str string
--- @return string[]
function path.split(path_str)
  local parts = {}
  for part in path_str:gmatch("[^/]+") do
    parts[#parts + 1] = part
  end
  return parts
end

--- Resolves `path` from `root` and returns the object at that path.
--- @param root FSObj
--- @param resolve_path string
--- @return FSObj | nil
function path.resolve_object(root, resolve_path)
  local current = root
  for _, part in ipairs(path.split(resolve_path)) do
    if part == "." then
      goto continue
    end
    local success, obj = pcall(function()
      return current:get_obj(part)
    end)
    if not success or not obj then
      return nil
    end
    current = obj

    ::continue::
  end
  return current
end

return path
