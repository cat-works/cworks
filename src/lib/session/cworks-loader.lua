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
  elseif sc_data["Handle"] ~= nil then
    return sc_data["Handle"]
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
  else
    print("Unknown data from kernel: " .. data)
  end

  return nil
end

function cworks.pass_poll_result(data)
  return dispatch(coroutine.yield(data))
end

function cworks.exit(retval)
  cworks.pass_poll_result(json.stringify({ Done = retval }))
end

function cworks.send(handle, data)
  cworks.pass_poll_result(json.stringify({ Send = { "$$bi:" .. handle, data } }))
end

function cworks.pending()
  cworks.pass_poll_result("\"Pending\"")
end

function cworks.sleep(seconds)
  cworks.pass_poll_result(json.stringify({ Sleep = seconds }))
end

function cworks.list(path)
  local ret = cworks.pass_poll_result(json.stringify({ List = path }))
  return ret
end

function cworks.stat(path)
  local ret = cworks.pass_poll_result(json.stringify({ Stat = path }))
  return ret
end

function cworks.mkdir(path, name)
  local ret = cworks.pass_poll_result(json.stringify({ Mkdir = { path, name } }))
  return ret
end

function cworks.get(path)
  local ret = cworks.pass_poll_result(json.stringify({ Get = path }))
  return ret
end

function cworks.set(path, data)
  local ret = cworks.pass_poll_result(json.stringify({ Set = { path, data } }))
  return ret
end

function cworks.subscribe(path, callback)
  channel_handlers[path] = callback
  local ret = cworks.pass_poll_result(json.stringify({ Subscribe = path }))
  return ret
end

function cworks.unsubscribe(path)
  channel_handlers[path] = nil
  local ret = cworks.pass_poll_result(json.stringify({ Unsubscribe = path }))
  return ret
end

function cworks.publish(path, data)
  local ret = cworks.pass_poll_result(json.stringify({ Publish = { path, data } }))
  return ret
end

package.loaded["cworks"] = cworks
