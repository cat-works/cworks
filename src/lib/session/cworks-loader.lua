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

local channel_handlers = {} ---@type table<string, function>

---@param data string
local function dispatch(data)
  local sc_data = json.parse(data)
  if sc_data == "None" then
    return nil
  elseif sc_data["FSList"] ~= nil then
    return sc_data["FSList"]
  elseif sc_data["FSStat"] ~= nil then
    return sc_data["FSStat"]
  elseif sc_data["FSGet"] ~= nil then
    return sc_data["FSGet"]
  elseif sc_data == "FSSuccess" then
    return "FSSuccess"
  elseif sc_data["Invoke"] ~= nil then
    local caller = sc_data["Invoke"]["caller_pid"]
    local path = sc_data["Invoke"]["path"]
    local arg = sc_data["Invoke"]["arg"]
    if channel_handlers[path] then
      channel_handlers[path](caller, arg)
    else
      print("No handler for path: " .. path)
    end

    return dispatch("Pending")
  else
    print("Unknown data from kernel: " .. data)
  end

  return nil
end

function cworks.pass_poll_result(data)
  return dispatch(coroutine.yield(data))
end

function cworks.exit()
  cworks.pass_poll_result(json.stringify("Done"))
end

function cworks.wait_for_event()
  cworks.pass_poll_result("\"WaitForEvent\"")
end

function cworks.sleep(seconds)
  cworks.pass_poll_result(json.stringify({ Sleep = seconds }))
end

function cworks.list(path)
  return cworks.pass_poll_result(json.stringify({ List = path }))
end

function cworks.stat(path)
  return cworks.pass_poll_result(json.stringify({ Stat = path }))
end

function cworks.mkdir(path, name)
  return cworks.pass_poll_result(json.stringify({ Mkdir = { path, name } }))
end

function cworks.get(path)
  return cworks.pass_poll_result(json.stringify({ Get = path }))
end

function cworks.set(path, data)
  return cworks.pass_poll_result(json.stringify({ Set = { path, data } }))
end

function cworks.subscribe(path, callback)
  channel_handlers[path] = callback
  return cworks.pass_poll_result(json.stringify({ Subscribe = path }))
end

function cworks.unsubscribe(path)
  channel_handlers[path] = nil
  return cworks.pass_poll_result(json.stringify({ Unsubscribe = path }))
end

function cworks.publish(path, data)
  return cworks.pass_poll_result(json.stringify({ Publish = { path, data } }))
end

package.loaded["cworks"] = cworks
