--[[ gui.lua

Widget protocol client for the Cat Workspace GUI server.

A widget is a CompoundFSObj value with:

  kind    (String)         : "window" | "grid" | "label" | "button" | "input" | "html"
  control (Func)           : change/close signals sent to the GUI server
  event   (Func)           : user events delivered from the GUI server
  content (CompoundFSObj)  : child widgets (a window holds exactly one)
  <props> (String/Int/...) : widget-specific properties (text, value, area, ...)

The FSObj tree is the single source of truth. `control` only carries a
"changed" signal; the server re-reads the node. This keeps the application
side ready for a future kernel-level FSObj watch mechanism (the signal is
simply dropped then).

Usage:
  local gui = require("gui").new(session, env)
  local win = gui.window{ title = "Demo" }
  local grid = win:get_grid()
  grid:label{ id = "msg", text = "hello", area = "0,0-1,1" }
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
    return setmetatable({ obj = obj, parent = parent, name = name }, Widget)
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

  function Widget:set_options(value)
    return self:set("options", value)
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
    session:subscribe(self.obj:get_obj("event"), function(data)
      local ev = { event = event }
      if data:type() == "CompoundFSObj" then
        if data:has_child("event") then
          ev.event = data:get_obj("event"):string()
        end
        if data:has_child("value") then
          ev.value = data:get_obj("value"):string()
        end
      end
      handler(ev)
    end)
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
    local child = self:_content():get_obj(name)
    return wrap(child, self, name)
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
    msg:add_child("node", self.obj)
    session:publish(self.obj:get_obj("control"), msg)
  end

  local api = {}

  function api.window(opts)
    opts = opts or {}
    local win = make_widget("window")
    set_prop(win, "title", opts.title or "Window")
    set_prop(win, "options", opts.options or "")
    if opts.x ~= nil then
      set_prop(win, "x", opts.x)
    end
    if opts.y ~= nil then
      set_prop(win, "y", opts.y)
    end
    if opts.w ~= nil then
      set_prop(win, "w", opts.w)
    end
    if opts.h ~= nil then
      set_prop(win, "h", opts.h)
    end

    local grid = make_widget("grid")
    set_prop(grid, "columns", opts.columns or 1)
    set_prop(grid, "gap", opts.gap or 0)
    win:get_obj("content"):add_child("0", grid)

    session:publish(control_ch, win)

    local window = wrap(win, nil, nil)
    window.grid = wrap(grid, window, "0")
    return window
  end

  function Widget:get_grid()
    return self.grid
  end

  return api
end

return gui
