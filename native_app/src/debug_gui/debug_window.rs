#![allow(clippy::type_complexity)]

use crate::game_loop::Timings;
use crate::gui::{GuiState, InspectedEntity};
use crate::uiworld::UiWorld;
use simulation::map_dynamic::ParkingManagement;
use simulation::transportation::TransportGrid;
use simulation::{Simulation, TrainID};
use std::time::{Duration, Instant};

use crate::gui::windows::settings::Settings;
use crate::inputmap::{InputAction, InputMap};
use egui::{Context, Widget};
use engine::{PerfCountersStatic, Tesselator};
use geom::{Camera, Color, LinearColor, Spline3, Vec2};
use prototypes::{GameDuration, GameTime, SECONDS_PER_DAY};
use simulation::map::{
    IntersectionID, Map, MapSubscriber, NetworkObjectID, TraverseKind, UpdateType,
};
use simulation::transportation::train::TrainReservations;
use simulation::world_command::WorldCommand;

#[derive(Default)]
pub struct DebugState {
    pub connectivity: (Option<MapSubscriber>, Vec<Vec<IntersectionID>>),
    pub debug_inspector: bool,
    pub timings_frozen: Option<(Timings, Instant)>,
}

pub struct DebugObjs(
    pub  Vec<(
        bool,
        &'static str,
        fn(&mut Tesselator, &Simulation, &UiWorld) -> Option<()>,
    )>,
);

impl Default for DebugObjs {
    fn default() -> Self {
        DebugObjs(vec![
            (true, "Debug pathfinder", debug_pathfinder),
            (false, "Debug train reservations", debug_trainreservations),
            (false, "Debug connectivity", debug_connectivity),
            (false, "Debug electricity", debug_electricity),
            (false, "Debug spatialmap", debug_spatialmap),
            (false, "Debug transport grid", debug_transport_grid),
            (false, "Debug lots", debug_lots),
            (false, "Debug road points", debug_road_points),
            (false, "Debug parking", debug_parking),
        ])
    }
}

#[derive(Clone)]
pub struct TestFieldProperties {
    size: u32,
    spacing: f32,
}

impl Default for TestFieldProperties {
    fn default() -> Self {
        Self {
            size: 10,
            spacing: 150.0,
        }
    }
}

pub fn debug_window(ui: &Context, uiworld: &UiWorld, sim: &Simulation) {
    if uiworld
        .write::<InputMap>()
        .just_act
        .contains(&InputAction::OpenDebugMenu)
    {
        uiworld.write::<GuiState>().debug_window ^= true;
    }

    let mut opened = uiworld.read::<GuiState>().debug_window;
    debug(
        egui::Window::new("Debug").open(&mut opened),
        ui,
        uiworld,
        sim,
    );
    uiworld.write::<GuiState>().debug_window = opened;
}

