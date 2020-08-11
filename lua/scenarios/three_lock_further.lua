local cartest = require "cartest"

function Init()
    cartest.add_car(vec2(0.0, 0.0), nil, vec2(0.0, 20.0))
    cartest.add_car(vec2(-3.5, 13.4), nil, vec2(10.0, 0.0))
    cartest.add_car(vec2(8.5, 10.6), nil, vec2(-10.0, 5.0))
end