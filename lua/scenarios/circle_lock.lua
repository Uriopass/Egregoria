local cartest = require "cartest"

function Init()
    local r = 19.0
    for i = 0, 19 do
        local ang = 2.0 * math.pi * i / 20.0
        local x = r * math.cos(ang)
        local y = r * math.sin(ang)
        local p = vec2(x, y)
        local dir = vec2(-y, x)
        cartest.add_car(p, nil, p + dir)
    end
end