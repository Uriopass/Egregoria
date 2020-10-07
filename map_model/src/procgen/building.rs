use geom::Polygon;

pub fn gen_exterior_workplace(size: f32) -> (Polygon, usize) {
    let a = rand_in(15.0, 20.0);
    let b = rand_in(15.0, 20.0);

    let width = f32::max(a, b) * (size / 40.0) * 1.5;
    let height = f32::min(a, b) * (size / 40.0);

    let mut p = Polygon::rect(width, height);
    let corn_coeff = rand_in(0.2, 0.3);

    p.split_segment(0, corn_coeff);
    p.split_segment(1, 1.0 - corn_coeff / (1.0 - corn_coeff));
    let extrude = rand_in(height * 0.3, height * 0.4);
    p.extrude(2, extrude);
    p.extrude(0, extrude);

    p.translate(-p.barycenter());
    (p, 3)
}

pub fn gen_exterior_house(size: f32) -> (Polygon, usize) {
    let a = rand_in(15.0, 20.0);
    let b = rand_in(15.0, 20.0);

    let width = f32::max(a, b) * (size / 40.0);
    let height = f32::min(a, b) * (size / 40.0);

    let mut p = Polygon::rect(width, height);
    let corn_coeff = rand_in(0.5, 0.75);
    let seg = rand_in(0.0, 3.99) as usize;

    p.split_segment(seg, corn_coeff);
    p.extrude(seg, rand_in(5.0, 10.0));

    p.translate(-p.barycenter());
    (p, if seg == 0 { 1 } else { 0 })
}

pub fn gen_exterior_supermarket(size: f32) -> (Polygon, usize) {
    let mut h = rand_in(25.0, 30.0);
    let mut w = h + rand_in(5.0, 10.0);

    w *= size / 40.0;
    h *= size / 40.0;

    let mut p = Polygon::rect(w, h);

    p.translate(-p.barycenter());
    (p, 0)
}

fn rand_in(min: f32, max: f32) -> f32 {
    min + rand::random::<f32>() * (max - min)
}
