local cworks = require("cworks");

local io_handle = cworks.ipc_connect("system/stdio/root")
cworks.send(io_handle, "Hello from test_proc.lua!")

local fs_handle = cworks.ipc_connect("system/file-system")
cworks.send(fs_handle, "List?/")

cworks.pending()
cworks.pending()
cworks.pending()

cworks.exit(0)
