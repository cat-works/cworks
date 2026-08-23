local ed_lib = {}

function ed_lib.new(cworks, load_back_channel, take_channel, push_channel)
  local self = {}
  self.take_buffer = ""
  cworks.subscribe(load_back_channel, function(caller, data)
    if data["String"] == nil then
      print("Invalid textarea data: " .. json.stringify(data))
      return
    end

    self.take_buffer = data["String"]
  end)
  function self.take()
    self.take_buffer = ""
    cworks.publish(take_channel, { String = load_back_channel })
    while self.take_buffer == "" do
      cworks.wait_for_event()
    end

    local buffer = self.take_buffer
    self.take_buffer = ""
    return buffer
  end

  function self.push(data)
    cworks.publish(push_channel, { String = data })
  end

  return self
end

return ed_lib
