--[[ stdio.lua

Internal module (package.loaded["__stdio"]) providing process-local stdio.

Exposes:
  stdio.build(env, cw) -> { print = print_fn, io = io_table, flush = flush_fn }

- Output sink is env.stdout (channel path). "" / nil disables the channel
  and falls back to C-level writes (real io.write on the main state).
- publish failures fall back to C-level writes (best-effort).
- stderr is redirected to stdout.
- Writes are deferred when not yieldable; flush points:
  next print/io.write, io.read wait start, run end, exit().
- env.stdin is required ("" is an error). Keystroke chunks are appended to a
  character-stream buffer; io.read supports the standard formats
  (table-driven: see read_handlers).
]]

local stdio = {}

local real_io = io
local c_write = io.write
local c_tostring = tostring
local c_type = type
local c_isyieldable = coroutine.isyieldable

local function to_write_arg(v, i)
  if c_type(v) == "string" then
    return v
  elseif c_type(v) == "number" then
    return tostring(v)
  end
  error("bad argument #" .. i .. " to 'write' (" .. c_tostring(v) .. ")", 2)
end

local function number_complete(buf)
  local ws = buf:match("^%s*")
  if ws == nil then
    return nil
  end
  local body = buf:sub(#ws + 1)
  if body == "" then
    return nil
  end

  local num, after = body:match("^([-+]?%d+%.?%d*[eE]?[-+]?%d*)(.*)$")
  if not num then
    num, after = body:match("^([-+]?%d*%.?%d+[eE]?[-+]?%d*)(.*)$")
  end
  if not num then
    return nil
  end

  if after == "" then
    return nil
  end

  local c = after:sub(1, 1)
  if c:match("[%d%.eE]") or c == "+" or c == "-" then
    return nil
  end

  local val = tonumber(num)
  if not val then
    return nil
  end
  return val, #ws + #num
end

-- Read handlers: fn(buf, fmt) -> drop_count, value | nil (not ready yet).
-- drop_count = number of leading chars to consume from buf.
local read_handlers = {
  a = function(buf)
    return #buf, buf
  end,
  l = function(buf)
    local nl = buf:find("\n", 1, true)
    if not nl then
      return nil
    end
    return nl, buf:sub(1, nl - 1)
  end,
  L = function(buf)
    local nl = buf:find("\n", 1, true)
    if not nl then
      return nil
    end
    return nl, buf:sub(1, nl)
  end,
  n = function(buf)
    local num, consumed = number_complete(buf)
    if not num then
      return nil
    end
    return consumed, num
  end,
  count = function(buf, n)
    if #buf < n then
      return nil
    end
    return n, buf:sub(1, n)
  end,
}

local function normalize_read_format(fmt)
  if c_type(fmt) == "number" then
    return "count"
  end
  if c_type(fmt) == "string" then
    return (fmt:gsub("^%*", ""))
  end
  return fmt
end

function stdio.build(env, cw)
  local out_path = env.stdout
  if out_path == nil or out_path == "" then
    out_path = nil
  end

  local in_path = env.stdin
  if in_path == nil or in_path == "" then
    error("stdio: env.stdin must be a channel path (empty string not allowed)")
  end

  local out_queue = {}
  local in_buf = ""

  local function deliver(chunk)
    if out_path == nil then
      c_write(chunk)
      return
    end
    local pub_ok, pub_res = pcall(function()
      return cw.publish(out_path, { String = chunk })
    end)
    if not pub_ok or pub_res ~= "FSSuccess" then
      c_write(chunk)
    end
  end

  local function emit_out(chunk)
    if c_isyieldable() then
      deliver(chunk)
    else
      out_queue[#out_queue + 1] = chunk
    end
  end

  local function flush_pending()
    if #out_queue == 0 or not c_isyieldable() then
      return
    end
    local joined = table.concat(out_queue)
    out_queue = {}
    deliver(joined)
  end

  cw.subscribe(in_path, function(caller, data)
    if c_type(data) ~= "table" or c_type(data["String"]) ~= "string" then
      return
    end
    local s = (data["String"]:gsub("\r\n", "\n"):gsub("\r", "\n"))
    in_buf = in_buf .. s
  end)

  local function format_print_args(...)
    local n = select("#", ...)
    local parts = {}
    for i = 1, n do
      parts[i] = c_tostring(select(i, ...))
    end
    return table.concat(parts, "\t")
  end

  local function stdout_write(...)
    flush_pending()
    local n = select("#", ...)
    local parts = {}
    for i = 1, n do
      parts[i] = to_write_arg(select(i, ...), i)
    end
    emit_out(table.concat(parts))
  end

  local function make_handle(write_fn, flush_fn)
    local h = {
      close = function()
        return true
      end,
      setvbuf = function(self)
        return self
      end,
    }
    if write_fn then
      h.write = function(self, ...)
        write_fn(...)
        return self
      end
    end
    if flush_fn then
      h.flush = function(self)
        flush_fn()
        return self
      end
    end
    return h
  end

  local stdout_handle = make_handle(stdout_write, flush_pending)
  local stderr_handle = make_handle(stdout_write, flush_pending)
  local stdin_handle = make_handle()

  local function read_one(fmt)
    local kind = normalize_read_format(fmt)
    local handler = read_handlers[kind]
    if not handler then
      error("invalid read format: " .. c_tostring(fmt))
    end
    while true do
      local drop, val = handler(in_buf, fmt)
      if drop ~= nil then
        in_buf = in_buf:sub(drop + 1)
        return val
      end
      flush_pending()
      cw.wait_for_event()
    end
  end

  local function read_fn(...)
    local n = select("#", ...)
    if n == 0 then
      return read_one("*l")
    end
    local results = {}
    for i = 1, n do
      results[i] = read_one(select(i, ...))
    end
    return table.unpack(results, 1, n)
  end

  local function lines_fn(...)
    local n = select("#", ...)
    local fmt = "*l"
    if n > 0 then
      local a = select(1, ...)
      if c_type(a) == "string" or c_type(a) == "number" then
        fmt = a
      end
    end
    return function()
      return read_one(fmt)
    end
  end

  local fake_io = {
    stdout = stdout_handle,
    stderr = stderr_handle,
    stdin = stdin_handle,
    write = function(...)
      return stdout_handle:write(...)
    end,
    read = read_fn,
    lines = lines_fn,
    flush = function()
      flush_pending()
    end,
  }

  for k, v in pairs(real_io) do
    if fake_io[k] == nil then
      fake_io[k] = v
    end
  end

  function fake_io.output(f)
    if f == nil then
      return stdout_handle
    end
    if c_type(f) == "string" then
      error("io.output: file paths are not supported")
    end
    return f
  end

  function fake_io.input(f)
    if f == nil then
      return stdin_handle
    end
    if c_type(f) == "string" then
      error("io.input: file paths are not supported")
    end
    return f
  end

  function fake_io.type(f)
    if f == stdout_handle or f == stderr_handle or f == stdin_handle then
      return "file"
    end
    return nil
  end

  function fake_io.open(...)
    error("io.open: file paths are not supported")
  end

  local function print_fn(...)
    flush_pending()
    emit_out(format_print_args(...) .. "\n")
  end

  return {
    print = print_fn,
    io = fake_io,
    flush = flush_pending,
  }
end

package.loaded["__stdio"] = stdio

return stdio
