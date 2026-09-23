local root = session:fs_root()
local lua_process_spawner = require("lua_process").LuaProcessSpawner.new(session, root, env)
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
  session:wait_for_event()
end

local sh_obj = root:get_obj("usr"):get_obj("bin"):get_obj("sh.lua")
if sh_obj == nil then
  error("sh.lua not found")
end
local sh = sh_obj:string()

lua_process_spawner:spawn("sh.lua", sh, "/", "sh", term.obj:get_obj("output"), term.obj:get_obj("input"))

while true do
  session:wait_for_event()
end
