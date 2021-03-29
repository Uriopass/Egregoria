#![allow(clippy::type_complexity)]

use crate::game_loop::Timings;
use crate::gui::InspectedEntity;
use crate::input::MouseInfo;
use crate::uiworld::UiWorld;
use common::{GameTime, SECONDS_PER_DAY};
use common::{Z_DEBUG, Z_DEBUG_BG};
use egregoria::map_dynamic::{Itinerary, ParkingManagement};
use egregoria::physics::CollisionWorld;
use egregoria::Egregoria;
use geom::{vec2, Camera, Color, Intersect, LinearColor, Segment, Spline, Vec2, AABB, OBB};
use imgui::im_str;
use imgui::Ui;
use map_model::{Map, RoadSegmentKind};
use wgpu_engine::Tesselator;

register_resource_noserialize!(DebugObjs);
pub struct DebugObjs(
    pub  Vec<(
        bool,
        &'static str,
        fn(&mut Tesselator, &Egregoria, &UiWorld) -> Option<()>,
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
            (false, "Debug lots", debug_lots),
            (false, "Debug road points", debug_road_points),
            (false, "Debug parking", debug_parking),
            (false, "Show grid", show_grid),
        ])
    }
}

pub fn debug(window: imgui::Window, ui: &Ui, uiworld: &mut UiWorld, goria: &Egregoria) {
    window.build(ui, || {
        let mut objs = uiworld.write::<DebugObjs>();
        for (val, name, _) in &mut objs.0 {
            ui.checkbox(&im_str!("{}", *name), val);
        }
        drop(objs);

        let time = goria.read::<GameTime>().timestamp;
        let daysecleft = SECONDS_PER_DAY - goria.read::<GameTime>().daytime.daysec();

        if ui.small_button(im_str!("set night")) {
            uiworld
                .commands()
                .set_game_time(GameTime::new(0.1, time + daysecleft as f64));
        }

        if ui.small_button(im_str!("set morning")) {
            uiworld.commands().set_game_time(GameTime::new(
                0.1,
                time + daysecleft as f64 + 7.0 * GameTime::HOUR as f64,
            ));
        }

        if ui.small_button(im_str!("set day")) {
            uiworld.commands().set_game_time(GameTime::new(
                0.1,
                time + daysecleft as f64 + 12.0 * GameTime::HOUR as f64,
            ));
        }

        if ui.small_button(im_str!("set dawn")) {
            uiworld.commands().set_game_time(GameTime::new(
                0.1,
                time + daysecleft as f64 + 18.0 * GameTime::HOUR as f64,
            ));
        }

        let timings = uiworld.read::<Timings>();
        let mouse = uiworld.read::<MouseInfo>().unprojected;
        let cam = uiworld.read::<Camera>().position;

        ui.text("Averaged over last 10 frames: ");
        ui.text(im_str!("Total time: {:.1}ms", timings.all.avg() * 1000.0));
        ui.text(im_str!(
            "World update time: {:.1}ms",
            timings.world_update.avg() * 1000.0
        ));
        ui.text(im_str!(
            "Render prepare time: {:.1}ms",
            timings.render.avg() * 1000.0
        ));
        ui.text(im_str!("Mouse pos: {:.1} {:.1}", mouse.x, mouse.y));
        ui.text(im_str!("Cam   pos: {:.1} {:.1} {:.1}", cam.x, cam.y, cam.z));
        ui.separator();
        ui.text("Game system times");

        ui.columns(2, im_str!("game times"), false);
        ui.text("System name");
        ui.next_column();
        ui.text("Time avg in ms over last 100 ticks");
        ui.next_column();

        for (name, time) in &timings.per_game_system {
            ui.text(name);
            ui.next_column();
            ui.text(im_str!("{:.3}", *time));
            ui.next_column();
        }
    })
}

