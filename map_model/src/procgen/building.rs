use geom::{Polygon, Vec3};
use rand::Rng;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct RoofFace {
    pub poly: Polygon,
    pub normal: Vec3,
}

pub fn gen_exterior_workplace(size: f32) -> (Polygon, usize, Option<Vec<RoofFace>>) {
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
    (p, 3, None)
}

pub fn gen_exterior_house(size: f32) -> (Polygon, usize, Option<Vec<RoofFace>>) {
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
    (p, if seg == 0 { 1 } else { 0 }, None)
}

pub fn gen_exterior_supermarket(size: f32) -> (Polygon, usize, Option<Vec<RoofFace>>) {
    let mut h = rand_in(25.0, 30.0);
    let mut w = h + rand_in(5.0, 10.0);

    w *= size / 40.0;
    h *= size / 40.0;

    let mut p = Polygon::rect(w, h);

    p.translate(-p.barycenter());
    (p, 0, None)
}

fn rand_in(min: f32, max: f32) -> f32 {
    rand::thread_rng().gen_range(min, max)
}

fn randi_in(min: i32, max: i32) -> i32 {
    rand::thread_rng().gen_range(min, max)
}

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
