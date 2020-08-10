function init()
    x = rand_in(0.0, 50.0)
    y = rand_in(0.0, 50.0)
    start = vec2(x, y)
    car = state:add_car(start)
    arrival = vec2(start:x(), start:y())
end

function success()
    return (arrival - state:pos(car)):magnitude() < 0.1
end