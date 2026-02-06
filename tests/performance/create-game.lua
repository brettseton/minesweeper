-- Stress test script for Minesweeper Game Creation
-- Usage: wrk -t2 -c10 -d30s -s tests/performance/create-game.lua http://localhost:8080/Game/new

-- Include an explicit XSRF token pair so unsafe requests work if you later extend
-- this script to POST/PUT/PATCH/DELETE endpoints.
function setup(thread)
   math.randomseed(os.time())
   thread:set("xsrf_token", tostring(math.random(100000000, 999999999)) .. tostring(os.time()))
end

local common_headers = {}

init = function(args)
   common_headers["X-XSRF-TOKEN"] = xsrf_token
   common_headers["Cookie"] = "XSRF-TOKEN=" .. xsrf_token
end

request = function()
   return wrk.format("POST", "/game/new", common_headers)
end

response = function(status, headers, body)
   if status ~= 200 then
      io.write("Request failed with status: " .. status .. "\n")
   end
end
