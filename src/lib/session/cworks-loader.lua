local loaders = package.searchers or package.loaders

table.insert(loaders, 1, function(module_name)
  local module_path = "/usr/lib/" .. module_name:gsub("%.", "/") .. ".lua"

  local sc_req_json = "{\"Get\": \"" .. module_path .. "\"}"
  local sc_res_json = coroutine.yield(sc_req_json)
  print(sc_res_json)
  return nil

  --[[ local sc_res = json.parse(sc_res_json)
  local module_content = nil

  if sc_res and sc_res["FSGet"] ~= nil then
    module_content = sc_res["FSGet"]["String"]
  else
    print("Unknown data from kernel: " .. json.stringify(sc_res))
    return nil
  end

  if module_content then
    local chunk, err = load(module_content, module_path)
    if not chunk then
      error("Error loading module '" .. module_name .. "': " .. err)
    end
    return chunk
  else
    return "\n\tno module '" .. module_name .. "' in cworks"
  end ]]
end)
