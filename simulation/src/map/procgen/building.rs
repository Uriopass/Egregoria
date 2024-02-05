use geom::skeleton::{faces_from_skeleton, skeleton};
use geom::{minmax, vec2, Intersect, LinearColor, Polygon, Segment, Shape, Vec2, Vec3, AABB};
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};
use std::panic::catch_unwind;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ColoredMesh {
    pub faces: Vec<(Vec<Vec3>, LinearColor)>,
}

impl ColoredMesh {
    pub fn bbox(&self) -> AABB {
        let (ll, ur) = unwrap_or!(
            minmax(self.faces.iter().flat_map(|x| &x.0).map(|x| x.xy())),
            return AABB::zero()
        );
        AABB::new_ll_ur(ll, ur)
    }

    pub fn translate(&mut self, off: Vec2) {
        for (p, _) in &mut self.faces {
            for v in p {
                *v += off.z0();
            }
        }
    }
}

pub fn gen_exterior_house(size: f32, seed: u64) -> (ColoredMesh, Vec2) {
    let mut retry_cnt = 0;
    'retry: loop {
        let mut ri = 0.0;
        let realseed = ((retry_cnt << 32) + seed) as f32;
        let mut gen_range = |a, b| -> f32 {
            ri += 1.0;
            common::rand::rand2(realseed, ri) * (b - a) + a
        };

        retry_cnt += 1;

        let width = gen_range(15.0, 20.0);
        let height = gen_range(20.0, 28.0);

        let mut p = Polygon::rect(width, height);

        for _ in 0..gen_range(1.0, 5.0) as usize {
            let seg = gen_range(0.0, p.len() as f32) as usize;

            let origlen = p.segment(seg).vec().mag();
            if origlen < 8.0 {
                continue;
            }

            let l = gen_range(-0.2, 0.5);
            let r = gen_range(l + 0.4, l + 1.0);
            if r <= 1.0 {
                p.split_segment(seg, r);
            }

            let newlen = p.segment(seg).vec().mag();

            if l >= 0.0 {
                p.split_segment(seg, l * origlen / newlen);
                p.extrude(seg + 1, gen_range(1.0, 8.0));
            } else {
                p.extrude(seg, gen_range(1.0, 8.0));
            }

            p.simplify();
        }

        for x in p.iter_mut() {
            *x *= size / 40.0;
        }

        let c = p.bbox().center();

        for x in p.iter_mut() {
            *x -= c;
        }

        let merge_triangles = gen_range(0.0, 1.0) < 0.5;

        // silence panics
        let hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(|_| ()));
        // have to catch because the algorithm for skeleton might fail and is quite complicated
        let (skeleton, (faces, contour)) = unwrap_or!(
            catch_unwind(|| {
                let skeleton = skeleton(p.as_slice(), &[]);
                let faces = faces_from_skeleton(p.as_slice(), &skeleton, merge_triangles)?;
                Some((skeleton, faces))
            })
            .ok()
            .flatten(),
            {
                std::panic::set_hook(hook);
                continue 'retry;
            }
        );

        std::panic::set_hook(hook);

        if faces.len() < 2 {
            continue 'retry;
        }

        let segments = skeleton
            .iter()
            .flat_map(|x| x.sinks.iter().map(move |&dst| Segment::new(x.source, dst)));

        for mut x in segments.clone() {
            x.scale(0.99);
            for mut y in segments.clone() {
                y.scale(0.99);
                if x == y {
                    continue;
                }

                if x.intersects(&y) {
                    continue 'retry;
                }
            }
        }

        for s in &skeleton {
            if !p.contains(s.source) {
                continue 'retry;
            }
        }

        let lowest_segment = unwrap_or!(
            p.segments().min_by_key(|s| OrderedFloat(s.src.y + s.dst.y)),
            continue 'retry
        );

        let mut roofs = ColoredMesh::default();
        let roof_col = LinearColor::from(crate::colors().roof_col);

        let height = 4.0 + gen_range(0.0, 2.0);

        for mut face in faces {
            if face.len() < 3 {
                continue 'retry;
            }
            for v in &mut face {
                v.z += height;
            }
            roofs.faces.push((face, roof_col));
        }

        if contour.len() < 4 {
            continue 'retry;
        }

        let mut walls = Vec::with_capacity(contour.len());

        for (a, b, c) in geom::skeleton::window(&contour) {
            let ba = (a - b).normalize().xy();
            let bc = (c - b).normalize().xy();

            let mut d = (ba + bc).try_normalize().unwrap_or_default();

            if ba.perp_dot(bc) > 0.0 {
                d = -d;
            }

            if d.is_close(Vec2::ZERO, 0.1) {
                d = ba.perpendicular();
            }

            walls.push(b + d.z0() * 0.8 + Vec3::z(height));
        }

        for (&a, &b, _) in geom::skeleton::window(&walls) {
            let face = vec![a, b, b.xy().z0(), a.xy().z0()];
            roofs.faces.push((face, crate::colors().house_col.into()));
        }

        return (roofs, lowest_segment.middle());
    }
}

///  XXXXX   
///  XXXXX   
///    XXX   
///     |
pub fn gen_exterior_farm(size: f32, seed: u64) -> (ColoredMesh, Vec2) {
    let h_size = 30.0;
    let (mut mesh, mut door_pos) = gen_exterior_house(h_size, seed);

    let gen_range = |a, b| -> f32 { common::rand::rand(seed as f32 + 7.0) * (b - a) + a };

    let b = mesh.bbox();
    let off = -b.ll - Vec2::splat(size * 0.5) + vec2(gen_range(0.0, size - h_size), 3.0);
    mesh.translate(off);
    door_pos += off;

    (mesh, door_pos)
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
// 6. Put a outgoing door somewhere
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
