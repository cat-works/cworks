local gui = require("gui").new(session, env)

local grid = gui.request {
  title = "Edit text",
  options = "+StandardApp",
  x = 100,
  y = 100,
  w = 400,
  h = 200,
}

grid:input {
  id = "content",
  type = "text",
  placeholder = "Enter Content",
  area = "0,0-2,2",
}


while true do
  session:wait_for_event()
end
