local cworks = require("cworks");

local stdio = {}

function stdio.write(data)
  cworks.publish("/run/debug-app/shell-out", { String = data })
end

local pwd = "/"
local args = ""
local path = args ~= "" and args or pwd
local list = cworks.list(path)
for _, item in ipairs(list) do
  local st = cworks.stat(path .. "/" .. item)
  local kind = st["kind"] ---@type string
  stdio.write(kind:sub(0, 1) .. " " .. item .. "\n")
end
