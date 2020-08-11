local cartest = require "cartest"

function Init()
    cartest.add_car(vec2(-5.0, 0.0), right, vec2(10.0, 0.0))
    cartest.add_car(vec2(5.0, 11.0), down, vec2(5.0, -5.0))
end