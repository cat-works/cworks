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

---Interprets the syscall data and dispatch callback or return it
---@param data string
---@return integer? handle
local function dispatch_syscall(data)
  print("Dispatching syscall with data: " .. hexdump(data))
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
    print("ReceivingData is not implemented yet!")
    return nil
  end

  return nil
end

local function exit(retval)
  -- pack retval as follows:
  -- 1 byte: type (0x01 for string)
  -- 8 bytes: retval (8byte big-endian integer)
  local retval_type = 0x01
  local retval_bytes = string.pack(">I8", retval)

  local packed_retval = string.char(retval_type) .. retval_bytes

  dispatch_syscall(coroutine.yield(packed_retval))
end

---Sends data to specified handle
---@param handle integer
---@param data string
local function send(handle, data)
  local handle_bytes = string.pack(">I16", handle)

  local packed_retval = string.char(0x05) .. handle_bytes .. data
  print("Sending data: " .. hexdump(packed_retval))

  dispatch_syscall(coroutine.yield(packed_retval))
end

local function ipc_connect(socket_name)
  return dispatch_syscall(coroutine.yield(string.char(0x04) .. socket_name))
end

local function pending()
  dispatch_syscall(coroutine.yield(string.char(0x00)))
end

---Sleeps specified amount of time
---@param seconds number
local function sleep(seconds)
  local seconds_bytes = string.pack(">f", seconds)


  dispatch_syscall(coroutine.yield(string.char(0x02) .. seconds_bytes))
end
