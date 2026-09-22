---@meta

-- Runtime definitions for the cworks Lua environment.
--
-- These globals are injected per process by bootstrap.lua (they are not
-- require-able modules), so they are declared here for LuaLS completion.

---@class FSObj
---@field [string] any
local FSObj = {}

-- directory-like methods ------------------------------------------------

---@param name string
---@return FSObj
function FSObj:get_obj(name) end

---@return string[]
function FSObj:list() end

---@param name string
---@return boolean
function FSObj:has_child(name) end

---@param name string
---@param obj FSObj
function FSObj:add_child(name, obj) end

---@param name string
function FSObj:remove_child(name) end

-- type inspection -------------------------------------------------------

---@return "Int"|"String"|"Boolean"|"Float"|"Double"|"Bytes"|"Null"|"CompoundFSObj"|"Func"
function FSObj:type() end

---@return string
function FSObj:string() end

---@return integer
function FSObj:int() end

---@return boolean
function FSObj:boolean() end

---@return number
function FSObj:float() end

---@return number
function FSObj:double() end

---@return integer[]
function FSObj:bytes() end

-- value constructors (static) -------------------------------------------

---@param value integer
---@return FSObj
function FSObj.new_int(value) end

---@param value string
---@return FSObj
function FSObj.new_string(value) end

---@param value boolean
---@return FSObj
function FSObj.new_boolean(value) end

---@param value number
---@return FSObj
function FSObj.new_float(value) end

---@param value number
---@return FSObj
function FSObj.new_double(value) end

---@param value integer[]
---@return FSObj
function FSObj.new_bytes(value) end

---@return FSObj
function FSObj.new_null() end

---@return FSObj
function FSObj.new_compound() end

---@return FSObj
function FSObj.new_func() end

-- child constructors ----------------------------------------------------

---@param name string
---@param value integer
---@return FSObj
function FSObj:create_int(name, value) end

---@param name string
---@param value string
---@return FSObj
function FSObj:create_string(name, value) end

---@param name string
---@param value boolean
---@return FSObj
function FSObj:create_boolean(name, value) end

---@param name string
---@param value number
---@return FSObj
function FSObj:create_float(name, value) end

---@param name string
---@param value number
---@return FSObj
function FSObj:create_double(name, value) end

---@param name string
---@param value integer[]
---@return FSObj
function FSObj:create_bytes(name, value) end

---@param name string
---@return FSObj
function FSObj:create_null(name) end

---@param name string
---@return FSObj
function FSObj:create_compound(name) end

---@param name string
---@return FSObj
function FSObj:create_func(name) end

-- child ensure (get-or-create) ------------------------------------------

---@param name string
---@param value integer
---@return FSObj
function FSObj:ensure_int(name, value) end

---@param name string
---@param value string
---@return FSObj
function FSObj:ensure_string(name, value) end

---@param name string
---@param value boolean
---@return FSObj
function FSObj:ensure_boolean(name, value) end

---@param name string
---@param value number
---@return FSObj
function FSObj:ensure_float(name, value) end

---@param name string
---@param value number
---@return FSObj
function FSObj:ensure_double(name, value) end

---@param name string
---@param value integer[]
---@return FSObj
function FSObj:ensure_bytes(name, value) end

---@param name string
---@return FSObj
function FSObj:ensure_null(name) end

---@param name string
---@return FSObj
function FSObj:ensure_compound(name) end

---@param name string
---@return FSObj
function FSObj:ensure_func(name) end

-- session ---------------------------------------------------------------

---@class LuaSession
local LuaSession = {}

---@param obj FSObj
---@param handler fun(data: FSObj)
function LuaSession:subscribe(obj, handler) end

---@param obj FSObj
function LuaSession:unsubscribe(obj) end

---@param obj FSObj
---@param data FSObj
function LuaSession:publish(obj, data) end

---@param seconds number
function LuaSession:sleep(seconds) end

function LuaSession:wait_for_event() end

---@param pid integer
function LuaSession:wait_for_process(pid) end

---@return integer
function LuaSession:get_pid() end

---@return FSObj
function LuaSession:fs_root() end

-- process environment ---------------------------------------------------

---@class Env
---@field session LuaSession
---@field root FSObj
---@field cwd string
---@field cmdline string
---@field stdout FSObj|nil
---@field stdin FSObj|nil
---@field exit fun()

-- globals injected per process ------------------------------------------

---@type LuaSession
session = nil

---@type Env
env = nil
