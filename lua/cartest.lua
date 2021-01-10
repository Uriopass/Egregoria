--- @class cartest
local cartest = { cars = {} }

--- If dir is nil, the car is facing the objective
function cartest.add_car(pos, dir, obj)
    if dir == nil then
        dir = (obj - pos):normalize()
    end
    cartest.cars[#cartest.cars + 1] = { e = world:add_car(pos, dir, obj), obj = obj, arrived = false }
end

function Success()
    local ok = true
    for i, car in ipairs(cartest.cars) do
        if not car.arrived then
            car.arrived = world:pos(car.e):distance(car.obj) < 2.0
            if car.arrived then
                world:remove(car.e)
            end
            ok = false
        end
    end
    return ok
end

function Draw()
    for i, car in ipairs(cartest.cars) do
        if car.arrived then
            draw:color(color(0.0, 1.0, 0.0, 1.0))
        else
            draw:color(color(1.0, 0.0, 0.0, 1.0))
        end
        draw:circle(car.obj, 0.5)
    end
end

function Cleanup()
    for i, car in ipairs(cartest.cars) do
        if not car.arrived then
            world:remove(car.e)
        end
    end
end

up = vec2(0.0, 1.0)
down = vec2(0.0, -1.0)
left = vec2(-1.0, 0.0)
right = vec2(1.0, 0.0)

return cartest