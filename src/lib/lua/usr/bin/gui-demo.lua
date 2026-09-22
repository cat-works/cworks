local cworks = require("cworks").new(session, env)
local gui = require("gui").new(session, env)

local win = gui.window {
  title = "GUI Demo",
  options = "+StandardApp",
  x = 40,
  y = 40,
  w = 360,
  h = 240,
}
local grid = win:get_grid()

grid:label { id = "msg", text = "Hello, world!", area = "0,0-1,2" }
local name_input = grid:input {
  id = "name",
  type = "text",
  placeholder = "your name",
  area = "1,0-2,2",
}
local greet = grid:button { id = "greet", text = "Greet", area = "2,0-3,1" }

local current_name = ""

name_input:on("input", function(ev)
  current_name = ev.value or ""
end)

greet:on("click", function()
  local text = "Hello!"
  if current_name ~= "" then
    text = "Hello, " .. current_name .. "!"
  end
  grid:get("msg"):set_text(text)
end)

win:on("close", function()
  win:close()
end)

while true do
  cworks.wait_for_event()
end
