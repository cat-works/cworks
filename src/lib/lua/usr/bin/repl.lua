local cworks = env.cworks

io.write("Entering REPL mode. Type 'exit' to leave.\n")
-- 1 ==> 1
-- print ==> print
while true do
  io.write("\x1b[2m>>\x1b[m ")
  local repl_line = io.read("*l")
  if repl_line == "exit" then
    io.write("Exiting REPL mode.\n")
    break
  end
  local func, err = load(repl_line)
  if not func then
    io.write("Error: " .. err .. "\n")
  else
    local success, result = pcall(func)
    if not success then
      io.write("Error: " .. result .. "\n")
    else
      if result ~= nil then
        if type(result) == "table" then
          io.write("Result: " .. json.stringify(result) .. "\n")
        else
          io.write("Result: " .. tostring(result) .. "\n")
        end
      end
    end
  end
end


cworks.exit()
