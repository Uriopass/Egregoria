local cartest = require "cartest"

function Init()
    local r = 19.0
    for i = 0, 10 do
        cartest.add_car(vec2(5.5 * i, 0), nil, vec2(5.5 * i + 10, 0))
        cartest.add_car(vec2(0, 5 + 5.5 * i), nil, vec2(0.0, 5.5 * i - 10))
        cartest.add_car(vec2(5 + 5.5 * i, 60), nil, vec2(5.5 * i - 10, 60))
        cartest.add_car(vec2(60, 5.5 * i), nil, vec2(60, 5.5 * i + 10))
    end
end