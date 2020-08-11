function init()
    x = rand_in(0.0, 50.0)
    y = rand_in(0.0, 50.0)
    start = vec2(x, y)
    car = world:add_car(start)
    arrival = vec2(start:x(), start:y())
end

function draw()
    
end

function success()
    return (arrival - world:pos(car)):magnitude() < 0.1
end