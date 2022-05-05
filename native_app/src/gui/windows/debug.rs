#![allow(clippy::type_complexity)]

use crate::game_loop::Timings;
use crate::gui::InspectedEntity;
use crate::input::MouseInfo;
use crate::uiworld::UiWorld;
use egregoria::map_dynamic::{Itinerary, ParkingManagement};
use egregoria::physics::CollisionWorld;
use egregoria::utils::time::{GameTime, SECONDS_PER_DAY};
use egregoria::Egregoria;

use geom::{Camera, Color, LinearColor, Spline3, Vec2};
use imgui::Ui;
use map_model::{IntersectionID, Map, RoadSegmentKind};
use wgpu_engine::Tesselator;

#[derive(Default)]
pub struct DebugState {
    connectivity: (u32, Vec<Vec<IntersectionID>>),
}

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
            (false, "Debug connectivity", debug_connectivity),
            (false, "Debug spatialmap", debug_spatialmap),
            (false, "Debug collision world", debug_coworld),
            (false, "Debug splines", debug_spline),
            (false, "Debug lots", debug_lots),
            (false, "Debug road points", debug_road_points),
            (false, "Debug parking", debug_parking),
            (false, "Show grid", show_grid),
        ])
    }
}

pub fn debug(
    window: imgui::Window<'_, &'static str>,
    ui: &Ui<'_>,
    uiworld: &mut UiWorld,
    goria: &Egregoria,
) {
    window.build(ui, || {
        let mut objs = uiworld.write::<DebugObjs>();
        for (val, name, _) in &mut objs.0 {
            ui.checkbox(name, val);
        }
        drop(objs);

        let time = goria.read::<GameTime>().timestamp;
        let daysecleft = SECONDS_PER_DAY - goria.read::<GameTime>().daytime.daysec();

        if ui.small_button("set night") {
            uiworld
                .commands()
                .set_game_time(GameTime::new(0.1, time + daysecleft as f64));
        }

        if ui.small_button("set morning") {
            uiworld.commands().set_game_time(GameTime::new(
                0.1,
                time + daysecleft as f64 + 7.0 * GameTime::HOUR as f64,
            ));
        }

        if ui.small_button("set day") {
            uiworld.commands().set_game_time(GameTime::new(
                0.1,
                time + daysecleft as f64 + 12.0 * GameTime::HOUR as f64,
            ));
        }

        if ui.small_button("set dawn") {
            uiworld.commands().set_game_time(GameTime::new(
                0.1,
                time + daysecleft as f64 + 18.0 * GameTime::HOUR as f64,
            ));
        }

        let timings = uiworld.read::<Timings>();
        let mouse = uiworld.read::<MouseInfo>().unprojected;
        let cam = uiworld.read::<Camera>().pos;

        ui.text("Averaged over last 10 frames: ");
        ui.text(format!("Total time: {:.1}ms", timings.all.avg() * 1000.0));
        ui.text(format!(
            "World update time: {:.1}ms",
            timings.world_update.avg() * 1000.0
        ));
        ui.text(format!(
            "Render prepare time: {:.1}ms",
            timings.render.avg() * 1000.0
        ));
        if let Some(mouse) = mouse {
            ui.text(format!("World mouse pos: {:.1} {:.1}", mouse.x, mouse.y));
        }
        ui.text(format!("Cam center:      {:.1} {:.1}", cam.x, cam.y));
        ui.separator();
        ui.text("Game system times");

        ui.columns(2, "game times", false);
        ui.text("System name");
        ui.next_column();
        ui.text("Time avg in ms over last 100 ticks");
        ui.next_column();

        for (name, time) in &timings.per_game_system {
            ui.text(name);
            ui.next_column();
            ui.text(format!("{:.3}", *time));
            ui.next_column();
        }
    });
}