pub fn show_grid(tess: &mut Tesselator, _: &Egregoria, uiworld: &UiWorld) -> Option<()> {
    let cam = &*uiworld.read::<Camera>();

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

pub fn debug_spline(tess: &mut Tesselator, goria: &Egregoria, _: &UiWorld) -> Option<()> {
    for road in goria.read::<Map>().roads().values() {
        if let RoadSegmentKind::Curved((fr_dr, to_der)) = road.segment {
            let fr = road.points.first();
            let to = road.points.last();
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

pub fn debug_lots(tess: &mut Tesselator, goria: &Egregoria, _: &UiWorld) -> Option<()> {
    tess.set_color(Color::RED);
    for lot in goria.read::<Map>().lots().values() {
        tess.draw_circle(lot.shape.corners[0], Z_DEBUG, 1.0);
    }

    Some(())
}

pub fn debug_road_points(tess: &mut Tesselator, goria: &Egregoria, _: &UiWorld) -> Option<()> {
    let map = goria.read::<Map>();
    tess.set_color(Color::RED);
    for (_, road) in map.roads() {
        tess.draw_polyline(road.points().as_slice(), Z_DEBUG, 0.3);
    }

    for (_, lane) in map.lanes() {
        let r = common::rand::rand2(lane.points.first().x, lane.points.first().y);
        tess.set_color(Color::hsv(r * 360.0, 0.8, 0.6, 0.5));

        tess.draw_polyline(lane.points.as_slice(), Z_DEBUG, 0.3);
    }
    Some(())
}

fn draw_spline(tess: &mut Tesselator, sp: &Spline) {
    tess.set_color(Color::RED);
    tess.draw_polyline(
        &sp.smart_points(0.1, 0.0, 1.0).collect::<Vec<_>>(),
        Z_DEBUG,
        2.0,
    );
    tess.set_color(Color::GREEN);

    tess.draw_stroke(sp.from, sp.from + sp.from_derivative, Z_DEBUG, 1.5);
    tess.draw_stroke(sp.to, sp.to + sp.to_derivative, Z_DEBUG, 1.5);

    tess.set_color(Color::PURPLE);
    tess.draw_circle(sp.from, Z_DEBUG, 1.0);
    tess.draw_circle(sp.to, Z_DEBUG, 1.0);

    tess.draw_circle(sp.from + sp.from_derivative, Z_DEBUG, 1.0);
    tess.draw_circle(sp.to + sp.to_derivative, Z_DEBUG, 1.0);
}

fn debug_coworld(tess: &mut Tesselator, goria: &Egregoria, _: &UiWorld) -> Option<()> {
    let coworld = goria.read::<CollisionWorld>();

    tess.set_color(Color::new(0.8, 0.8, 0.9, 0.5));
    for h in coworld.handles() {
        let pos = coworld.get(h)?.0;
        tess.draw_circle(pos, Z_DEBUG, 3.0);
    }
    Some(())
}

pub fn debug_obb(tess: &mut Tesselator, goria: &Egregoria, uiworld: &UiWorld) -> Option<()> {
    let time = goria.read::<GameTime>();
    let mouse = uiworld.read::<MouseInfo>().unprojected;

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
    tess.draw_filled_polygon(&obb1.corners, Z_DEBUG_BG);
    tess.draw_filled_polygon(&obbm.corners, Z_DEBUG_BG);

    tess.set_color(LinearColor::gray(0.8));
    tess.draw_line(seg.src, seg.dst, Z_DEBUG_BG);
    tess.set_color(LinearColor::gray(0.9));

    tess.color = LinearColor::WHITE;
    tess.color.a = if aabb.intersects(&tr) { 0.4 } else { 0.2 };

    tess.draw_line(tr.src, tr.dst, Z_DEBUG_BG);
    tess.draw_rect_cos_sin(aabb.center(), Z_DEBUG, aabb.w(), aabb.h(), Vec2::UNIT_X);

    Some(())
}

pub fn debug_parking(tess: &mut Tesselator, goria: &Egregoria, _: &UiWorld) -> Option<()> {
    let map: &Map = &goria.read::<Map>();
    let pm = goria.read::<ParkingManagement>();

    for (id, spot) in map.parking.all_spots() {
        let color = if pm.is_free(id) {
            LinearColor::GREEN
        } else {
            LinearColor::RED
        };

        tess.set_color(color);
        tess.draw_circle(spot.trans.position(), Z_DEBUG, 2.0);
    }

    Some(())
}

pub fn debug_pathfinder(tess: &mut Tesselator, goria: &Egregoria, uiworld: &UiWorld) -> Option<()> {
    let map: &Map = &goria.read::<Map>();
    let selected = uiworld.read::<InspectedEntity>().e?;
    let pos = goria.pos(selected)?;

    let itinerary = goria.comp::<Itinerary>(selected)?;

    tess.set_color(LinearColor::GREEN);
    tess.draw_polyline(itinerary.local_path(), Z_DEBUG, 1.0);

    if let Some(p) = itinerary.get_point() {
        tess.draw_stroke(p, pos, Z_DEBUG, 1.0);
    }

    if let egregoria::map_dynamic::ItineraryKind::Route(r) = itinerary.kind() {
        tess.set_color(LinearColor::RED);
        for l in &r.reversed_route {
            if let Some(l) = l.raw_points(map) {
                tess.draw_polyline(l.as_slice(), Z_DEBUG, 3.0);
            }
        }
        tess.set_color(if itinerary.has_ended(0.0) {
            LinearColor::GREEN
        } else {
            LinearColor::MAGENTA
        });

        tess.draw_circle(r.end_pos, Z_DEBUG, 1.0);
    }
    Some(())
}

pub fn debug_rays(tess: &mut Tesselator, goria: &Egregoria, uiworld: &UiWorld) -> Option<()> {
    let time = goria.read::<GameTime>();
    let time = time.timestamp * 0.2;
    let c = time.cos() as f32;
    let s = time.sin() as f32;
    let mouse = uiworld.read::<MouseInfo>().unprojected;

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
    tess.draw_line(r.from, r.from + r.dir * 50.0, Z_DEBUG);
    tess.draw_line(r2.from, r2.from + r2.dir * 50.0, Z_DEBUG);

    let inter = r.intersection_point(&r2);
    if let Some(v) = inter {
        tess.set_color(LinearColor::RED);

        tess.draw_circle(v, Z_DEBUG, 2.0);
    }

    Some(())
}

pub fn debug_spatialmap(tess: &mut Tesselator, goria: &Egregoria, _: &UiWorld) -> Option<()> {
    let map: &Map = &goria.read::<Map>();
    for r in map.spatial_map().debug_grid() {
        tess.set_color(LinearColor {
            a: 0.1,
            ..LinearColor::BLUE
        });
        tess.draw_rect_cos_sin(r.center(), Z_DEBUG, r.w(), r.h(), Vec2::UNIT_X);
    }

    Some(())
}