/// debug window for various debug options
fn debug(window: egui::Window<'_>, ui: &egui::Context, uiworld: &UiWorld, sim: &Simulation) {
    window.show(ui, |ui| {
        let mut objs = uiworld.write::<DebugObjs>();
        for (val, name, _) in &mut objs.0 {
            ui.checkbox(val, *name);
        }
        ui.checkbox(
            &mut uiworld.write::<DebugState>().debug_inspector,
            "Debug inspector",
        );
        ui.checkbox(
            &mut uiworld.write::<Settings>().gfx.fog_shader_debug,
            "Debug fog shader",
        );
        drop(objs);

        let time = *sim.read::<GameTime>();
        let daysecleft = SECONDS_PER_DAY - sim.read::<GameTime>().daytime.daysec();

        if ui.small_button("set night").clicked() {
            uiworld
                .commands()
                .set_game_time(time + GameDuration::from_secs(daysecleft as u64));
        }

        if ui.small_button("set morning").clicked() {
            uiworld.commands().set_game_time(
                time + GameDuration::from_secs(
                    (daysecleft as f64 + 6.0 * GameTime::HOUR as f64) as u64,
                ),
            );
        }

        if ui.small_button("set day").clicked() {
            uiworld.commands().set_game_time(
                time + GameDuration::from_secs(
                    (daysecleft as f64 + 12.0 * GameTime::HOUR as f64) as u64,
                ),
            );
        }

        if ui.small_button("set dawn").clicked() {
            uiworld.commands().set_game_time(
                time + GameDuration::from_secs(
                    (daysecleft as f64 + 21.7 * GameTime::HOUR as f64) as u64,
                ),
            );
        }

        ui.label(format!(
            "World timestamp: {:.1}",
            sim.read::<GameTime>().timestamp
        ));

        ui.label(format!("Tick: {}", time.tick));

        let mouse = uiworld.read::<InputMap>().unprojected;
        let cam = uiworld.read::<Camera>().pos;

        let mut debug_st = uiworld.write::<DebugState>();

        if let Some((timings_frozen, last_update)) = debug_st.timings_frozen.as_mut() {
            if last_update.elapsed() > Duration::from_millis(300) {
                *timings_frozen = uiworld.read::<Timings>().clone();
                *last_update = Instant::now();
            }
        } else {
            debug_st.timings_frozen = Some((uiworld.read::<Timings>().clone(), Instant::now()));
        }

        let (timings, _) = debug_st.timings_frozen.as_ref().unwrap();

        ui.label("Averaged over last 10 frames: ");
        ui.label(format!(
            "Total time (incl. GPU): {:.1}ms",
            timings.all.avg() * 1000.0
        ));
        ui.label(format!(
            "World update time: {:.1}ms",
            timings.world_update.avg() * 1000.0
        ));
        ui.label(format!(
            "Render prepare time: {:.1}ms",
            timings.render.avg() * 1000.0
        ));
        ui.label(format!(
            "Engine render encode time: {:.1}ms",
            timings.engine_render_time.avg() * 1000.0
        ));
        ui.label(format!(
            "GUI time: {:.1}ms",
            timings.gui_time.avg() * 1000.0
        ));
        ui.label(format!(
            "CPU time: {:.1}ms",
            timings.total_cpu_time.avg() * 1000.0
        ));

        let counters = uiworld.read::<PerfCountersStatic>();
        ui.label(format!("{} drawcalls", counters.total_drawcalls));
        ui.label(format!("{}k triangles", counters.total_triangles / 1000));
        ui.add_space(5.0);
        ui.label(format!("{} depth drawcalls", counters.depth_drawcalls));
        ui.label(format!(
            "{}k depth triangles",
            counters.depth_triangles / 1000
        ));
        ui.add_space(5.0);
        ui.label(format!("{} shadow drawcalls", counters.shadows_drawcalls));
        ui.label(format!(
            "{}k shadow triangles",
            counters.shadows_triangles / 1000
        ));
        ui.add_space(5.0);
        ui.label(format!(
            "{}k heightmap triangles",
            counters.heightmap_triangles / 1000
        ));
        ui.label(format!(
            "{}k heightmap depth triangles",
            counters.heightmap_depth_triangles / 1000
        ));
        ui.label(format!(
            "{}k heightmap shadow triangles",
            counters.heightmap_shadows_triangles / 1000
        ));
        drop(counters);

        if let Some(mouse) = mouse {
            ui.label(format!(
                "World mouse pos: {:.1} {:.1} {:.2}",
                mouse.x, mouse.y, mouse.z
            ));
        }
        ui.label(format!("Cam center:      {:.1} {:.1}", cam.x, cam.y));
        ui.separator();

        if ui.small_button("load Paris map").clicked() {
            uiworld.commands().map_load_paris();
        }
        if ui.small_button("Spawn 10 random cars").clicked() {
            uiworld
                .commands()
                .push(WorldCommand::SpawnRandomCars { n_cars: 10 })
        }
        ui.separator();
        let mut state = uiworld.write::<TestFieldProperties>();

        ui.horizontal(|ui| {
            egui::DragValue::new(&mut state.size)
                .range(2..=100u32)
                .ui(ui);
            ui.label("size");
        });

        ui.horizontal(|ui| {
            egui::DragValue::new(&mut state.spacing)
                .range(30.0..=1000.0f32)
                .ui(ui);
            ui.label("spacing");
        });

        if ui.small_button("load test field").clicked() {
            uiworld.commands().map_load_testfield(
                uiworld.read::<Camera>().pos.xy(),
                state.size,
                state.spacing,
            );
        }

        ui.label(format!("{} pedestrians", sim.world().humans.len()));
        ui.label(format!("{} vehicles", sim.world().vehicles.len()));

        ui.separator();
        ui.label("Game system times");

        ui.columns(2, |ui| {
            ui[0].label("Systen name");
            ui[1].label("Time (ms) over last 100 ticks");

            for &(ref name, time) in &timings.per_game_system {
                ui[0].label(name);
                ui[1].label(format!("{time:.3}"));
            }
        });
    });
}

