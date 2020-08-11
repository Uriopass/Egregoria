local cartest = require "cartest"

function Init()
    cartest.add_car(vec2(-10.0, 0.0), right, vec2(10.0, 0.0))
    cartest.add_car(vec2(10.0, 0.0), left, vec2(-10.0, 0.0))
end