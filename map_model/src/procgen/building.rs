use geom::skeleton::{faces_from_skeleton, skeleton};
use geom::{vec2, vec3, Color, LinearColor, Polygon, Vec2, Vec3};
use ordered_float::OrderedFloat;
use rand::Rng;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct RoofFace {
    pub poly: Polygon,
    pub normal: Vec3,
}

pub fn gen_exterior_workplace(size: f32) -> (Vec<(Polygon, LinearColor)>, Vec2) {
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
    (vec![(p, Color::new(0.48, 0.48, 0.5, 1.0).into())], door_pos)
}

pub fn gen_exterior_house_new(size: f32) -> (Vec<(Polygon, LinearColor)>, Vec2) {
    let width = rand_in(10.0, 15.0);
    let height = rand_in(15.0, 20.0);

    let mut p = Polygon::rect(width, height);

    let seg = rand_in(0.0, 4.0) as usize;

    let corn_coeff = rand_in(0.5, 0.75);
    p.split_segment(seg, corn_coeff);
    p.extrude(seg, rand_in(5.0, 10.0));

    p.iter_mut().for_each(|x| *x *= size / 40.0);

    let skeleton = skeleton(p.as_slice(), &[]);

    let faces = faces_from_skeleton(p.as_slice(), &skeleton);

    let mut roofs = vec![];
    let roof_col = common::config().roof_col;

    for face in faces.into_iter().rev() {
        let normal = (face[0] - face[1]).cross(face[2] - face[1]).normalize();

        let luminosity = 0.8 + 0.2 * vec3(0.7, 0.3, 0.5).normalize().dot(normal);
        let col = luminosity * LinearColor::from(roof_col);
        roofs.push((Polygon(face.into_iter().map(|x| x.xy()).collect()), col));
    }

    let lowest_segment = p
        .segments()
        .min_by_key(|s| OrderedFloat(s.src.y + s.dst.y))
        .unwrap();

    (roofs, lowest_segment.middle())
}

pub fn gen_exterior_supermarket(size: f32) -> (Vec<(Polygon, LinearColor)>, Vec2) {
    let mut h = rand_in(25.0, 30.0);
    let mut w = h + rand_in(5.0, 10.0);

    w *= size / 40.0;
    h *= size / 40.0;

    let mut p = Polygon::rect(w, h);

    let mut door_pos = vec2(w * 0.5, 0.0);
    let off = -p.barycenter();

    door_pos += off;
    p.translate(off);

    (vec![(p, Color::new(0.52, 0.5, 0.50, 1.0).into())], door_pos)
}

///  -------------------
///  -------------------
///  -------------------
///  -------------------
///  -------------------
///           
///  XXXXX   
///  XXXXX   
///    XXX   
///     |    
pub fn gen_exterior_farm(size: f32) -> (Vec<(Polygon, LinearColor)>, Vec2) {
    let h_size = 30.0;
    let (mut polys, mut door_pos) = gen_exterior_house_new(h_size);

    let mut off = Vec2::splat(h_size * 0.5 - size * 0.5);
    off.x += rand_in(0.0, size - h_size);
    for p in &mut polys {
        p.0.translate(off);
    }
    door_pos += off;

    polys.splice(
        0..0,
        vec![(
            Polygon::centered_rect(size, size),
            Color::new(0.75, 0.60, 0.35, 1.0).into(),
        )],
    );

    for i in -1..5 {
        let mut p = Polygon::centered_rect(size - 5.0, 3.0);
        p.translate(vec2(0.0, i as f32 * 8.5));
        polys.push((p, Color::new(0.62, 0.5, 0.29, 1.0).into()))
    }

    (polys, door_pos)
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
