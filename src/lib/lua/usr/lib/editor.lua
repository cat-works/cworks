local ed_lib = {}

---create a new editor instance
---@param session LuaSession
---@param load_back_channel FSObj
---@param take_channel FSObj
---@param push_channel FSObj
---@return table
function ed_lib.new(session, load_back_channel, take_channel, push_channel)
  local self = {}
  self.take_buffer = ""
  session:subscribe(load_back_channel, function(data)
    if data:type() ~= "String" then
      print("Invalid textarea data: " .. data:type())
      return
    end

    self.take_buffer = data:string()
  end)
  function self.take()
    self.take_buffer = ""
    session:publish(take_channel, load_back_channel)
    while self.take_buffer == "" do
      session:wait_for_event()
    end

    local buffer = self.take_buffer
    self.take_buffer = ""
    return buffer
  end

  function self.push(data)
    session:publish(push_channel, { String = data })
  end

  return self
end

return ed_lib
