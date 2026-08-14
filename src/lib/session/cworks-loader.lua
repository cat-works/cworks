local json = require("json")

local cworks = {}

---Dumps the text in hex format
---@param s string
---@return string
function cworks.hexdump(s)
  local out = {}
  for i = 1, #s do
    local c = s:sub(i, i)
    if c:match("%w") then
      out[#out + 1] = c
    else
      out[#out + 1] = string.format("0x%02x", string.byte(c))
    end
  end
  return table.concat(out, " ")
end

---Table to store syscall handlers
---@type table<integer, function>
local syscall_handlers = {}

---Interprets the syscall data and dispatch callback or return it
---@param data string
---@return integer? handle
local function dispatch_syscall(data)
  local sc_data = json.parse(data)
  if sc_data == "None" then
    return nil
  elseif sc_data["Handle"] ~= nil then
    return sc_data["Handle"]
  elseif sc_data["FSList"] ~= nil then
    return sc_data["FSList"]
  elseif sc_data["FSStat"] ~= nil then
    return sc_data["FSStat"]
  elseif sc_data["ReceivingData"] ~= nil then
    local handle = sc_data["ReceivingData"]["focus"]
    local data = sc_data["ReceivingData"]["data"]
    if syscall_handlers[handle] then
      syscall_handlers[handle](data)
    else
      print("No handler for handle: " .. handle)
    end
  else
    print("Unknown syscall data: " .. data)
  end

  return nil
end

function cworks.do_syscall(data)
  return dispatch_syscall(coroutine.yield(data))
end

function cworks.exit(retval)
  cworks.do_syscall(json.stringify({ Done = retval }))
end

function cworks.send(handle, data)
  cworks.do_syscall(json.stringify({ Syscall = { Send = { "$$bi:" .. handle, data } } }))
end

function cworks.ipc_connect(socket_name, data_callback)
  local handle = cworks.do_syscall(json.stringify({ Syscall = { IpcConnect = socket_name } }))
  if handle then
    syscall_handlers[handle] = data_callback
    return handle
  else
    print("Failed to connect to IPC socket: " .. socket_name)
    return 0
  end
end

function cworks.pending()
  cworks.do_syscall("\"Pending\"")
end

function cworks.sleep(seconds)
  cworks.do_syscall(json.stringify({ Syscall = { Sleep = seconds } }))
end

function cworks.list(path)
  local ret = cworks.do_syscall(json.stringify({ Syscall = { List = path } }))
  return ret
end

function cworks.stat(path)
  local ret = cworks.do_syscall(json.stringify({ Syscall = { Stat = path } }))
  return ret
end

function cworks.mkdir(path, name)
  local ret = cworks.do_syscall(json.stringify({ Syscall = { Mkdir = { path, name } } }))
  return ret
end

package.loaded["cworks"] = cworks
