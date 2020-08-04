#![allow(dead_code)]

use crate::geometry::Tesselator;
use geom::obb::OBB;
use geom::{vec2, Vec2};
use lazy_static::*;
use map_model::Map;
use scale::engine_interaction::{MouseInfo, TimeInfo};
use scale::imgui::im_str;
use scale::imgui::Ui;
use scale::interaction::InspectedEntity;
use scale::map_interaction::Itinerary;
use scale::physics::Transform;
use scale::rendering::LinearColor;
use scale::specs::prelude::*;
use std::sync::Mutex;

lazy_static! {
    pub static ref DEBUG_OBJS: Mutex<
        Vec<(
            bool,
            &'static str,
            Box<dyn Sync + Send + Fn(&mut Tesselator, &World) -> Option<()>>
        )>,
    > = Mutex::new(vec![
        (false, "Debug pathfindder", Box::new(debug_pathfinder)),
        (false, "Debug rays", Box::new(debug_rays)),
        (false, "Debug spatialmap", Box::new(debug_spatialmap)),
        (false, "Debug OBBs", Box::new(debug_obb))
    ]);
}

pub fn debug_menu(gui: &mut scale::gui::Gui, ui: &Ui) {
    if gui.show_debug {
        scale::imgui::Window::new(im_str!("Debug layers"))
            .opened(&mut gui.show_debug)
            .build(&ui, || {
                let mut objs = DEBUG_OBJS.lock().unwrap();
                for (val, name, _) in &mut *objs {
                    ui.checkbox(&im_str!("{}", *name), val);
                }
            })
    }
}

pub fn debug_obb(tess: &mut Tesselator, world: &World) -> Option<()> {
    let time = world.read_resource::<TimeInfo>();
    let mouse = world.read_resource::<MouseInfo>().unprojected;

    let time = time.time * 0.2;
    let c = time.cos() as f32;
    let s = time.sin() as f32;

    let obb1 = OBB::new(Vec2::ZERO, vec2(c, s), 10.0, 5.0);

    let obb2 = OBB::new(
        mouse,
        vec2((time * 3.0).cos() as f32, (time * 3.0).sin() as f32),
        8.0,
        6.0,
    );

    let color = if obb1.intersects(obb2) {
        LinearColor::RED
    } else {
        LinearColor::BLUE
    };

    tess.set_color(color);
    tess.draw_filled_polygon(&obb1.corners, 0.99);
    tess.draw_filled_polygon(&obb2.corners, 0.99);

    Some(())
}

pub fn debug_pathfinder(tess: &mut Tesselator, world: &World) -> Option<()> {
    let map: &Map = &world.read_resource::<Map>();
    let selected = world.read_resource::<InspectedEntity>().e?;
    let pos = world.read_storage::<Transform>().get(selected)?.position();

    let stor = world.read_storage::<Itinerary>();
    let itinerary = stor.get(selected)?;

    tess.color = LinearColor::GREEN;
    tess.draw_polyline(&itinerary.local_path(), 1.0, 1.0);

    if let Some(p) = itinerary.get_point() {
        tess.draw_stroke(p, pos, 1.0, 1.0);
    }

    if let scale::map_interaction::ItineraryKind::Route(r) = itinerary.kind() {
        tess.color = LinearColor::RED;
        for l in &r.reversed_route {
            tess.draw_polyline(l.raw_points(map).as_slice(), 1.0, 3.0);
        }
        tess.color = LinearColor::MAGENTA;
        tess.draw_circle(r.end_pos, 1.0, 1.0);
    }
    Some(())
}

pub fn debug_rays(tess: &mut Tesselator, world: &World) -> Option<()> {
    let time = world.read_resource::<TimeInfo>();
    let time = time.time * 0.2;
    let c = time.cos() as f32;
    let s = time.sin() as f32;

    let r = geom::intersections::Ray {
        from: 10.0 * vec2(c, s),
        dir: vec2(
            (time * 2.3 + 1.0).cos() as f32,
            (time * 2.3 + 1.0).sin() as f32,
        ),
    };

    let r2 = geom::intersections::Ray {
        from: 10.0 * vec2((time as f32 * 1.5 + 3.0).cos(), s * 2.0),
        dir: vec2(c, -s),
    };

    tess.color = LinearColor::WHITE;
    tess.draw_line(r.from, r.from + r.dir * 50.0, 0.5);
    tess.draw_line(r2.from, r2.from + r2.dir * 50.0, 0.5);

    let inter = geom::intersections::intersection_point(r, r2);
    if let Some(v) = inter {
        tess.color = LinearColor::RED;

        tess.draw_circle(v, 0.5, 2.0);
    }

    Some(())
}

pub fn debug_spatialmap(tess: &mut Tesselator, world: &World) -> Option<()> {
    let map: &Map = &world.read_resource::<Map>();
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
