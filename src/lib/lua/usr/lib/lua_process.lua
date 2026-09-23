--- @class LuaProcessSpawner
--- @field session LuaSession
--- @field root FSObj
--- @field last_pid integer|nil
--- @field reply_to FSObj
--- @field request_to FSObj
--- @field default_stdout FSObj|nil
--- @field default_stdin FSObj|nil
local LuaProcessSpawner = {}
LuaProcessSpawner.__index = LuaProcessSpawner

--- Creates a new LuaProcessSpawner instance
--- @param session LuaSession
--- @param root FSObj
--- @param env Env
--- @return LuaProcessSpawner
function LuaProcessSpawner.new(session, root, env)
  local reply_to = root:ensure_compound("run"):ensure_compound("debug-app"):ensure_func("lua-sp")
  local request_to = root:ensure_compound("run"):ensure_compound("sys"):ensure_func("exec-lua")
  local obj = setmetatable({
    session = session,
    root = root,
    last_pid = nil,
    reply_to = reply_to,
    request_to = request_to,
    default_stdout = env.stdout,
    default_stdin = env.stdin,
  }, LuaProcessSpawner)
  session:subscribe(
    reply_to,
    function(data)
      local pid
      if data:type() == "Int" then
        pid = data:int()
      elseif data:type() == "String" then
        pid = tonumber(data:string())
      end
      if pid == nil then
        print("Invalid pid received: " .. data:type())
        return
      end

      obj.last_pid = pid
    end)

  return obj
end

--- Spawns lua process
--- @param name string
--- @param code string
--- @param cwd string
--- @param cmd_line string
--- @param stdout FSObj|nil
--- @param stdin FSObj|nil
--- @return number|unknown
function LuaProcessSpawner:spawn(name, code, cwd, cmd_line, stdout, stdin)
  local stdout_obj = stdout or self.default_stdout
  local stdin_obj = stdin or self.default_stdin

  if not stdout_obj then
    error("No stdout object provided and no default stdout available")
  end
  if not stdin_obj then
    error("No stdin object provided and no default stdin available")
  end

  local request = self.root:new_compound()
  request:create_string("name", name)
  request:create_string("code", code)
  request:create_string("cwd", cwd)
  request:create_string("cmd_line", cmd_line)
  request:add_child("stdout", stdout_obj)
  request:add_child("stdin", stdin_obj)
  request:add_child("pid_reply_to", self.reply_to)

  self.session:publish(self.request_to, request)

  while self.last_pid == nil do
    self.session:wait_for_event()
  end

  local pid = self.last_pid
  self.last_pid = nil

  return pid
end

local lua_process = {}
lua_process.LuaProcessSpawner = LuaProcessSpawner

return lua_process
