local cworks = require("cworks").new(session, env)
local gui = require("gui").new(session, env)

local grid = gui.request {
  title = "Terminal",
  x = 60,
  y = 60,
  w = 560,
  h = 360,
}
grid:set("rows", "1fr")

local term = grid:embed("terminal", {
  id = "term",
  area = "0,0-1,1",
  channels = { "output", "input" },
})

local ready = false
term:on("ready", function()
  ready = true
end)
while not ready do
  cworks.wait_for_event()
end

local sh = cworks.get("/usr/bin/sh.lua"):string()
local req = cworks.root.new_compound()
req:create_string("name", "sh.lua")
req:create_string("code", sh)
req:create_string("cmd_line", "sh")
req:create_string("cwd", "/")
req:add_child("stdout", term.obj:get_obj("output"))
req:add_child("stdin", term.obj:get_obj("input"))
cworks.publish("/run/sys/exec-lua", req)

while true do
  cworks.wait_for_event()
end
