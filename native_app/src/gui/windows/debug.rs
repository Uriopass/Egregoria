#![allow(clippy::type_complexity)]

use crate::gui::InspectedEntity;
use common::{GameTime, SECONDS_PER_DAY};
use egregoria::engine_interaction::{MouseInfo, RenderStats};
use egregoria::map_dynamic::Itinerary;
use egregoria::physics::CollisionWorld;
use egregoria::rendering::immediate::ImmediateDraw;
use egregoria::utils::frame_log::FrameLog;
use egregoria::Egregoria;
use geom::{vec2, Camera, Color, Intersect, LinearColor, Segment, Spline, Vec2, AABB, OBB};
use imgui::im_str;
use imgui::Ui;
use map_model::{Map, RoadSegmentKind};
use wgpu_engine::Tesselator;

pub struct DebugObjs(
    pub  Vec<(
        bool,
        &'static str,
        fn(&mut Tesselator, &Egregoria) -> Option<()>,
    )>,
);

impl Default for DebugObjs {
    fn default() -> Self {
        DebugObjs(vec![
            (true, "Debug pathfinder", debug_pathfinder),
            (false, "Debug spatialmap", debug_spatialmap),
            (false, "Debug collision world", debug_coworld),
            (false, "Debug OBBs", debug_obb),
            (false, "Debug rays", debug_rays),
            (false, "Debug splines", debug_spline),
            (false, "Debug turns", debug_turns),
            (false, "Debug road points", debug_road_points),
            (false, "Show grid", show_grid),
        ])
    }
}

pub fn debug(ui: &Ui, goria: &mut Egregoria) {
    let mut objs = goria.write::<DebugObjs>();
    for (val, name, _) in &mut objs.0 {
        ui.checkbox(&im_str!("{}", *name), val);
    }
    drop(objs);

    if ui.small_button(&im_str!("build one house")) {
        let house = map_model::procgen::gen_exterior_house_new(40.0);

        for (mut p, col) in house.0 {
            for x in p.iter_mut() {
                *x += goria.read::<Camera>().position.xy();
            }

            goria
                .write::<ImmediateDraw>()
                .polygon(p)
                .color(col)
                .persistent();
        }
    }

    let time = goria.read::<GameTime>().timestamp;
    let daysecleft = SECONDS_PER_DAY - goria.read::<GameTime>().daytime.daysec();

    if ui.small_button(&im_str!("set night")) {
        *goria.write::<GameTime>() = GameTime::new(0.1, time + daysecleft as f64);
    }

    if ui.small_button(&im_str!("set morning")) {
        *goria.write::<GameTime>() =
            GameTime::new(0.1, time + daysecleft as f64 + 7.0 * GameTime::HOUR as f64);
    }

    if ui.small_button(&im_str!("set day")) {
        *goria.write::<GameTime>() =
            GameTime::new(0.1, time + daysecleft as f64 + 12.0 * GameTime::HOUR as f64);
    }

    if ui.small_button(&im_str!("set dawn")) {
        *goria.write::<GameTime>() =
            GameTime::new(0.1, time + daysecleft as f64 + 18.0 * GameTime::HOUR as f64);
    }

    let stats = goria.read::<RenderStats>();
    let mouse = goria.read::<MouseInfo>().unprojected;
    let cam = goria.read::<Camera>().position;

    ui.text("Averaged over last 10 frames: ");
    ui.text(im_str!("Total time: {:.1}ms", stats.all.avg() * 1000.0));
    ui.text(im_str!(
        "World update time: {:.1}ms",
        stats.world_update.avg() * 1000.0
    ));
    ui.text(im_str!("Render time: {:.1}ms", stats.render.avg() * 1000.0));
    ui.text(im_str!("Mouse pos: {:.1} {:.1}", mouse.x, mouse.y));
    ui.text(im_str!("Cam   pos: {:.1} {:.1} {:.1}", cam.x, cam.y, cam.z));
    ui.separator();
    ui.text("Frame log");
    let flog = goria.read::<FrameLog>();
    {
        let fl = flog.get_frame_log();
        for s in &*fl {
            ui.text(im_str!("{}", s));
        }
    }
    flog.clear();
}

