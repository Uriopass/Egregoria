local cartest = require "cartest"

function Init()
    cartest.add_car(vec2(0.0, 2.3), nil, vec2(0.0, 20.0))
    cartest.add_car(vec2(-3.1, 12.4), nil, vec2(10.0, 0.0))
    cartest.add_car(vec2(7.5, 9.6), nil, vec2(-10.0, 5.0))
end