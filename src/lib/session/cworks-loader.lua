local json = require("json")

---Dumps the text in hex format
---@param s string
---@return string
local function hexdump(s)
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

---Escapes string for JSON
---@param s string
---@return string
local function json_escape(s)
  local result = ""
  for i = 1, #s do
    local c = s:sub(i, i)
    if c == '"' then
      result = result .. '\\"'
    elseif c == '\\' then
      result = result .. '\\\\'
    elseif c == '\b' then
      result = result .. '\\b'
    elseif c == '\f' then
      result = result .. '\\f'
    elseif c == '\n' then
      result = result .. '\\n'
    elseif c == '\r' then
      result = result .. '\\r'
    elseif c == '\t' then
      result = result .. '\\t'
    else
      local byte = string.byte(c)
      if byte < 32 or byte > 126 then
        result = result .. string.format("\\u%04x", byte)
      else
        result = result .. c
      end
    end
  end
  return result
end


---Table to store syscall handlers
---@type table<integer, function>
local syscall_handlers = {}

---Interprets the syscall data and dispatch callback or return it
---@param data string
---@return integer? handle
local function dispatch_syscall(data)
  local sc_data = json.parse(data)
  local syscall_type = -50
  if sc_data == "None" then            -- none
    return nil
  elseif sc_data["Handle"] ~= nil then -- handle
    return sc_data["Handle"]
  else
    print("Unknown syscall data: " .. data)
  end

  return nil
end

---do_syscall
---@param data string
local function do_syscall(data)
  return dispatch_syscall(coroutine.yield(data))
end

local function exit(retval)
  do_syscall("{\"Done\": " .. retval .. "}")
end

---Sends data to specified handle
---@param handle integer
---@param data string
local function send(handle, data)
  do_syscall("{\"Syscall\":{\"Send\":[\"$$bi:" .. handle .. "\", \"" .. json_escape(data) .. "\"]}}")
end

---Connects to the IPC socket
---@param socket_name string
---@param data_callback function
---@return integer handle
local function ipc_connect(socket_name, data_callback)
  local handle = do_syscall("{\"Syscall\":{\"IpcConnect\":\"" .. socket_name .. "\"}}")
  if handle then
    syscall_handlers[handle] = data_callback
    return handle
  else
    print("Failed to connect to IPC socket: " .. socket_name)
    return 0
  end
end

local function pending()
  do_syscall("Pending")
end

---Sleeps specified amount of time
---@param seconds number
local function sleep(seconds)
  do_syscall("{\"Syscall\":{\"Sleep\":" .. seconds .. "}}")
end

package.loaded["cworks"] = {
  -- misc
  hexdump = hexdump,

  -- syscall layer
  dispatch_syscall = dispatch_syscall,
  do_syscall = do_syscall,

  -- syscall
  exit = exit,
  send = send,
  ipc_connect = ipc_connect,
  pending = pending,
  sleep = sleep,
}
