local native_require = require
local json = package.loaded["json"]

if not json then
  error("cworks-loader: 'json' must be preloaded into package.loaded before this loader is installed")
end

local function fetch_module(modname)
  local module_path = "/usr/lib/" .. modname:gsub("%.", "/") .. ".lua"
  local sc_req_json = json.stringify({ Get = module_path })
  local sc_res_json = coroutine.yield(sc_req_json)
  local sc_res = json.parse(sc_res_json)

  if sc_res == nil then
    return nil, "syscall returned nil"
  end

  if sc_res == "None" or sc_res == "Pending" then
    return nil, "syscall returned " .. tostring(sc_res)
  end

  if sc_res["Fail"] ~= nil then
    return nil, tostring(sc_res["Fail"])
  end

  local content = sc_res["FSGet"]
  if content == nil or content["String"] == nil then
    return nil, "no module '" .. modname .. "' in cworks (" .. json.stringify(sc_res) .. ")"
  end

  return content["String"]
end

function require(modname)
  if type(modname) ~= "string" then
    return native_require(modname)
  end

  local loaded = package.loaded[modname]
  if loaded ~= nil then
    return loaded
  end

  local preload = package.preload[modname]
  if preload ~= nil then
    local ok, result = pcall(preload, modname)
    if not ok then
      error("error loading preload module '" .. modname .. "': " .. tostring(result), 2)
    end
    package.loaded[modname] = result or true
    return package.loaded[modname]
  end

  local content, err_msg = fetch_module(modname)
  if content == nil then
    error("module '" .. modname .. "' not found: " .. err_msg, 2)
  end

  local chunk, load_err = load(content, "@" .. "/usr/lib/" .. modname:gsub("%.", "/") .. ".lua")
  if not chunk then
    error("error loading module '" .. modname .. "': " .. tostring(load_err), 2)
  end

  local ok, result = pcall(chunk, modname)
  if not ok then
    error("error running module '" .. modname .. "': " .. tostring(result), 2)
  end

  package.loaded[modname] = result or true
  return package.loaded[modname]
end
