function init()
    local start = vec2(0.0, 0.0)
    arrival = start + vec2(15.0, 0.0)
    car = world:add_car(start, arrival)
end

function paint()
    draw:color(color(1.0, 0.0, 0.0, 1.0))
    draw:circle(arrival, 0.5)
end

function success()
    return (arrival - world:pos(car)):magnitude() < 1.5
end