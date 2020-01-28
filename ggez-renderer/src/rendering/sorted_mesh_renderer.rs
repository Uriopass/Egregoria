use crate::rendering::meshrenderable::MeshRenderable;
use crate::rendering::render_context::RenderContext;
use scale::engine_interaction::{MeshRenderEventReader, MAX_LAYERS};
use scale::physics::Transform;
use scale::rendering::meshrender_component::MeshRender;
use specs::storage::ComponentEvent;
use specs::{BitSet, Join, World, WorldExt};

pub struct SortedMeshRenderer {
    inserted: BitSet,
    removed: BitSet,

    layers: Vec<BitSet>,
}

impl SortedMeshRenderer {
    pub fn new() -> Self {
        SortedMeshRenderer {
            layers: (0..MAX_LAYERS).map(|_| BitSet::new()).collect(),
            inserted: BitSet::new(),
            removed: BitSet::new(),
        }
    }

    pub fn render(&mut self, world: &mut World, rc: &mut RenderContext) {
        let transforms = world.read_component::<Transform>();
        let mesh_render = world.write_component::<MeshRender>();

        self.inserted.clear();
        self.removed.clear();
        {
            let mut reader_id = world.write_resource::<MeshRenderEventReader>();

            let events = mesh_render.channel().read(&mut reader_id.0);
            for event in events {
                match event {
                    ComponentEvent::Inserted(id) => {
                        self.inserted.add(*id);
                    }
                    ComponentEvent::Removed(id) => {
                        self.removed.add(*id);
                    }
                    _ => (),
                };
            }
        }

        // To iterate over all inserted/modified components;
        for (mr, id) in (&mesh_render, &self.inserted).join() {
            let b: &mut BitSet = &mut self.layers[mr.layer() as usize];
            b.add(id);
        }

        // To iterate over all inserted/modified components;
        for (mr, id) in (&mesh_render, &self.removed).join() {
            let b: &mut BitSet = &mut self.layers[mr.layer() as usize];
            b.remove(id);
        }

        for b in &self.layers {
            for (trans, mr, _) in (&transforms, &mesh_render, b).join() {
                if mr.hide {
                    continue;
                }
                for order in &mr.orders {
                    order.draw(trans, &transforms, rc);
                }
            }
        }
    }
}
