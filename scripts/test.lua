function gen_sq()
    local w = rand_in(12.0, 22.0);
    local h = rand_in(10.0, 20.0);

    local p = poly_rect(w, h);
    p:translate(-p:barycenter());
    return p
end

function gen_exterior_corner()
    local a = rand_in(10.0, 20.0)
    local b = rand_in(10.0, 20.0)

    local w = math.max(a, b)
    local h = math.min(a, b)

    local p = poly_rect(w, h)

    local corn_coeff = rand_in(0.5, 0.75)

    local seg = math.floor(rand_in(0.0, 3.99))
    p:split_segment(seg, corn_coeff)
    p:extrude(seg, rand_in(5.0, 10.0))

    p:translate(-p:barycenter());
    return p
end
    
return gen_exterior_corner()