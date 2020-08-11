--- @class cartest
local cartest = { cars = {} }

--- If dir is nil, the car is facing the objective
function cartest.add_car(pos, dir, obj)
    if dir == nil then
        dir = (obj - pos):normalize()
    end
    cartest.cars[#cartest.cars + 1] = { e = world:add_car(pos, dir, obj), obj = obj }
end

function Success()
    local ok = true
    for i, car in ipairs(cartest.cars) do
        local arrived = world:pos(car.e):distance(car.obj) < 1.5
        ok = ok and arrived
    end
    return ok
end

function Draw()
    for i, car in ipairs(cartest.cars) do
        local arrived = world:pos(car.e):distance(car.obj) < 1.5
        if arrived then
            draw:color(color(0.0, 1.0, 0.0, 1.0))
        else
            draw:color(color(1.0, 0.0, 0.0, 1.0))
        end
        draw:circle(car.obj, 0.5)
    end
end

function Cleanup()
    for i, car in ipairs(cartest.cars) do
        world:remove(car.e)
    end
end

up = vec2(0.0, 1.0)
down = vec2(0.0, -1.0)
left = vec2(-1.0, 0.0)
right = vec2(1.0, 0.0)

return cartest