pub fn show_grid(tess: &mut Tesselator, g: &Egregoria, uiworld: &UiWorld) -> Option<()> {
    let cam = &*uiworld.read::<Camera>();

    if cam.eye().z > 1000.0 {
        return Some(());
    }

    let gray_maj = 0.5;
    let gray_min = 0.3;
    let map = g.map();
    let h = |p| map.terrain.height(p);
    if cam.eye().z < 300.0 {
        tess.set_color(Color::new(gray_min, gray_min, gray_min, 0.5));
        tess.draw_grid(1.0, h);
    }
    tess.set_color(Color::new(gray_maj, gray_maj, gray_maj, 0.5));
    tess.draw_grid(10.0, h);
    Some(())
}

pub fn debug_spline(tess: &mut Tesselator, goria: &Egregoria, _: &UiWorld) -> Option<()> {
    for road in goria.map().roads().values() {
        if let RoadSegmentKind::Curved((fr_dr, to_der)) = road.segment {
            let fr = road.points.first();
            let to = road.points.last();
            draw_spline(
                tess,
                Spline3 {
                    from: fr,
                    to,
                    from_derivative: fr_dr.z0(),
                    to_derivative: to_der.z0(),
                },
            );
        }
    }

    Some(())
}

pub fn debug_lots(tess: &mut Tesselator, goria: &Egregoria, _: &UiWorld) -> Option<()> {
    tess.set_color(Color::RED);
    for lot in goria.map().lots().values() {
        tess.draw_circle(lot.shape.corners[0].z(lot.height), 1.0);
    }

    Some(())
}

pub fn debug_road_points(tess: &mut Tesselator, goria: &Egregoria, _: &UiWorld) -> Option<()> {
    let map = goria.map();
    tess.set_color(Color::RED.a(0.5));
    for (_, road) in map.roads() {
        for (_, p) in road.points.as_slice().iter().enumerate() {
            tess.draw_circle(p.up(0.02), 0.3);
        }
        tess.draw_polyline(
            &*road
                .points()
                .as_slice()
                .iter()
                .map(|x| x.up(0.01))
                .collect::<Vec<_>>(),
            0.3,
        );
    }

    for (_, lane) in map.lanes() {
        let r = common::rand::rand2(lane.points.first().x, lane.points.first().y);
        tess.set_color(Color::hsv(r * 360.0, 0.8, 0.6, 0.5));

        tess.draw_polyline(
            &*lane
                .points
                .as_slice()
                .iter()
                .map(|x| x.up(0.01))
                .collect::<Vec<_>>(),
            0.3,
        );
    }
    Some(())
}

pub fn debug_connectivity(tess: &mut Tesselator, goria: &Egregoria, uiw: &UiWorld) -> Option<()> {
    use map_model::pathfinding_crate::directed::strongly_connected_components::strongly_connected_components;
    let mut state = uiw.write::<DebugState>();
    let map = goria.map();

    if state.connectivity.0 != map.dirt_id.0 {
        state.connectivity.0 = map.dirt_id.0;
        let nodes: Vec<_> = map.intersections().keys().collect();
        let roads = map.roads();
        let inter = map.intersections();
        let components = strongly_connected_components(&nodes, |i| {
            inter
                .get(*i)
                .into_iter()
                .flat_map(|i| i.driving_neighbours(roads))
        });
        state.connectivity.1 = components;
    }

    for (i, comp) in state.connectivity.1.iter().enumerate() {
        let r = common::rand::randu(i as u32);
        tess.set_color(Color::hsv(r * 360.0, 0.8, 0.6, 0.5));

        for int in comp.iter().flat_map(|x| map.intersections().get(*x)) {
            tess.draw_circle(int.pos, 8.0);
        }
    }

    Some(())
}

