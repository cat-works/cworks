--[[ path.lua

Path helpers shared by the usr scripts.
]]

local path = {}

-- Join `name` onto `base`. An absolute `name` is returned as-is.
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

return path
