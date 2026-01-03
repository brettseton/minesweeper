-- Stress test script for Minesweeper Game Creation
-- Usage: wrk -t2 -c10 -d30s -s tests/performance/create-game.lua http://localhost:8080/Game/new

request = function()
   return wrk.format("GET", "/game/new")
end

response = function(status, headers, body)
   if status ~= 200 then
      io.write("Request failed with status: " .. status .. "\n")
   end
end

