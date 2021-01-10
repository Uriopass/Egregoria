local cartest = require "cartest"
local seed = 100003417
local seed = 10

function Init()
    local r = 21.0
    local arrivalR = 21.0
    local N = 22
    for i = 0, N - 1 do
        local ang = 2.0 * math.pi * i / (N + 0.0)
        local x = r * math.cos(ang)
        local y = r * math.sin(ang)
        local p = vec2(x, y)

        local ang = 2.0 * math.pi * ((seed ^ i + 2) % (N + 1) - 1) / (N + 0.0)
        local x2 = arrivalR * math.cos(ang)
        local y2 = arrivalR * math.sin(ang)
        local p2 = vec2(x2, y2)

        cartest.add_car(p, vec2(-y, x), p2)
    end
end