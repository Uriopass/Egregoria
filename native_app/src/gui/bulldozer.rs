use super::Tool;
use egregoria::engine_interaction::{MouseButton, MouseInfo};
use egregoria::rendering::meshrender_component::{CircleRender, MeshRender};
use egregoria::NoSerialize;
use geom::{Color, Transform, Vec2};
use legion::world::SubWorld;
use legion::{system, IntoQuery};
use legion::{Entity, World};
use map_model::{Map, ProjectKind};

pub struct BulldozerResource {
    project: Entity,
}

impl BulldozerResource {
    pub fn new(world: &mut World) -> Self {
        let mut mr = MeshRender::simple(
            CircleRender {
                offset: Vec2::ZERO,
                radius: 2.0,
                color: Color::RED,
            },
            0.9,
        );
        mr.hide = true;

        let e = world.push((Transform::zero(), mr, NoSerialize));
        Self { project: e }
    }
}

#[system]
#[write_component(MeshRender)]
#[write_component(Transform)]
pub fn bulldozer(
    #[resource] tool: &Tool,
    #[resource] mouseinfo: &MouseInfo,
    #[resource] map: &mut Map,
    #[resource] self_r: &BulldozerResource,
    sw: &mut SubWorld,
) {
    let (mr, transform): (&mut MeshRender, &mut Transform) =
        <(&mut MeshRender, &mut Transform)>::query()
            .get_mut(sw, self_r.project)
            .unwrap();

    if !matches!(*tool, Tool::Bulldozer) {
        mr.hide = true;
        return;
    }
    mr.hide = false;

    let cur_proj = map.project(mouseinfo.unprojected);

    transform.set_position(cur_proj.pos);

    if mouseinfo.just_pressed.contains(&MouseButton::Left) {
        let mut potentially_empty = Vec::new();
        log::info!("bulldozer {:?}", cur_proj);
        match cur_proj.kind {
            ProjectKind::Inter(id) => {
                potentially_empty.extend(map.intersections()[id].neighbors(map.roads()));
                map.remove_intersection(id)
            }
            ProjectKind::Road(id) => {
                let r = &map.roads()[id];

                potentially_empty.push(r.src);
                potentially_empty.push(r.dst);

                map.remove_road(id);
            }
            ProjectKind::Building(id) => {
                map.remove_building(id);
            }
            ProjectKind::Ground | ProjectKind::Lot(_) => {}
        }

        for id in potentially_empty {
            if map.intersections()[id].roads.is_empty() {
                map.remove_intersection(id);
            }
        }
    }
}
