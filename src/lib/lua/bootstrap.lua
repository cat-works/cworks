--[[ bootstrap.lua

Internal module (package.loaded["__bootstrap"]) providing:
  - the process runtime: per-process _ENV, handle construction, deferred exit
  - the module loader (merged from the old cworks.lua / cworks-loader.lua)

run(env, user_src, name):
  1. build the process handle (syscall core + API + per-process handler table)
  2. build stdio (print / io) from env via __stdio
  3. register thread -> handle in a weak-key registry
  4. env.cworks = handle
  5. build proc_env = { env, print, io, require } with __index = _G
  6. load and run user code with proc_env as the environment
  7. flush pending output and send "Done"
]]

local json = package.loaded["json"]

if not json then
  error("bootstrap: 'json' must be preloaded into package.loaded")
end

local bootstrap = {}

local handles = setmetatable({}, { __mode = "k" })

local native_require = require
local c_write = io.write
local c_tostring = tostring
local c_type = type

local ALWAYS = function()
  return true
end

local function module_path(modname)
  return "/usr/lib/" .. modname:gsub("%.", "/") .. ".lua"
end

local function fetch_done(r)
  return r == nil
      or r == "None"
      or r == "Pending"
      or c_type(r) ~= "table"
      or r["FSGet"] ~= nil
      or r["Fail"] ~= nil
end

local function parse_fetch_response(res, modname)
  if res == nil then
    return nil, "syscall returned nil"
  end
  if res == "None" then
    return nil, "syscall returned None"
  end
  if res == "Pending" then
    return nil, "syscall returned Pending"
  end
  if c_type(res) ~= "table" then
    return nil, "unexpected syscall response: " .. json.stringify(res)
  end
  if res["Fail"] ~= nil then
    return nil, c_tostring(res["Fail"])
  end

  local content = res["FSGet"]
  if content == nil or content["String"] == nil then
    return nil, "no module '" .. modname .. "' in cworks (" .. json.stringify(res) .. ")"
  end

  return content["String"]
end

local function fetch_module(modname, cw)
  local path = module_path(modname)

  local res
  if cw then
    res = cw.syscall({ Get = path }, fetch_done)
  else
    -- raw fallback for the narrow registry-miss window
    local sc_res_json = coroutine.yield(json.stringify({ Get = path }))
    res = json.parse(sc_res_json)
  end

  return parse_fetch_response(res, modname)
end

local function loader_core(modname, cw)
  if c_type(modname) ~= "string" then
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
      error("error loading preload module '" .. modname .. "': " .. c_tostring(result), 2)
    end
    package.loaded[modname] = result or true
    return package.loaded[modname]
  end

  local content, err_msg = fetch_module(modname, cw)
  if content == nil then
    error("module '" .. modname .. "' not found: " .. err_msg, 2)
  end

  local chunk, load_err = load(content, "@" .. module_path(modname))
  if not chunk then
    error("error loading module '" .. modname .. "': " .. c_tostring(load_err), 2)
  end

  -- circular require guard: mark loaded before running the chunk (native semantics)
  package.loaded[modname] = true

  local ok, result = pcall(chunk, modname)
  if not ok then
    error("error running module '" .. modname .. "': " .. c_tostring(result), 2)
  end

  package.loaded[modname] = result or true
  return package.loaded[modname]
end

local function build_handle()
  local handlers = {}

  local function syscall(req, until_fn)
    local cur_req = req
    while true do
      local sc_res_json = coroutine.yield(json.stringify(cur_req))
      local sc_res = json.parse(sc_res_json)

      if c_type(sc_res) == "table" and sc_res["Invoke"] ~= nil then
        local inv = sc_res["Invoke"]
        local path = inv["path"]
        local handler = handlers[path]
        if handler then
          local ok, err = pcall(handler, inv["caller_pid"], inv["arg"])
          if not ok then
            c_write("[bootstrap] handler error on " .. c_tostring(path) .. ": " .. c_tostring(err) .. "\n")
          end
        end
        cur_req = "Pending"
      elseif until_fn(sc_res) then
        return sc_res
      else
        cur_req = "Pending"
      end
    end
  end

  local function simple(req)
    local res = syscall(req, ALWAYS)
    if c_type(res) == "table" then
      if res["FSList"] ~= nil then
        return res["FSList"]
      end
      if res["FSStat"] ~= nil then
        return res["FSStat"]
      end
      if res["FSGet"] ~= nil then
        return res["FSGet"]
      end
      return nil
    end
    return res
  end

  local cw = {
    handlers = handlers,
    syscall = syscall,
    list = function(path)
      return simple({ List = path })
    end,
    stat = function(path)
      return simple({ Stat = path })
    end,
    get = function(path)
      return simple({ Get = path })
    end,
    set = function(path, data)
      return simple({ Set = { path, data } })
    end,
    mkdir = function(path, name)
      return simple({ Mkdir = { path, name } })
    end,
    subscribe = function(path, callback)
      handlers[path] = callback
      return simple({ Subscribe = path })
    end,
    unsubscribe = function(path)
      handlers[path] = nil
      return simple({ Unsubscribe = path })
    end,
    publish = function(path, data)
      return simple({ Publish = { path, data } })
    end,
    sleep = function(seconds)
      return simple({ Sleep = seconds })
    end,
    wait_for_event = function()
      return simple("WaitForEvent")
    end,
    wait_for_process = function(pid)
      return simple({ WaitForProcess = pid })
    end,
  }

  return cw
end

local function proc_require(cw)
  return function(modname)
    return loader_core(modname, cw)
  end
end

function bootstrap.run(env, user_src, name)
  local stdio_mod = package.loaded["__stdio"]
  if not stdio_mod then
    error("bootstrap: '__stdio' must be loaded before bootstrap")
  end

  local cw = build_handle()
  local st = stdio_mod.build(env, cw)

  cw.exit = function()
    st.flush()
    cw.syscall("Done", ALWAYS)
  end

  handles[coroutine.running()] = cw
  env.cworks = cw

  local proc_env = setmetatable({
    env = env,
    print = st.print,
    io = st.io,
    require = proc_require(cw),
  }, { __index = _G })

  local chunk, load_err = load(user_src, "@" .. c_tostring(name), "t", proc_env)
  if not chunk then
    error("error loading process '" .. c_tostring(name) .. "': " .. c_tostring(load_err))
  end

  local ok, run_err = pcall(chunk)
  if not ok then
    error(run_err, 0)
  end

  st.flush()
  cw.syscall("Done", ALWAYS)
end

require = function(modname)
  return loader_core(modname, handles[coroutine.running()])
end

package.loaded["__bootstrap"] = bootstrap

return bootstrap
