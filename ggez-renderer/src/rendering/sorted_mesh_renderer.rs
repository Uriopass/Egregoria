use crate::rendering::meshrenderable::MeshRenderable;
use crate::rendering::render_context::RenderContext;
use scale::engine_interaction::MAX_LAYERS;
use scale::physics::Transform;
use scale::rendering::meshrender_component::MeshRender;
use scale::specs::{BitSet, Join, World, WorldExt};

pub struct SortedMeshRenderer {
    layers: Vec<BitSet>,
}

impl SortedMeshRenderer {
    pub fn new() -> Self {
        SortedMeshRenderer {
            layers: (0..MAX_LAYERS).map(|_| BitSet::new()).collect(),
        }
    }

    pub fn render(&mut self, world: &mut World, rc: &mut RenderContext) {
        let transforms = world.read_component::<Transform>();
        let mesh_render = world.write_component::<MeshRender>();

        for layer in &mut self.layers {
            layer.clear()
        }

        // To iterate over all inserted/modified components;
        for (mr, id) in (&mesh_render, mesh_render.mask()).join() {
            self.layers[mr.layer() as usize].add(id);
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