fn draw_spline(tess: &mut Tesselator, mut sp: Spline3) {
    sp.from = sp.from.up(0.3);
    sp.to = sp.to.up(0.3);
    tess.set_color(Color::RED);
    tess.draw_polyline(&sp.smart_points(0.1, 0.0, 1.0).collect::<Vec<_>>(), 1.0);
    tess.set_color(Color::GREEN);

    tess.draw_stroke(sp.from, sp.from + sp.from_derivative, 0.75);
    tess.draw_stroke(sp.to, sp.to - sp.to_derivative, 0.75);

    tess.set_color(Color::PURPLE);
    tess.draw_circle(sp.from, 0.7);
    tess.draw_circle(sp.to, 0.7);

    tess.draw_circle(sp.from + sp.from_derivative, 0.7);
    tess.draw_circle(sp.to - sp.to_derivative, 0.7);
}

fn debug_coworld(tess: &mut Tesselator, goria: &Egregoria, _: &UiWorld) -> Option<()> {
    let coworld = goria.read::<CollisionWorld>();

    tess.set_color(Color::new(0.8, 0.8, 0.9, 0.5));
    for h in coworld.handles() {
        let (pos, obj) = coworld.get(h)?;
        tess.draw_circle(pos.z(obj.height + 0.1), 3.0);
    }
    Some(())
}

/*
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
*/

pub fn debug_parking(tess: &mut Tesselator, goria: &Egregoria, _: &UiWorld) -> Option<()> {
    let map: &Map = &goria.map();
    let pm = goria.read::<ParkingManagement>();

    for (id, spot) in map.parking.all_spots() {
        let color = if pm.is_spot_free(id) {
            LinearColor::GREEN
        } else {
            LinearColor::RED
        };

        tess.set_color(color);
        tess.draw_circle(spot.trans.position, 2.0);
    }

    Some(())
}

pub fn debug_pathfinder(tess: &mut Tesselator, goria: &Egregoria, uiworld: &UiWorld) -> Option<()> {
    let map: &Map = &goria.map();
    let selected = uiworld.read::<InspectedEntity>().e?;
    let pos = goria.pos(selected)?;

    let itinerary = goria.comp::<Itinerary>(selected)?;

    tess.set_color(LinearColor::GREEN);
    tess.draw_polyline(
        &*itinerary
            .local_path()
            .iter()
            .map(|x| x.up(0.15))
            .collect::<Vec<_>>(),
        1.0,
    );

    if let Some(p) = itinerary.get_point() {
        tess.draw_stroke(p.up(0.18), pos.up(0.18), 1.0);
    }

    if let egregoria::map_dynamic::ItineraryKind::Route(r, _) = itinerary.kind() {
        tess.set_color(LinearColor::RED);
        for l in &r.reversed_route {
            if let Some(l) = l.raw_points(map) {
                tess.draw_polyline(
                    &*l.as_slice().iter().map(|x| x.up(0.1)).collect::<Vec<_>>(),
                    3.0,
                );
            }
        }
        tess.set_color(if itinerary.has_ended(0.0) {
            LinearColor::GREEN
        } else {
            LinearColor::MAGENTA
        });

        tess.draw_circle(r.end_pos.up(0.2), 1.0);
    }
    Some(())
}

/*
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
    tess.draw_line(r.from, r.from + r.dir * 50.0);
    tess.draw_line(r2.from, r2.from + r2.dir * 50.0);

    let inter = r.intersection_point(&r2);
    if let Some(v) = inter {
        tess.set_color(LinearColor::RED);

        tess.draw_circle(v.z0(), 2.0);
    }

    Some(())
}*/

pub fn debug_spatialmap(tess: &mut Tesselator, goria: &Egregoria, _: &UiWorld) -> Option<()> {
    let map: &Map = &goria.map();
    for r in map.spatial_map().debug_grid() {
        tess.set_color(LinearColor::BLUE.a(0.1));
        tess.draw_rect_cos_sin(
            r.center().z(map.terrain.height(r.center()).unwrap_or(0.0)),
            r.w(),
            r.h(),
            Vec2::X,
        );
    }

    Some(())
}
