--[[ bootstrap.lua

Internal module (package.loaded["__bootstrap"]) providing the process runtime
and the module loader. The host stores the process environment (cwd, cmd_line,
stdout/stdin handles) under /run/proc/<pid>; this module reads it and runs the
user code with the process-local environment.
]]

local bootstrap = {}

local native_require = require
local c_type = type
local c_tostring = tostring
local c_error = error
local c_pcall = pcall

local EXIT = {}

local function load_module(root, modname)
  local current = root:get_obj("usr"):get_obj("lib")
  local parts = {}
  for part in modname:gmatch("[^%.]+") do
    parts[#parts + 1] = part
  end
  for i = 1, #parts - 1 do
    current = current:get_obj(parts[i])
  end
  return current:get_obj(parts[#parts] .. ".lua"):string()
end

local loader_core

local function proc_require(root)
  return function(modname)
    return loader_core(root, modname)
  end
end

loader_core = function(root, modname)
  if c_type(modname) ~= "string" then
    return native_require(modname)
  end

  local loaded = package.loaded[modname]
  if loaded ~= nil then
    return loaded
  end

  local preload = package.preload[modname]
  if preload ~= nil then
    local ok, result = c_pcall(preload, modname)
    if not ok then
      c_error("error loading preload module '" .. modname .. "': " .. c_tostring(result), 2)
    end
    package.loaded[modname] = result or true
    return package.loaded[modname]
  end

  local ok, content = c_pcall(load_module, root, modname)
  if not ok then
    c_error("module '" .. modname .. "' not found in cworks", 2)
  end

  local mod_env = setmetatable({ require = proc_require(root) }, { __index = _G })
  local chunk, load_err = load(content, "@" .. modname, "t", mod_env)
  if not chunk then
    c_error("error loading module '" .. modname .. "': " .. c_tostring(load_err), 2)
  end

  package.loaded[modname] = true

  local ran, result = c_pcall(chunk, modname)
  if not ran then
    c_error("error running module '" .. modname .. "': " .. c_tostring(result), 2)
  end

  package.loaded[modname] = result or true
  return package.loaded[modname]
end

function bootstrap.run(session, user_src, name)
  local stdio_mod = package.loaded["__stdio"]
  if not stdio_mod then
    c_error("bootstrap: '__stdio' must be loaded before bootstrap")
  end

  local root = session:fs_root()
  local pid = session:get_pid()
  local proc_ns = root:get_obj("run"):get_obj("proc"):get_obj(tostring(pid))

  local function opt(child)
    local ok, value = c_pcall(function()
      return proc_ns:get_obj(child)
    end)
    if ok then
      return value
    end
    return nil
  end

  local function opt_string(child)
    local value = opt(child)
    if value == nil then
      return nil
    end
    return value:string()
  end

  local env = {
    session = session,
    root = root,
    cwd = opt_string("cwd") or "/",
    cmdline = opt_string("cmd_line") or "",
    stdout = opt("stdout"),
    stdin = opt("stdin"),
  }

  local st = stdio_mod.build(env, session)

  local fake_os = {};
  -- clone os to fake_os
  for k, v in pairs(os) do
    fake_os[k] = v
  end
  fake_os.exit = function()
    st.flush()
    c_error(EXIT)
  end

  local proc_env = setmetatable({
    env = env,
    session = session,
    print = st.print,
    io = st.io,
    os = fake_os,
    require = proc_require(root),
  }, { __index = _G })

  local chunk, load_err = load(user_src, "@" .. c_tostring(name), "t", proc_env)
  if not chunk then
    c_error("error loading process '" .. c_tostring(name) .. "': " .. c_tostring(load_err))
  end

  local ok, run_err = c_pcall(chunk)
  if not ok and run_err ~= EXIT then
    c_error(run_err, 0)
  end

  st.flush()
end

package.loaded["__bootstrap"] = bootstrap

return bootstrap
