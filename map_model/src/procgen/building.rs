use geom::{vec2, vec3, Polygon, Vec2, Vec3};
use rand::Rng;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct RoofFace {
    pub poly: Polygon,
    pub normal: Vec3,
}

pub fn gen_exterior_workplace(size: f32) -> (Polygon, Vec2, Option<Vec<RoofFace>>) {
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
    let door_pos = (p[3] + p[4]) * 0.5;
    (p, door_pos, None)
}

///
/// Generates the exterior of a house with the roofs too
///
///   6       5
///   |----a----|
///   |    |    |
///   | r4 | r3 | 4     3
///   |    |    |-------v
/// h |    |  /    r2   |
///   |    b------------c
///   |  /    r1        |
///   |---------.-------|
///   0         1      2
///            w
///
pub fn gen_exterior_house(size: f32) -> (Polygon, Vec2, Option<Vec<RoofFace>>) {
    let width = rand_in(10.0, 15.0) * (size / 40.0);
    let height = rand_in(15.0, 20.0) * (size / 40.0);

    let mut p = Polygon::rect(width, height);
    let corn_coeff = rand_in(0.5, 0.75);
    p.split_segment(1, corn_coeff);
    p.extrude(1, rand_in(5.0, 10.0));

    let a = vec2(width * 0.5, height);
    let c = (p[3] + p[2]) / 2.0;
    let b = vec2(a.x, c.y);

    let r1 = Polygon(vec![p[0], p[2], c, b]);
    let r2 = Polygon(vec![b, c, p[3], p[4]]);
    let r3 = Polygon(vec![b, p[4], p[5], a]);
    let r4 = Polygon(vec![p[0], b, a, p[6]]);

    let h = 0.5;

    let mut roofs = vec![
        RoofFace {
            poly: r1,
            normal: vec3(0.0, -1.0, h).normalize(),
        },
        RoofFace {
            poly: r2,
            normal: vec3(0.0, 1.0, h).normalize(),
        },
        RoofFace {
            poly: r3,
            normal: vec3(1.0, 0.0, h).normalize(),
        },
        RoofFace {
            poly: r4,
            normal: vec3(-1.0, 0.0, h).normalize(),
        },
    ];

    let mut door_pos = vec2(width * 0.5, 0.0);
    let off = -p.barycenter();

    door_pos += off;
    p.translate(off);

    let mut r = vec2(1.0, 0.0);
    if rand_in(0.0, 1.0) < 0.5 {
        r = r.perpendicular();
    }
    if rand_in(0.0, 1.0) < 0.5 {
        r = -r;
    }

    p.rotate(r);

    for roof in roofs.iter_mut() {
        roof.poly.translate(off);
        roof.poly.rotate(r);
        roof.normal = roof.normal.rotate_z(r);
    }

    (p, door_pos, Some(roofs))
}

pub fn gen_exterior_supermarket(size: f32) -> (Polygon, Vec2, Option<Vec<RoofFace>>) {
    let mut h = rand_in(25.0, 30.0);
    let mut w = h + rand_in(5.0, 10.0);

    w *= size / 40.0;
    h *= size / 40.0;

    let mut p = Polygon::rect(w, h);

    let mut door_pos = vec2(w * 0.5, 0.0);
    let off = -p.barycenter();

    door_pos += off;
    p.translate(off);

    (p, door_pos, None)
}

fn rand_in(min: f32, max: f32) -> f32 {
    rand::thread_rng().gen_range(min, max)
}

/*
fn randi_in(min: i32, max: i32) -> i32 {
    rand::thread_rng().gen_range(min, max)
}
 */

// How to gen a house
// Idea: Make everything out of rectangles
// 1. Make exterior
//    - pick random rectangle
//    - add random rectangle along this rectangle (or not)
//    - add random rectangle along this rectangle (or not)
// 3. Merge the rectangles in one shape
// 3. Recursively split the shape horizontally and vertically
// 4. Score the resulting house based on "rectanglicity" and size of resulting regions
//    - rectanglicity: area of region divided by area of smallest surrounding bbox
// 5. Put holes in between regions for the doors
// 6. Put a outgoing door somwhere
// 7. Assign rooms somehow
//  necessary:
//    - bedroom
//    - kitchen
//    - toilets
//  optional:
//    - dining room
//    - office
//    - playroom
// 8. Score the room assignment based on some rules: kitchen next to bedrooms, small toilet and big bedroom etc

/*
const SIZE: usize = 200; // 20 meters

type Idx = (usize, usize);

struct HGrid([[u8; SIZE]; SIZE]);

struct GeneratedHouse {
    exterior: Polygon,
    //    rooms: Vec<(RoomType, Polygon)>,
    //    walls: Vec<>
}

impl HGrid {
    fn v(&self, pos: Idx) -> u8 {
        self.0[pos.1][pos.0]
    }

    fn add_rectangle(&mut self, near: Idx) {
        let w = randi_in(10, 50);
    }
}
*/
//fn gen_house