pub fn debug_lots(tess: &mut Tesselator, sim: &Simulation, _: &UiWorld) -> Option<()> {
    tess.set_color(Color::RED);
    for lot in sim.map().lots().values() {
        tess.draw_circle(lot.shape.corners[0].z(lot.height), 1.0);
    }

    Some(())
}

pub fn debug_road_points(tess: &mut Tesselator, sim: &Simulation, _: &UiWorld) -> Option<()> {
    let map = sim.map();
    tess.set_color(Color::RED.a(0.5));
    for (_, road) in map.roads() {
        for p in road.points.as_slice() {
            tess.draw_circle(p.up(0.02), 0.3);
        }
        tess.draw_polyline(
            &road
                .points()
                .as_slice()
                .iter()
                .map(|x| x.up(0.01))
                .collect::<Vec<_>>(),
            0.3,
            false,
        );
    }

    for (_, lane) in map.lanes() {
        let r = common::rand::rand2(lane.points.first().x, lane.points.first().y);
        tess.set_color(Color::hsv(r * 360.0, 0.8, 0.6, 0.5));

        tess.draw_polyline(
            &lane
                .points
                .as_slice()
                .iter()
                .map(|x| x.up(0.01))
                .collect::<Vec<_>>(),
            0.3,
            false,
        );
    }
    Some(())
}

fn random_color(i: u64) -> Color {
    let r = common::rand::randu(i as u32);
    Color::hsv(r * 360.0, 0.8, 0.6, 0.5)
}

pub fn debug_electricity(tess: &mut Tesselator, sim: &Simulation, _: &UiWorld) -> Option<()> {
    let map = sim.map();

    let getpos = |object: NetworkObjectID| match object {
        NetworkObjectID::Building(b) => Some(map.get(b)?.obb.center().z(5.0)),
        NetworkObjectID::Intersection(i) => Some(map.get(i)?.pos.up(5.0)),
        NetworkObjectID::Road(r) => {
            let road = map.get(r)?;
            Some(road.points.middle().up(5.0))
        }
    };

    for network in map.electricity.networks() {
        tess.set_color(random_color(common::hash_u64(network.id)));

        for object in &network.objects {
            tess.draw_circle(getpos(*object)?, 10.0);
        }
    }

    for (k, v) in map.electricity.graph().iter() {
        tess.set_color(random_color(common::hash_u64(map.electricity.net_id(*k)?)));
        for v in v {
            tess.draw_stroke(getpos(*k)?, getpos(*v)?, 3.0);
        }
    }

    Some(())
}

