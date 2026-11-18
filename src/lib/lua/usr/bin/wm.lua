--[[ wm.lua

Default window manager for the Cat Workspace GUI.

The display server owns the root `Display` object and renders it. This process
manages windows (frames) on the Display:

  * creates a frame (decoration + content Grid) on `create`
  * replies to the requesting app with the content Grid
  * cascades placement, manages stacking/focus
  * applies WM-driven move/resize requests
  * removes frames on close

Protocol (see gui_app.ts):
  /run/gui/wm-req (DS -> WM) : attach, create, focus, move, resize
  /run/gui/wm-res (WM -> DS) : sync, ready
]]

local gui = require("gui").new(session, env)

local display = nil
local next_z = 1
local counter = 0

local function int_prop(obj, key, default)
  if obj:has_child(key) then
    local value = obj:get_obj(key)
    if value:type() == "Int" then
      return value:int()
    end
  end
  return default
end

local function str_prop(obj, key, default)
  if obj:has_child(key) then
    local value = obj:get_obj(key)
    if value:type() == "String" then
      return value:string()
    end
  end
  return default
end

local function changed(obj)
  if obj == nil then
    return
  end
  local msg = session:fs_root().new_compound()
  msg:create_string("op", "changed")
  obj:get_obj("control"):publish(msg)
end

local function publish_event(grid, event, fields)
  local msg = session:fs_root().new_compound()
  msg:create_string("event", event)
  if fields ~= nil then
    for key, value in pairs(fields) do
      if type(value) == "number" then
        msg:create_int(key, value)
      else
        msg:create_string(key, value)
      end
    end
  end
  grid:get_obj("event"):publish(msg)
end

local function body_of(window)
  return window:get_obj("content"):get_obj("body")
end

local function count_frames()
  if display == nil then
    return 0
  end
  local count = 0
  for _, name in ipairs(display:list()) do
    if name ~= "." and name ~= ".." then
      local child = display:get_obj(name)
      if child:type() == "CompoundFSObj" and child:has_child("kind") then
        count = count + 1
      end
    end
  end
  return count
end

local function remove_frame(window)
  if display == nil then
    return
  end
  local name = str_prop(window, "wm_name", nil)
  if name ~= nil then
    display:remove_child(name)
    changed(display)
  end
end

local function make_frame(title)
  counter = counter + 1
  local name = "win" .. tostring(counter)

  local frame = gui.widget("grid", {
    columns = 1,
    rows = "auto 1fr auto",
    background = "#1e2127",
    border = "1px solid #3a4150",
  })

  local bar = frame:grid({
    id = "titlebar",
    columns = "1fr auto",
    gap = 2,
    area = "0,0-1,1",
    background = "#2a303a",
  })
  bar:label({ id = "grip", text = title, drag = "move", area = "0,0-1,1" })
  local close = bar:button({ id = "close", text = "×", area = "0,1-1,2" })

  local body = frame:grid({ id = "body", area = "1,0-2,1", padding = 8 })

  local footer = frame:grid({
    id = "footer",
    columns = "1fr auto",
    area = "2,0-3,1",
    background = "#2a303a",
  })
  footer:label({ id = "spacer", text = "", area = "0,0-1,1" })
  footer:label({ id = "resize", text = "◢", drag = "resize", area = "0,1-1,2" })

  local body = frame:grid({ id = "body" })

  gui.set(frame.obj, "wm_name", name)

  close:on("click", function()
    remove_frame(frame.obj)
    publish_event(body.obj, "close")
  end)

  -- app-initiated close (grid:close())
  body.obj:get_obj("control"):subscribe(function(data)
    if data:type() == "CompoundFSObj"
        and data:has_child("op")
        and data:get_obj("op"):string() == "close"
    then
      remove_frame(frame.obj)
      publish_event(body.obj, "close")
    end
  end)

  return frame, body, name
end

local root = session:fs_root()
local wm_req = root:get_obj("run"):get_obj("gui"):ensure_func("wm-req")
local wm_res = root:get_obj("run"):get_obj("gui"):ensure_func("wm-res")
wm_req:subscribe(function(data)
  if data:type() ~= "CompoundFSObj" then
    return
  end
  local op = str_prop(data, "op", "")

  if op == "attach" then
    display = data:get_obj("display")
    if not display:has_child("background") then
      gui.set(display, "background", "transparent")
    end
    changed(display)
    local ready = root.new_compound()
    ready:create_string("op", "ready")
    wm_res:publish(ready)
  elseif op == "create" then
    if display == nil then
      return
    end
    local title = str_prop(data, "title", "Window")
    local frame, body, name = make_frame(title)
    local cascade = count_frames()
    gui.set(frame.obj, "x", int_prop(data, "x", 40) + cascade * 24)
    gui.set(frame.obj, "y", int_prop(data, "y", 40) + cascade * 24)
    gui.set(frame.obj, "w", int_prop(data, "w", 360))
    gui.set(frame.obj, "h", int_prop(data, "h", 260))
    gui.set(frame.obj, "z", next_z)
    next_z = next_z + 1
    display:add_child(name, frame.obj)
    changed(display)
    data:get_obj("reply_to"):publish(body.obj)
  elseif op == "focus" then
    local window = data:get_obj("window")
    gui.set(window, "z", next_z)
    next_z = next_z + 1
    changed(display)
    publish_event(body_of(window), "focus")
  elseif op == "move" then
    local window = data:get_obj("window")
    local x = int_prop(data, "x", 0)
    local y = int_prop(data, "y", 0)
    gui.set(window, "x", x)
    gui.set(window, "y", y)
    changed(display)
    publish_event(body_of(window), "move", { x = x, y = y })
  elseif op == "resize" then
    local window = data:get_obj("window")
    local w = int_prop(data, "w", 100)
    local h = int_prop(data, "h", 100)
    gui.set(window, "w", w)
    gui.set(window, "h", h)
    changed(display)
    publish_event(body_of(window), "resize", { w = w, h = h })
  end
end)

-- Ask the display server for the Display object.
local sync = root.new_compound()
sync:create_string("op", "sync")
wm_res:publish(sync)

while true do
  session:wait_for_event()
end
