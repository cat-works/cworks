-- make hexdump from 'string'
function hexdump(s)
  local out = {}
  for i = 1, #s do
    local c = s:sub(i, i)
    if c:match("%w") then
      out[#out + 1] = c
    else
      out[#out + 1] = string.format("0x%02x", string.byte(c))
    end
  end
  return table.concat(out, " ")
end

local a = arg
print("Hello from test_proc.lua!")
print("Type: " .. type(a))
print("Arg: " .. hexdump(a))

coroutine.yield("\1\0\0\0\0\0\0\0\0")

return 22; -- should not be runned
