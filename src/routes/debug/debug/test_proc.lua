local cworks = require("cworks");

local io_handle = cworks.ipc_connect("system/stdio/root", function() end)

local fs_handle = cworks.ipc_connect("system/file-system", function(text)
  cworks.send(io_handle, "File system response: " .. text)
end)

cworks.send(io_handle, "fs_handle: " .. fs_handle .. "\n")
cworks.send(fs_handle, "List?/")

for j = 1, 10, 1 do
  for i = 1, 10, 1 do
    cworks.sleep(0.1)
  end
  cworks.send(io_handle, "j = " .. j)
end

cworks.send(io_handle, "Exiting\n")
cworks.exit(0)