pub fn show_grid(tess: &mut Tesselator, state: &Egregoria) -> Option<()> {
    let cam = &*state.read::<Camera>();

    if cam.position.z > 1000.0 {
        return Some(());
    }

    let gray_maj = 0.5;
    let gray_min = 0.3;
    if cam.position.z < 300.0 {
        tess.set_color(Color::new(gray_min, gray_min, gray_min, 0.5));
        tess.draw_grid(1.0);
    }
    tess.set_color(Color::new(gray_maj, gray_maj, gray_maj, 0.5));
    tess.draw_grid(10.0);
    Some(())
}

pub fn debug_spline(tess: &mut Tesselator, world: &Egregoria) -> Option<()> {
    for road in world.read::<Map>().roads().values() {
        if let RoadSegmentKind::Curved((fr_dr, to_der)) = road.segment {
            let fr = road.src_point;
            let to = road.dst_point;
            draw_spline(
                tess,
                &Spline {
                    from: fr,
                    to,
                    from_derivative: fr_dr,
                    to_derivative: to_der,
                },
            );
        }
    }

    Some(())
}

pub fn debug_road_points(tess: &mut Tesselator, world: &Egregoria) -> Option<()> {
    let map = world.read::<Map>();
    tess.set_color(Color::RED);
    for (_, road) in map.roads() {
        tess.draw_polyline(road.generated_points().as_slice(), 1.0, 0.1);
    }
    Some(())
}

pub fn debug_turns(tess: &mut Tesselator, world: &Egregoria) -> Option<()> {
    let map = world.read::<Map>();
    let lanes = map.lanes();
    tess.set_color(LinearColor::RED);
    for inter in map.intersections().values() {
        for turn in inter.turns() {
            let p = unwrap_or!(turn.points.get(turn.points.n_points() / 2), continue);
            let r = common::rand::rand2(p.x, p.y);
            tess.set_color(Color::hsv(r * 360.0, 0.8, 0.6, 0.5));

            tess.draw_polyline_with_dir(
                turn.points.as_slice(),
                -lanes[turn.id.src].orientation_from(inter.id),
                lanes[turn.id.dst].orientation_from(inter.id),
                1.0,
                1.0,
            );
        }
    }

    Some(())
}

fn draw_spline(tess: &mut Tesselator, sp: &Spline) {
    tess.set_color(Color::RED);
    tess.draw_polyline(
        &sp.smart_points(0.1, 0.0, 1.0).collect::<Vec<_>>(),
        1.0,
        2.0,
    );
    tess.set_color(Color::GREEN);

    tess.draw_stroke(sp.from, sp.from + sp.from_derivative, 1.0, 1.5);
    tess.draw_stroke(sp.to, sp.to + sp.to_derivative, 1.0, 1.5);

    tess.set_color(Color::PURPLE);
    tess.draw_circle(sp.from, 1.0, 1.0);
    tess.draw_circle(sp.to, 1.0, 1.0);

    tess.draw_circle(sp.from + sp.from_derivative, 1.0, 1.0);
    tess.draw_circle(sp.to + sp.to_derivative, 1.0, 1.0);
}

fn debug_coworld(tess: &mut Tesselator, world: &Egregoria) -> Option<()> {
    let coworld = world.read::<CollisionWorld>();

    tess.set_color(Color::new(0.8, 0.8, 0.9, 0.5));
    for h in coworld.handles() {
        let pos = coworld.get(h).unwrap().0;
        tess.draw_circle(pos, 1.0, 3.0);
    }
    Some(())
}

