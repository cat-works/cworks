--[[ cworks.lua

Compatibility facade providing the path-based `cworks` API on top of FSObj
handles. Loaded per process via `require("cworks").new(session, env)`.
]]

local cworks = {}

local c_type = type
local c_pairs = pairs
local c_ipairs = ipairs
local c_error = error
local c_pcall = pcall

local function split_parts(path)
  local parts = {}
  for part in path:gmatch("[^/]+") do
    parts[#parts + 1] = part
  end
  return parts
end

local function split_dirname(path)
  local dir, name = path:match("^(.*)/([^/]+)$")
  if name == nil then
    return "", path
  end
  if dir == "" then
    return "/", name
  end
  return dir, name
end

local function resolve(root, path)
  local current = root
  for _, part in c_ipairs(split_parts(path)) do
    if part ~= "." then
      current = current:get_obj(part)
    end
  end
  return current
end

local function get_or_create(parent, name, create)
  if parent:has_child(name) then
    return parent:get_obj(name)
  end
  return create(parent, name)
end

local function ensure_dir(root, path)
  local current = root
  for _, part in c_ipairs(split_parts(path)) do
    if part ~= "." then
      current = get_or_create(current, part, function(parent, child)
        return parent:create_compound(child)
      end)
    end
  end
  return current
end

local function ensure_func(root, path)
  local dir, name = split_dirname(path)
  local parent = ensure_dir(root, dir)
  return get_or_create(parent, name, function(p, child)
    return p:create_func(child)
  end)
end

function cworks.new(session, env)
  local root = env.root

  local function fill_child(parent, name, value)
    local t = c_type(value)
    if t == "table" and value.CompoundFSObj ~= nil then
      local child = parent:create_compound(name)
      for key, item in c_pairs(value.CompoundFSObj.children or {}) do
        fill_child(child, key, item)
      end
    elseif t == "table" and value.String ~= nil then
      parent:create_string(name, value.String)
    elseif t == "table" and value.Int ~= nil then
      parent:create_int(name, value.Int)
    elseif t == "table" and value.Boolean ~= nil then
      parent:create_boolean(name, value.Boolean)
    elseif t == "table" and value.Float ~= nil then
      parent:create_float(name, value.Float)
    elseif t == "table" and value.Double ~= nil then
      parent:create_double(name, value.Double)
    elseif t == "table" and value.Null ~= nil then
      parent:create_null(name)
    elseif t == "table" and value.Func ~= nil then
      parent:create_func(name)
    elseif t == "string" then
      parent:create_string(name, value)
    elseif t == "number" then
      if value % 1 == 0 then
        parent:create_int(name, value)
      else
        parent:create_double(name, value)
      end
    elseif t == "boolean" then
      parent:create_boolean(name, value)
    else
      c_error("cworks: cannot convert child '" .. name .. "' of type " .. t)
    end
  end

  local function to_object(value)
    local t = c_type(value)
    if t == "string" then
      return root.new_string(value)
    elseif t == "number" then
      if value % 1 == 0 then
        return root.new_int(value)
      end
      return root.new_double(value)
    elseif t == "boolean" then
      return root.new_boolean(value)
    elseif t == "nil" then
      return root.new_null()
    elseif t == "table" then
      if value.CompoundFSObj ~= nil then
        local obj = root.new_compound()
        for key, item in c_pairs(value.CompoundFSObj.children or {}) do
          fill_child(obj, key, item)
        end
        return obj
      end
      if value.String ~= nil then return root.new_string(value.String) end
      if value.Int ~= nil then return root.new_int(value.Int) end
      if value.Boolean ~= nil then return root.new_boolean(value.Boolean) end
      if value.Float ~= nil then return root.new_float(value.Float) end
      if value.Double ~= nil then return root.new_double(value.Double) end
      if value.Null ~= nil then return root.new_null() end
      if value.Func ~= nil then return root.new_func() end
    end
    c_error("cworks: cannot convert value of type " .. t)
  end

  local function stat_of(path)
    local ok, obj = c_pcall(resolve, root, path)
    if not ok then
      return nil
    end
    local kind = "File"
    local obj_type = obj:type()
    if obj_type == "CompoundFSObj" then
      kind = "Directory"
    elseif obj_type == "Func" then
      kind = "Channel"
    end
    return { kind = kind, type = obj_type }
  end

  return {
    session = session,
    root = root,
    exit = env.exit,
    resolve = function(path)
      return resolve(root, path)
    end,
    to_obj = to_object,
    list = function(path)
      return resolve(root, path):list()
    end,
    get = function(path)
      return resolve(root, path)
    end,
    stat = stat_of,
    mkdir = function(path, name)
      return resolve(root, path):create_compound(name)
    end,
    set = function(path, value)
      local dir, name = split_dirname(path)
      fill_child(ensure_dir(root, dir), name, value)
      return true
    end,
    subscribe = function(path, handler)
      local obj = ensure_func(root, path)
      session:subscribe(obj, function(data)
        handler(nil, data)
      end)
    end,
    unsubscribe = function(path)
      session:unsubscribe(resolve(root, path))
    end,
    publish = function(path, value)
      local obj = value
      if c_type(value) ~= "userdata" then
        obj = to_object(value)
      end
      session:publish(ensure_func(root, path), obj)
    end,
    sleep = function(seconds)
      session:sleep(seconds)
    end,
    wait_for_event = function()
      session:wait_for_event()
    end,
    wait_for_process = function(pid)
      c_pcall(function()
        session:wait_for_process(pid)
      end)
    end,
    get_pid = function()
      return session:get_pid()
    end,
  }
end

return cworks