pub fn debug_connectivity(tess: &mut Tesselator, sim: &Simulation, uiw: &UiWorld) -> Option<()> {
    use simulation::map::pathfinding_crate::directed::strongly_connected_components::strongly_connected_components;
    let mut state = uiw.write::<DebugState>();
    let map = sim.map();

    if state.connectivity.0.is_none() {
        state.connectivity.0 = Some(map.subscribe(UpdateType::Road));
    }
    let sub = state.connectivity.0.as_mut().unwrap();

    if sub.take_updated_chunks().next().is_some() {
        let nodes: Vec<_> = map.intersections().keys().collect();
        let roads = map.roads();
        let inter = map.intersections();
        let components = strongly_connected_components(&nodes, |i| {
            inter
                .get(*i)
                .into_iter()
                .flat_map(|i| i.vehicle_neighbours(roads))
        });
        state.connectivity.1 = components;
    }

    for (i, comp) in state.connectivity.1.iter().enumerate() {
        tess.set_color(random_color(i as u64));

        for int in comp.iter().flat_map(|x| map.intersections().get(*x)) {
            tess.draw_circle(int.pos, 8.0);
        }
    }

    Some(())
}

#[allow(unused)]
fn draw_spline(tess: &mut Tesselator, mut sp: Spline3) {
    sp.from = sp.from.up(0.3);
    sp.to = sp.to.up(0.3);
    tess.set_color(Color::RED);
    tess.draw_polyline(
        &sp.smart_points(0.1, 0.0, 1.0).collect::<Vec<_>>(),
        1.0,
        false,
    );
    tess.set_color(Color::GREEN);

    tess.draw_stroke(sp.from, sp.from + sp.from_derivative, 0.75);
    tess.draw_stroke(sp.to, sp.to - sp.to_derivative, 0.75);

    tess.set_color(Color::PURPLE);
    tess.draw_circle(sp.from, 0.7);
    tess.draw_circle(sp.to, 0.7);

    tess.draw_circle(sp.from + sp.from_derivative, 0.7);
    tess.draw_circle(sp.to - sp.to_derivative, 0.7);
}

fn debug_transport_grid(tess: &mut Tesselator, sim: &Simulation, _: &UiWorld) -> Option<()> {
    let transport_grid = sim.read::<TransportGrid>();

    tess.set_color(Color::new(0.8, 0.8, 0.9, 0.5));
    for h in transport_grid.handles() {
        let (pos, obj) = transport_grid.get(h)?;
        tess.draw_circle(pos.z(obj.height + 0.1), 3.0);
    }
    Some(())
}