pub fn debug_obb(tess: &mut Tesselator, world: &Egregoria) -> Option<()> {
    let time = world.read::<GameTime>();
    let mouse = world.read::<MouseInfo>().unprojected;

    let time = time.timestamp * 0.2;
    let c = time.cos() as f32;
    let s = time.sin() as f32;

    let obb1 = OBB::new(Vec2::ZERO, vec2(c, s), 10.0, 5.0);

    let obbm = OBB::new(
        mouse,
        vec2((time * 3.0).cos() as f32, (time * 3.0).sin() as f32),
        8.0,
        6.0,
    );

    let seg = Segment::new(vec2(0.0, 10.0), vec2(18.0, 14.0));

    let mut color = if obb1.intersects(&obbm) {
        LinearColor::RED
    } else {
        LinearColor::BLUE
    };

    if obbm.intersects(&seg) {
        color = LinearColor::WHITE
    }

    if obb1.contains(mouse) {
        color = LinearColor::CYAN
    }

    let axis = obbm.axis();
    let w = axis[0].magnitude();
    let h = axis[1].magnitude();
    let tr = Segment {
        src: (seg.src - obbm.corners[0]).rotated_by(axis[0].flipy()),
        dst: (seg.dst - obbm.corners[0]).rotated_by(axis[0].flipy()),
    };
    let aabb = AABB::new(Vec2::ZERO, vec2(w * w, h * w));

    color.a = 0.5;

    tess.set_color(color);
    tess.draw_filled_polygon(&obb1.corners, 0.99);
    tess.draw_filled_polygon(&obbm.corners, 0.99);

    tess.set_color(LinearColor::gray(0.8));
    tess.draw_line(seg.src, seg.dst, 0.99);
    tess.set_color(LinearColor::gray(0.9));

    tess.color = LinearColor::WHITE;
    tess.color.a = if aabb.intersects(&tr) { 0.4 } else { 0.2 };

    tess.draw_line(tr.src, tr.dst, 0.99);
    tess.draw_rect_cos_sin(aabb.center(), 1.0, aabb.w(), aabb.h(), Vec2::UNIT_X);

    Some(())
}

pub fn debug_pathfinder(tess: &mut Tesselator, world: &Egregoria) -> Option<()> {
    let map: &Map = &world.read::<Map>();
    let selected = world.read::<InspectedEntity>().e?;
    let pos = world.pos(selected)?;

    let itinerary = world.comp::<Itinerary>(selected)?;

    tess.set_color(LinearColor::GREEN);
    tess.draw_polyline(&itinerary.local_path(), 1.0, 1.0);

    if let Some(p) = itinerary.get_point() {
        tess.draw_stroke(p, pos, 1.0, 1.0);
    }

    if let egregoria::map_dynamic::ItineraryKind::Route(r) = itinerary.kind() {
        tess.set_color(LinearColor::RED);
        for l in &r.reversed_route {
            if let Some(l) = l.raw_points(map) {
                tess.draw_polyline(l.as_slice(), 1.0, 3.0);
            }
        }
        tess.set_color(if itinerary.has_ended(0.0) {
            LinearColor::GREEN
        } else {
            LinearColor::MAGENTA
        });

        tess.draw_circle(r.end_pos, 1.0, 1.0);
    }
    Some(())
}

pub fn debug_rays(tess: &mut Tesselator, world: &Egregoria) -> Option<()> {
    let time = world.read::<GameTime>();
    let time = time.timestamp * 0.2;
    let c = time.cos() as f32;
    let s = time.sin() as f32;
    let mouse = world.read::<MouseInfo>().unprojected;

    let r = geom::Ray {
        from: 10.0 * vec2(c, s),
        dir: vec2(
            (time * 2.3 + 1.0).cos() as f32,
            (time * 2.3 + 1.0).sin() as f32,
        ),
    };

    let r2 = geom::Ray {
        from: mouse,
        dir: vec2((time * 3.0).cos() as f32, (time * 3.0).sin() as f32),
    };

    tess.set_color(LinearColor::WHITE);
    tess.draw_line(r.from, r.from + r.dir * 50.0, 0.5);
    tess.draw_line(r2.from, r2.from + r2.dir * 50.0, 0.5);

    let inter = r.intersection_point(&r2);
    if let Some(v) = inter {
        tess.set_color(LinearColor::RED);

        tess.draw_circle(v, 0.5, 2.0);
    }

    Some(())
}

pub fn debug_spatialmap(tess: &mut Tesselator, world: &Egregoria) -> Option<()> {
    let map: &Map = &world.read::<Map>();
    for r in map.spatial_map().debug_grid() {
        tess.set_color(LinearColor {
            a: 0.1,
            ..LinearColor::BLUE
        });
        tess.draw_rect_cos_sin(r.center(), 1.0, r.w(), r.h(), Vec2::UNIT_X);
    }

    Some(())
}
