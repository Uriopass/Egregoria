#![allow(clippy::type_complexity)]

use common::inspect::InspectedEntity;
use common::{GameTime, SECONDS_PER_DAY};
use egregoria::engine_interaction::MouseInfo;
use egregoria::map_dynamic::Itinerary;
use egregoria::physics::CollisionWorld;
use egregoria::rendering::{Color, LinearColor};
use egregoria::Egregoria;
use geom::Spline;
use geom::{vec2, Vec2};
use geom::{Camera, OBB};
use imgui::im_str;
use imgui::Ui;
use lazy_static::*;
use map_model::{Map, RoadSegmentKind};
use std::sync::Mutex;
use wgpu_engine::Tesselator;

lazy_static! {
    pub static ref DEBUG_OBJS: Mutex<
        Vec<(
            bool,
            &'static str,
            Box<dyn Send + Fn(&mut Tesselator, &mut Egregoria) -> Option<()>>
        )>,
    > = Mutex::new(vec![
        (true, "Debug pathfinder", Box::new(debug_pathfinder)),
        (false, "Debug spatialmap", Box::new(debug_spatialmap)),
        (false, "Debug collision world", Box::new(debug_coworld)),
        (false, "Debug OBBs", Box::new(debug_obb)),
        (false, "Debug rays", Box::new(debug_rays)),
        (false, "Debug splines", Box::new(debug_spline)),
        (false, "Show grid", Box::new(show_grid))
    ]);
}

pub fn add_debug_menu(gui: &mut gui::Gui) {
    gui.windows.insert(
        im_str!("Debug layers"),
        |ui: &Ui, goria: &mut Egregoria| {
            let mut objs = DEBUG_OBJS.lock().unwrap();
            for (val, name, _) in &mut *objs {
                ui.checkbox(&im_str!("{}", *name), val);
            }
            drop(objs);
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
        },
        false,
    );
}

pub fn show_grid(tess: &mut Tesselator, state: &mut Egregoria) -> Option<()> {
    let cam = &*state.read::<Camera>();

    if cam.zoom < 1.0 {
        return Some(());
    }

    let gray_maj = 0.5;
    let gray_min = 0.3;
    if cam.zoom > 6.0 {
        tess.set_color(Color::new(gray_min, gray_min, gray_min, 0.5));
        tess.draw_grid(1.0);
    }
    tess.set_color(Color::new(gray_maj, gray_maj, gray_maj, 0.5));
    tess.draw_grid(10.0);
    Some(())
}

pub fn debug_spline(tess: &mut Tesselator, world: &mut Egregoria) -> Option<()> {
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

fn debug_coworld(tess: &mut Tesselator, world: &mut Egregoria) -> Option<()> {
    let coworld = world.read::<CollisionWorld>();

    tess.set_color(Color::new(0.8, 0.8, 0.9, 0.5));
    for h in coworld.handles() {
        let pos = coworld.get(h).unwrap().0;
        tess.draw_circle(pos.into(), 1.0, 3.0);
    }
    Some(())
}

pub fn debug_obb(tess: &mut Tesselator, world: &mut Egregoria) -> Option<()> {
    let time = world.read::<GameTime>();
    let mouse = world.read::<MouseInfo>().unprojected;

    let time = time.timestamp * 0.2;
    let c = time.cos() as f32;
    let s = time.sin() as f32;

    let obb1 = OBB::new(Vec2::ZERO, vec2(c, s), 10.0, 5.0);

    let obb2 = OBB::new(
        mouse,
        vec2((time * 3.0).cos() as f32, (time * 3.0).sin() as f32),
        8.0,
        6.0,
    );

    let mut color = if obb1.intersects(obb2) {
        LinearColor::RED
    } else {
        LinearColor::BLUE
    };

    if obb1.contains(mouse) {
        color = LinearColor::CYAN
    }

    color.a = 0.5;

    tess.set_color(color);
    tess.draw_filled_polygon(&obb1.corners, 0.99);
    tess.draw_filled_polygon(&obb2.corners, 0.99);

    Some(())
}

pub fn debug_pathfinder(tess: &mut Tesselator, world: &mut Egregoria) -> Option<()> {
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

pub fn debug_rays(tess: &mut Tesselator, world: &mut Egregoria) -> Option<()> {
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

    let inter = geom::intersection_point(r, r2);
    if let Some(v) = inter {
        tess.set_color(LinearColor::RED);

        tess.draw_circle(v, 0.5, 2.0);
    }

    Some(())
}

pub fn debug_spatialmap(tess: &mut Tesselator, world: &mut Egregoria) -> Option<()> {
    let map: &Map = &world.read::<Map>();
    for r in map.spatial_map().debug_grid() {
        tess.set_color(LinearColor {
            a: 0.1,
            ..LinearColor::BLUE
        });
        tess.draw_rect_cos_sin(
            vec2(r.x + r.w * 0.5, r.y + r.h * 0.5),
            1.0,
            r.w,
            r.h,
            Vec2::UNIT_X,
        );
    }

    Some(())
}
