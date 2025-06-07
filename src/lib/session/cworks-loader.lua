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


---Table to store syscall handlers
---@type table<integer, function>
local syscall_handlers = {}

---Interprets the syscall data and dispatch callback or return it
---@param data string
---@return integer? handle
local function dispatch_syscall(data)
  local syscall_type = string.byte(data, 1)
  local syscall_data = data:sub(2)

  if syscall_type == 0x00 then                              -- none
    return nil
  elseif 0x01 <= syscall_type and syscall_type <= 0x06 then -- some failure
    print("Syscall failure: " .. syscall_type)
    return syscall_type
  elseif syscall_type == 0x07 then -- handle
    local handle = string.unpack(">I16", syscall_data)
    return handle
  elseif syscall_type == 0x08 then -- connection
    local client, server = string.unpack(">I16I16", syscall_data)
    -- not implemented!
    print("Connection syscall not implemented yet!")
    return nil
  elseif syscall_type == 0x09 then    -- receiving_data
    local handle = string.unpack(">I16", syscall_data)
    local data_content = data:sub(10) -- 1 byte for type, 8 bytes for handle
    if syscall_handlers[handle] then
      -- Call the handler with the received data
      syscall_handlers[handle](data_content)
    else
      print("No handler for handle: " .. handle)
    end
    return nil
  end

  return nil
end

---do_syscall
---@param data string
local function do_syscall(data)
  return dispatch_syscall(coroutine.yield(data))
end

local function exit(retval)
  -- pack retval as follows:
  -- 1 byte: type (0x01 for string)
  -- 8 bytes: retval (8byte big-endian integer)
  local retval_bytes = string.pack(">I8", retval)

  do_syscall(string.char(0x01) .. retval_bytes)
end

---Sends data to specified handle
---@param handle integer
---@param data string
local function send(handle, data)
  local handle_bytes = string.pack(">I16", handle)
  do_syscall(string.char(0x05) .. handle_bytes .. data)
end

---Connects to the IPC socket
---@param socket_name string
---@param data_callback function
---@return integer handle
local function ipc_connect(socket_name, data_callback)
  local handle = do_syscall(string.char(0x04) .. socket_name)
  if handle then
    syscall_handlers[handle] = data_callback
    return handle
  else
    print("Failed to connect to IPC socket: " .. socket_name)
    return 0
  end
end

local function pending()
  do_syscall(string.char(0x00))
end

---Sleeps specified amount of time
---@param seconds number
local function sleep(seconds)
  local seconds_bytes = string.pack(">f", seconds)
  do_syscall(string.char(0x02) .. seconds_bytes)
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
