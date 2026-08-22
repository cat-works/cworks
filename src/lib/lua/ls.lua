local cworks = env.cworks;

local pwd = "/"
local args = ""
local path = args ~= "" and args or pwd
local list = cworks.list(path)
for _, item in ipairs(list) do
  local st = cworks.stat(path .. "/" .. item)
  local kind = st["kind"] ---@type string
  io.write(kind:sub(0, 1) .. " " .. item .. "\n")
end
