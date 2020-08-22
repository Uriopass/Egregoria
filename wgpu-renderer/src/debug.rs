#![allow(clippy::type_complexity)]

use crate::geometry::Tesselator;
use egregoria::engine_interaction::{MouseInfo, TimeInfo};
use egregoria::imgui::im_str;
use egregoria::imgui::Ui;
use egregoria::interaction::{InspectedEntity, Movable, Selectable};
use egregoria::map_interaction::Itinerary;
use egregoria::physics::{CollisionWorld, Transform};
use egregoria::rendering::{Color, LinearColor};
use egregoria::specs::prelude::*;
use geom::obb::OBB;
use geom::splines::Spline;
use geom::{vec2, Vec2};
use lazy_static::*;
use map_model::{Map, RoadSegmentKind};
use std::sync::Mutex;

lazy_static! {
    pub static ref DEBUG_OBJS: Mutex<
        Vec<(
            bool,
            &'static str,
            Box<dyn Send + Fn(&mut Tesselator, &mut World) -> Option<()>>
        )>,
    > = Mutex::new(vec![
        (false, "Debug pathfinder", Box::new(debug_pathfinder)),
        (false, "Debug spatialmap", Box::new(debug_spatialmap)),
        (false, "Debug collision world", Box::new(debug_coworld)),
        (false, "Debug OBBs", Box::new(debug_obb)),
        (false, "Debug rays", Box::new(debug_rays)),
        (false, "Debug splines", Box::new(debug_spline))
    ]);
}

pub fn debug_menu(gui: &mut egregoria::gui::Gui, ui: &Ui) {
    if !gui.show_debug_layers {
        return;
    }
    egregoria::imgui::Window::new(im_str!("Debug layers"))
        .opened(&mut gui.show_debug_layers)
        .build(&ui, || {
            let mut objs = DEBUG_OBJS.lock().unwrap();
            for (val, name, _) in &mut *objs {
                ui.checkbox(&im_str!("{}", *name), val);
            }
        })
}

#[derive(Clone)]
pub struct DbgSplineState {
    pub from: Entity,
    pub to: Entity,
    pub from_der: Entity,
    pub to_der: Entity,
}

pub fn debug_spline(tess: &mut Tesselator, world: &mut World) -> Option<()> {
    if world.try_fetch::<DbgSplineState>().is_none() {
        let from = world
            .create_entity()
            .with(Transform::new([0.0, 0.0]))
            .with(Movable)
            .with(Selectable::new(3.0))
            .build();

        let to = world
            .create_entity()
            .with(Transform::new([10.0, 10.0]))
            .with(Movable)
            .with(Selectable::new(3.0))
            .build();

        let from_der = world
            .create_entity()
            .with(Transform::new([0.0, 2.0]))
            .with(Movable)
            .with(Selectable::new(3.0))
            .build();

        let to_der = world
            .create_entity()
            .with(Transform::new([12.0, 10.0]))
            .with(Movable)
            .with(Selectable::new(3.0))
            .build();

        world.insert(DbgSplineState {
            from,
            to,
            from_der,
            to_der,
        });
    }
    let st = world.write_resource::<DbgSplineState>();

    let tr_st = world.read_storage::<Transform>();

    let from = tr_st.get(st.from).unwrap().position();
    let to = tr_st.get(st.to).unwrap().position();
    let sp = Spline {
        from,
        to,
        from_derivative: tr_st.get(st.from_der).unwrap().position() - from,
        to_derivative: tr_st.get(st.to_der).unwrap().position() - to,
    };

    draw_spline(tess, &sp);

    for road in world.read_resource::<Map>().roads().values() {
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

fn debug_coworld(tess: &mut Tesselator, world: &mut World) -> Option<()> {
    let coworld = world.read_resource::<CollisionWorld>();

    tess.set_color(Color::new(0.8, 0.8, 0.9, 0.5));
    for h in coworld.handles() {
        let pos = coworld.get(h).unwrap().0;
        tess.draw_circle(pos.into(), 1.0, 3.0);
    }
    Some(())
}

pub fn debug_obb(tess: &mut Tesselator, world: &mut World) -> Option<()> {
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

pub fn debug_pathfinder(tess: &mut Tesselator, world: &mut World) -> Option<()> {
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

    if let egregoria::map_interaction::ItineraryKind::Route(r) = itinerary.kind() {
        tess.color = LinearColor::RED;
        for l in &r.reversed_route {
            tess.draw_polyline(l.raw_points(map).as_slice(), 1.0, 3.0);
        }
        tess.color = LinearColor::MAGENTA;
        tess.draw_circle(r.end_pos, 1.0, 1.0);
    }
    Some(())
}

pub fn debug_rays(tess: &mut Tesselator, world: &mut World) -> Option<()> {
    let time = world.read_resource::<TimeInfo>();
    let time = time.time * 0.2;
    let c = time.cos() as f32;
    let s = time.sin() as f32;
    let mouse = world.read_resource::<MouseInfo>().unprojected;

    let r = geom::intersections::Ray {
        from: 10.0 * vec2(c, s),
        dir: vec2(
            (time * 2.3 + 1.0).cos() as f32,
            (time * 2.3 + 1.0).sin() as f32,
        ),
    };

    let r2 = geom::intersections::Ray {
        from: mouse,
        dir: vec2((time * 3.0).cos() as f32, (time * 3.0).sin() as f32),
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

pub fn debug_spatialmap(tess: &mut Tesselator, world: &mut World) -> Option<()> {
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
