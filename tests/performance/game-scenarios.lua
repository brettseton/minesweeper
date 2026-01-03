-- Stress test script for Minesweeper Game with multiple routes
-- Usage: wrk -t2 -c10 -d30s -s tests/performance/game-scenarios.lua http://localhost:4533

local game_ids = {}
local counter = 0

-- Mock headers for potential auth bypass in test environments
local common_headers = {
   ["Content-Type"] = "application/json"
}

request = function()
   counter = counter + 1
   local r = math.random(1, 100)
   
   -- 5% chance to check account status
   if r <= 5 then
      return wrk.format("GET", "/account/status", common_headers)
   -- 5% chance to create a CUSTOM new game (15x15 with 30 mines)
   elseif r <= 20 then
      return wrk.format("GET", "/game/new/15/15/30", common_headers)
   -- 10% chance to create a standard new game
   elseif r <= 30 or #game_ids == 0 then
      return wrk.format("GET", "/game/new", common_headers)
   -- 40% chance to get an existing game state
   elseif r <= 70 then
      local id = game_ids[math.random(#game_ids)]
      return wrk.format("GET", "/game/" .. id, common_headers)
   -- 20% chance to make a move
   elseif r <= 90 then
      local id = game_ids[math.random(#game_ids)]
      local x = math.random(0, 9)
      local y = math.random(0, 9)
      local body = '{"x":' .. x .. ',"y":' .. y .. '}'
      return wrk.format("POST", "/game/" .. id, common_headers, body)
   -- 10% chance to toggle a flag
   else
      local id = game_ids[math.random(#game_ids)]
      local x = math.random(0, 9)
      local y = math.random(0, 9)
      local body = '{"x":' .. x .. ',"y":' .. y .. '}'
      return wrk.format("POST", "/game/flag/" .. id, common_headers, body)
   end
end
response = function(status, headers, body)
   if status == 200 then
      -- Simple pattern matching to extract "id":123 from JSON response
      local id = body:match("\"id\":%s*(%d+)")
      if id then
         -- Keep a small pool of active game IDs per thread to avoid memory bloat
         if #game_ids < 100 then
            table.insert(game_ids, id)
         else
            game_ids[math.random(#game_ids)] = id
         end
      end
   elseif status ~= 200 and status ~= 302 then
      io.write("Request failed with status: " .. status .. "\n")
   end
end