/*
pub fn debug_obb(tess: &mut Tesselator<true, sim: &Simulation, uiworld: &UiWorld) -> Option<()> {
    let time = sim.read::<GameTime>();
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

pub fn debug_parking(tess: &mut Tesselator, sim: &Simulation, _: &UiWorld) -> Option<()> {
    let map: &Map = &sim.map();
    let pm = sim.read::<ParkingManagement>();

    for (id, spot) in map.parking.all_spots() {
        let color = if pm.is_spot_free(id) {
            LinearColor::GREEN
        } else {
            LinearColor::RED
        };

        tess.set_color(color);
        tess.draw_circle(spot.trans.pos.up(0.5), 2.0);
    }

    Some(())
}

pub fn debug_trainreservations(
    tess: &mut Tesselator,
    sim: &Simulation,
    uiworld: &UiWorld,
) -> Option<()> {
    let reservs = sim.read::<TrainReservations>();
    let map = sim.map();
    tess.set_color(LinearColor::new(0.8, 0.3, 0.3, 1.0));
    for (id, poses) in &reservs.localisations {
        let points = match id {
            TraverseKind::Lane(lid) => &unwrap_cont!(map.lanes().get(*lid)).points,
            TraverseKind::Turn(tid) => {
                &unwrap_cont!(map.intersections().get(tid.parent))
                    .find_turn(*tid)?
                    .points
            }
        };

        for p in poses.values() {
            let along = points.point_along(*p + points.length());
            tess.draw_circle(along.up(0.3), 3.0);
        }
    }

    for (inter, e) in &reservs.reservations {
        tess.set_color(LinearColor::new(0.3, 0.8, 0.3, 1.0));
        let inter = unwrap_cont!(map.intersections().get(*inter));
        tess.draw_circle(inter.pos.up(0.3), 3.0);

        let p = unwrap_cont!(sim.pos(*e));

        tess.set_color(LinearColor::new(0.2, 0.2, 0.2, 1.0));
        tess.draw_stroke(inter.pos.up(0.5), p, 2.0);
    }
    let selected = uiworld.read::<InspectedEntity>().e?;

    let t_id: TrainID = selected.try_into().ok()?;
    let t = sim.world().trains.get(t_id)?;

    let travers = t.it.get_travers()?;
    let dist_to_next = travers
        .kind
        .length(map.lanes(), map.intersections())
        .unwrap_or(0.0)
        - t.res.cur_travers_dist;

    let stop_dist = t.speed.0 * t.speed.0 / (2.0 * t.locomotive.dec_force);
    for (v, _, _, _) in simulation::transportation::train::traverse_forward(
        &map,
        &t.it,
        stop_dist + 15.0,
        dist_to_next,
        t.locomotive.length + 50.0,
    ) {
        match v {
            TraverseKind::Lane(_) => {}
            TraverseKind::Turn(t) => {
                if map
                    .intersections()
                    .get(t.parent)
                    .map(|i| i.roads.len() <= 2)
                    .unwrap_or(true)
                {
                    continue;
                }
                tess.draw_circle(map.intersections().get(t.parent)?.pos.up(3.0), 3.5);
            }
        }
    }

    Some(())
}

pub fn debug_pathfinder(tess: &mut Tesselator, sim: &Simulation, uiworld: &UiWorld) -> Option<()> {
    let map: &Map = &sim.map();
    let selected = uiworld.read::<InspectedEntity>().e?;
    let pos = sim.pos_any(selected)?;

    let itinerary = sim.world().it_any(selected)?;

    tess.set_color(LinearColor::GREEN);
    tess.draw_polyline(
        &itinerary
            .local_path()
            .iter()
            .map(|x| x.up(0.15))
            .collect::<Vec<_>>(),
        1.0,
        false,
    );

    if let Some(p) = itinerary.get_point() {
        tess.draw_stroke(p.up(0.18), pos.up(0.18), 1.0);
    }

    if let Some(r) = itinerary.get_route() {
        tess.set_color(LinearColor::RED);
        for (i, l) in r.reversed_route.iter().enumerate() {
            if let Some(l) = l.raw_points(map) {
                if i == 0 {
                    tess.set_color(LinearColor::GREEN);
                    let to_cut = l.length() - l.length_at_proj(l.project(r.end_pos));
                    tess.draw_polyline(
                        &l.cut(0.0, to_cut)
                            .as_slice()
                            .iter()
                            .map(|x| x.up(0.1))
                            .collect::<Vec<_>>(),
                        3.0,
                        false,
                    );
                    continue;
                }
                tess.set_color(LinearColor::RED);
                tess.draw_polyline(
                    &l.as_slice().iter().map(|x| x.up(0.1)).collect::<Vec<_>>(),
                    3.0,
                    false,
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
pub fn debug_rays(tess: &mut Tesselator<true, sim: &Simulation, uiworld: &UiWorld) -> Option<()> {
    let time = sim.read::<GameTime>();
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

pub fn debug_spatialmap(tess: &mut Tesselator, sim: &Simulation, _: &UiWorld) -> Option<()> {
    let map: &Map = &sim.map();
    for r in map.spatial_map().debug_grid() {
        tess.set_color(LinearColor::BLUE.a(0.1));
        tess.draw_rect_cos_sin(
            r.center()
                .z(map.environment.height(r.center()).unwrap_or(0.0)),
            r.w(),
            r.h(),
            Vec2::X,
        );
    }

    Some(())
}
