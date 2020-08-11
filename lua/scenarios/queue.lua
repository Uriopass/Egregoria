local cartest = require "cartest"

function Init()
    cartest.add_car(vec2(-10.0, 0.0), right, vec2(10.0, 0.0))
    cartest.add_car(vec2(-15.0, 0.0), right, vec2(0.0, 0.0))
    cartest.add_car(vec2(-20.0, 0.0), right, vec2(-10.0, 0.0))
end