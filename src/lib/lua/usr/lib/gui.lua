--[[ gui.lua

Widget protocol client for the Cat Workspace GUI server (display server).

A widget is a CompoundFSObj value with:

  kind    (String)         : "grid" | "label" | "button" | "input" | "html"
  control (Func)           : change signals sent to the display server / WM
  event   (Func)           : user events delivered from the display server / WM
  content (CompoundFSObj)  : child widgets
  <props> (String/Int/...) : widget-specific properties (text, value, area, ...)

The FSObj tree is the single source of truth. `control` only carries a
"changed" signal; the display server re-reads the node.

Window creation:
  local grid = gui.request{ title = "Demo" }   -- asks the DS, the WM replies
                                               -- with the window's content Grid
  grid:label{ text = "hello" }

The window itself (frame, decoration, geometry) is owned by the window manager
process. `gui.request` returns only the content Grid the app should populate.
]]

local cworks_mod = require("cworks")

local gui = {}

local function set_prop(obj, key, value)
  if obj:has_child(key) then
    obj:remove_child(key)
  end
  local t = type(value)
  if t == "string" then
    obj:create_string(key, value)
  elseif t == "number" then
    if value % 1 == 0 then
      obj:create_int(key, value)
    else
      obj:create_double(key, value)
    end
  elseif t == "boolean" then
    obj:create_boolean(key, value)
  else
    error("gui: unsupported property type '" .. t .. "' for '" .. key .. "'")
  end
end

function gui.new(session, env)
  local root = env.root
  local cworks = cworks_mod.new(session, env)
  local control_ch = cworks.resolve("/run/gui/req")

  local counter = 0

  local function make_widget(kind)
    local widget = root.new_compound()
    widget:create_string("kind", kind)
    widget:add_child("control", root.new_func())
    widget:add_child("event", root.new_func())
    widget:add_child("content", root.new_compound())
    return widget
  end

  local function signal(widget)
    local msg = root.new_compound()
    msg:create_string("op", "changed")
    session:publish(widget.obj:get_obj("control"), msg)
  end

  local Widget = {}
  Widget.__index = Widget

  local function wrap(obj, parent, name)
    return setmetatable(
      { obj = obj, parent = parent, name = name, handlers = {}, subscribed = false, children = {} },
      Widget
    )
  end

  function Widget:_content()
    return self.obj:get_obj("content")
  end

  function Widget:set(key, value)
    set_prop(self.obj, key, value)
    signal(self)
    return self
  end

  function Widget:set_text(value)
    return self:set("text", value)
  end

  function Widget:set_value(value)
    return self:set("value", value)
  end

  function Widget:set_html(value)
    return self:set("html", value)
  end

  function Widget:set_title(value)
    return self:set("title", value)
  end

  function Widget:prop(key)
    if self.obj:has_child(key) then
      return self.obj:get_obj(key)
    end
    return nil
  end

  function Widget:value()
    local value = self:prop("value")
    if value == nil then
      return nil
    end
    return value:string()
  end

  function Widget:on(event, handler)
    self.handlers[event] = self.handlers[event] or {}
    table.insert(self.handlers[event], handler)
    if not self.subscribed then
      self.subscribed = true
      local widget = self
      session:subscribe(self.obj:get_obj("event"), function(data)
        local ev = { event = nil }
        if data:type() == "CompoundFSObj" then
          if data:has_child("event") then
            ev.event = data:get_obj("event"):string()
          end
          if data:has_child("value") then
            ev.value = data:get_obj("value"):string()
          end
          if data:has_child("x") then
            ev.x = data:get_obj("x"):int()
          end
          if data:has_child("y") then
            ev.y = data:get_obj("y"):int()
          end
          if data:has_child("w") then
            ev.w = data:get_obj("w"):int()
          end
          if data:has_child("h") then
            ev.h = data:get_obj("h"):int()
          end
        end
        local list = widget.handlers[ev.event]
        if list ~= nil then
          for _, fn in ipairs(list) do
            fn(ev)
          end
        end
      end)
    end
    return self
  end

  function Widget:add(kind, opts)
    opts = opts or {}
    counter = counter + 1
    local name = opts.id or ("w" .. tostring(counter))
    local child = make_widget(kind)
    for key, value in pairs(opts) do
      if key ~= "id" then
        set_prop(child, key, value)
      end
    end
    self:_content():add_child(name, child)
    local widget = wrap(child, self, name)
    self.children[name] = widget
    signal(self)
    return widget
  end

  function Widget:grid(opts)
    return self:add("grid", opts)
  end

  function Widget:label(opts)
    return self:add("label", opts)
  end

  function Widget:button(opts)
    return self:add("button", opts)
  end

  function Widget:input(opts)
    return self:add("input", opts)
  end

  function Widget:html(opts)
    return self:add("html", opts)
  end

  function Widget:get(name)
    if self.children[name] ~= nil then
      return self.children[name]
    end
    local child = self:_content():get_obj(name)
    local widget = wrap(child, self, name)
    self.children[name] = widget
    return widget
  end

  function Widget:remove()
    if self.parent ~= nil then
      self.parent:_content():remove_child(self.name)
      signal(self.parent)
      self.parent = nil
    end
  end

  function Widget:close()
    local msg = root.new_compound()
    msg:create_string("op", "close")
    session:publish(self.obj:get_obj("control"), msg)
  end

  local api = {}

  -- Build a standalone widget tree (used by the window manager to construct
  -- window frames). Not registered with the display server.
  function api.widget(kind, opts)
    local obj = make_widget(kind)
    if opts ~= nil then
      for key, value in pairs(opts) do
        if key ~= "id" then
          set_prop(obj, key, value)
        end
      end
    end
    return wrap(obj, nil, nil)
  end

  function api.set(obj, key, value)
    set_prop(obj, key, value)
  end

  function api.signal(obj)
    local msg = root.new_compound()
    msg:create_string("op", "changed")
    session:publish(obj:get_obj("control"), msg)
  end

  -- Ask the display server for a new window; the window manager replies with
  -- the window's content Grid. Blocks until the Grid arrives.
  function api.request(opts)
    opts = opts or {}
    local reply = root.new_func()
    local grid = nil
    session:subscribe(reply, function(data)
      grid = data
    end)

    local msg = root.new_compound()
    msg:create_string("op", "create")
    set_prop(msg, "title", opts.title or "Window")
    set_prop(msg, "options", opts.options or "")
    if opts.x ~= nil then
      set_prop(msg, "x", opts.x)
    end
    if opts.y ~= nil then
      set_prop(msg, "y", opts.y)
    end
    if opts.w ~= nil then
      set_prop(msg, "w", opts.w)
    end
    if opts.h ~= nil then
      set_prop(msg, "h", opts.h)
    end
    msg:add_child("reply_to", reply)
    session:publish(control_ch, msg)

    while grid == nil do
      session:wait_for_event()
    end

    return wrap(grid, nil, nil)
  end

  return api
end

return gui
