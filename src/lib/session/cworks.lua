local json = require("json")

local cworks = {}

local channel_handlers = {} ---@type table<string, function>

function cworks.syscall(sc_req)
  local sc_req_json = json.stringify(sc_req)
  local sc_res_json = coroutine.yield(sc_req_json)
  local sc_res = json.parse(sc_res_json)

  if sc_res == "None" then
    return nil
  elseif sc_res["FSList"] ~= nil then
    return sc_res["FSList"]
  elseif sc_res["FSStat"] ~= nil then
    return sc_res["FSStat"]
  elseif sc_res["FSGet"] ~= nil then
    return sc_res["FSGet"]
  elseif sc_res == "FSSuccess" then
    return "FSSuccess"
  elseif sc_res["Invoke"] ~= nil then
    local caller = sc_res["Invoke"]["caller_pid"]
    local path = sc_res["Invoke"]["path"]
    local arg = sc_res["Invoke"]["arg"]
    if channel_handlers[path] then
      channel_handlers[path](caller, arg)
    else
      print("No handler for path: " .. path)
    end

    return cworks.syscall("Pending")
  else
    print("Unknown data from kernel: " .. json.stringify(sc_res))
    return nil
  end
end

function cworks.exit()
  cworks.syscall("Done")
end

function cworks.wait_for_event()
  cworks.syscall("WaitForEvent")
end

function cworks.sleep(seconds)
  cworks.syscall({ Sleep = seconds })
end

function cworks.list(path)
  return cworks.syscall({ List = path })
end

function cworks.stat(path)
  return cworks.syscall({ Stat = path })
end

function cworks.mkdir(path, name)
  return cworks.syscall({ Mkdir = { path, name } })
end

function cworks.get(path)
  return cworks.syscall({ Get = path })
end

function cworks.set(path, data)
  return cworks.syscall({ Set = { path, data } })
end

function cworks.subscribe(path, callback)
  channel_handlers[path] = callback
  return cworks.syscall({ Subscribe = path })
end

function cworks.unsubscribe(path)
  channel_handlers[path] = nil
  return cworks.syscall({ Unsubscribe = path })
end

function cworks.publish(path, data)
  return cworks.syscall({ Publish = { path, data } })
end

return cworks
