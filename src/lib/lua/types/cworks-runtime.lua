---@meta
-- Runtime definitions for the cworks Lua environment.
--
-- These globals are injected per process by bootstrap.lua (they are not
-- require-able modules), so they are declared here for LuaLS completion.

---@class FSObj
---@field get_obj fun(self: FSObj, name: string): FSObj
---@field list fun(self: FSObj): string[]
---@field has_child fun(self: FSObj, name: string): boolean
---@field add_child fun(self: FSObj, name: string, obj: FSObj)
---@field remove_child fun(self: FSObj, name: string)
---@field type fun(self: FSObj): "Int"|"String"|"Boolean"|"Float"|"Double"|"Bytes"|"Null"|"CompoundFSObj"|"Func"
---@field int fun(self: FSObj): integer
---@field string fun(self: FSObj): string
---@field boolean fun(self: FSObj): boolean
---@field float fun(self: FSObj): number
---@field double fun(self: FSObj): number
---@field bytes fun(self: FSObj): integer[]
---@field new_int fun(value: integer): FSObj
---@field new_string fun(value: string): FSObj
---@field new_boolean fun(value: boolean): FSObj
---@field new_float fun(value: number): FSObj
---@field new_double fun(value: number): FSObj
---@field new_bytes fun(value: integer[]): FSObj
---@field new_null fun(): FSObj
---@field new_compound fun(): FSObj
---@field new_func fun(): FSObj
---@field create_int fun(self: FSObj, name: string, value: integer): FSObj
---@field create_string fun(self: FSObj, name: string, value: string): FSObj
---@field create_boolean fun(self: FSObj, name: string, value: boolean): FSObj
---@field create_float fun(self: FSObj, name: string, value: number): FSObj
---@field create_double fun(self: FSObj, name: string, value: number): FSObj
---@field create_bytes fun(self: FSObj, name: string, value: integer[]): FSObj
---@field create_null fun(self: FSObj, name: string): FSObj
---@field create_compound fun(self: FSObj, name: string): FSObj
---@field create_func fun(self: FSObj, name: string): FSObj
---@field ensure_int fun(self: FSObj, name: string, value: integer): FSObj
---@field ensure_string fun(self: FSObj, name: string, value: string): FSObj
---@field ensure_boolean fun(self: FSObj, name: string, value: boolean): FSObj
---@field ensure_float fun(self: FSObj, name: string, value: number): FSObj
---@field ensure_double fun(self: FSObj, name: string, value: number): FSObj
---@field ensure_bytes fun(self: FSObj, name: string, value: integer[]): FSObj
---@field ensure_null fun(self: FSObj, name: string): FSObj
---@field ensure_compound fun(self: FSObj, name: string): FSObj
---@field ensure_func fun(self: FSObj, name: string): FSObj
---@field subscribe fun(self: FSObj, callback: fun(data: FSObj))
---@field publish fun(self: FSObj, data: FSObj)

---@class LuaSession
---@field sleep fun(self: LuaSession, seconds: number)
---@field wait_for_event fun(self: LuaSession)
---@field wait_for_process fun(self: LuaSession, pid: integer)
---@field get_pid fun(self: LuaSession): integer
---@field fs_root fun(self: LuaSession): FSObj

---@class Env
---@field session LuaSession
---@field root FSObj
---@field cwd string
---@field cmdline string
---@field stdout FSObj|nil
---@field stdin FSObj|nil
---@field exit fun()

---@type LuaSession
session = nil

---@type Env
env = nil
