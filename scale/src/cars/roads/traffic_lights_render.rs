use crate::cars::roads::RoadGraph;
use crate::cars::RoadNodeComponent;
use crate::engine_interaction::TimeInfo;
use crate::rendering::meshrender_component::MeshRender;
use crate::rendering::meshrender_component::MeshRenderEnum;
use specs::prelude::*;
use specs::shred::PanicHandler;

struct TrafficLightRender;

impl<'a> System<'a> for TrafficLightRender {
    type SystemData = (
        Read<'a, TimeInfo>,
        Read<'a, RoadGraph, PanicHandler>,
        ReadStorage<'a, RoadNodeComponent>,
        WriteStorage<'a, MeshRender>,
    );

    fn run(&mut self, (time, rg, rncs, mut meshrenders): Self::SystemData) {
        for (rnc, mr) in (&rncs, &mut meshrenders).join() {
            mr.orders.last_mut().map(|x| match x {
                MeshRenderEnum::Circle(c) => {
                    c.color = rg.nodes()[&rnc.id]
                        .light
                        .get_color(time.time as u64)
                        .as_render_color();
                }
                _ => {}
            });
        }
    }
}
