local cartest = require "cartest"

function Init()
    cartest.add_car(vec2(-10.0, 0.0), right, vec2(10.0, 0.0))
    cartest.add_car(vec2(-15.0, 0.0), right, vec2(0.0, 0.0))
    cartest.add_car(vec2(-20.0, 0.0), right, vec2(-5.0, 0.0))
    cartest.add_car(vec2(-25.0, 0.0), right, vec2(-10.0, 0.0))
    cartest.add_car(vec2(-30.0, 0.0), right, vec2(-15.0, 0.0))
    cartest.add_car(vec2(-35.0, 0.0), right, vec2(-20.0, 0.0))
    cartest.add_car(vec2(-40.0, 0.0), right, vec2(-25.0, 0.0))
end