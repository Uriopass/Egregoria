use crate::rendering::map_mesh::MapMeshHandler;
use common::{Z_SIGNAL, Z_TREE, Z_TREE_SHADOW};
use egregoria::Egregoria;
use flat_spatial::storage::Storage;
use geom::{lerp, vec2, Color, LinearColor, AABB};
use map_model::{Lane, Map, ProjectKind, TrafficBehavior};
use std::rc::Rc;
use wgpu_engine::{
    FrameContext, GfxContext, MultiSpriteBatch, MultiSpriteBatchBuilder, SpriteBatch,
    SpriteBatchBuilder, Tesselator,
};

pub struct RoadRenderer {
    meshb: MapMeshHandler,

    tree_shadows: Option<Rc<SpriteBatch>>,
    tree_shadows_builder: SpriteBatchBuilder,
    trees: Option<Rc<MultiSpriteBatch>>,
    tree_builder: MultiSpriteBatchBuilder,

    trees_dirt_id: u32,
    last_cam: AABB,
}

impl RoadRenderer {
    pub fn new(gfx: &mut GfxContext, goria: &Egregoria) -> Self {
        let tree_builder = MultiSpriteBatchBuilder::from_paths(
            gfx,
            &[
                "assets/tree.png",
                "assets/tree2.png",
                "assets/tree3.png",
                "assets/tree4.png",
                "assets/tree5.png",
                "assets/tree6.png",
                "assets/tree7.png",
            ],
        );
        let tree_shadow_builder = SpriteBatchBuilder::from_path(gfx, "assets/tree_shadow.png");

        RoadRenderer {
            meshb: MapMeshHandler::new(gfx, goria),
            tree_shadows: None,
            tree_shadows_builder: tree_shadow_builder,
            last_cam: AABB::zero(),
            trees: None,
            tree_builder,
            trees_dirt_id: 0,
        }
    }

    fn render_lane_signals(n: &Lane, sr: &mut Tesselator, time: u32) {
        if n.control.is_always() {
            return;
        }

        let dir = n.orientation_from(n.dst);
        let dir_perp = dir.perpendicular();

        let r_center = n.points.last() + dir_perp * -3.5 + dir * -1.0;

        // Stop sign
        if n.control.is_stop_sign() {
            sr.set_color(LinearColor::WHITE);
            sr.draw_regular_polygon(r_center, Z_SIGNAL, 0.5, 8, std::f32::consts::FRAC_PI_8);

            sr.set_color(LinearColor::RED);
            sr.draw_regular_polygon(r_center, Z_SIGNAL, 0.4, 8, std::f32::consts::FRAC_PI_8);
            return;
        }

        // Traffic light
        let size = 0.5; // light size

        sr.color = Color::gray(0.2).into();
        sr.draw_rect_cos_sin(r_center, Z_SIGNAL, size + 0.1, size * 3.0 + 0.1, dir);

        for i in -1..2 {
            sr.draw_circle(r_center + i as f32 * dir_perp * size, Z_SIGNAL, size * 0.5);
        }
        sr.set_color(match n.control.get_behavior(time) {
            TrafficBehavior::RED | TrafficBehavior::STOP => LinearColor::RED,
            TrafficBehavior::ORANGE => LinearColor::ORANGE,
            TrafficBehavior::GREEN => LinearColor::GREEN,
        });

        let offset = match n.control.get_behavior(time) {
            TrafficBehavior::RED => -size,
            TrafficBehavior::ORANGE => 0.0,
            TrafficBehavior::GREEN => size,
            TrafficBehavior::STOP => unreachable!(),
        };

        sr.draw_circle(r_center + offset * dir_perp, Z_SIGNAL, size * 0.5);
    }

    fn signals_render(map: &Map, time: u32, sr: &mut Tesselator) {
        match sr.cull_rect {
            Some(rect) => {
                if rect.w().max(rect.h()) > 1500.0 {
                    return;
                }
                for n in map
                    .spatial_map()
                    .query(rect)
                    .filter_map(|k| match k {
                        ProjectKind::Road(id) => Some(id),
                        _ => None,
                    })
                    .flat_map(|id| map.roads()[id].lanes_iter())
                    .map(|(id, _)| &map.lanes()[id])
                {
                    Self::render_lane_signals(n, sr, time);
                }
            }
            None => {
                for n in map.lanes().values() {
                    Self::render_lane_signals(n, sr, time);
                }
            }
        }
    }

    pub fn trees(
        &mut self,
        map: &Map,
        screen: AABB,
        gfx: &GfxContext,
    ) -> (Rc<MultiSpriteBatch>, Option<Rc<SpriteBatch>>) {
        let st = map.trees.grid.storage();
        if map.trees.dirt_id == self.trees_dirt_id
            && self.tree_shadows.is_some()
            && st.cell_id(screen.ll) == st.cell_id(self.last_cam.ll)
            && st.cell_id(screen.ur) == st.cell_id(self.last_cam.ur)
        {
            if let Some(trees) = self.trees.as_ref() {
                return (trees.clone(), self.tree_shadows.clone());
            }
        }

        self.trees_dirt_id = map.trees.dirt_id;

        self.tree_builder.clear();
        self.tree_shadows_builder.clear();

        let k = screen.w().min(screen.h());
        if k > 4500.0 {
            return (
                Rc::new(self.tree_builder.build(gfx)),
                self.tree_shadows_builder.build(gfx).map(Rc::new),
            );
        }

        let alpha_cutoff = lerp(1.0, 0.0, (k - 3000.0) / 1500.0);

        let tree_col = LinearColor::from(common::config().tree_col).a(alpha_cutoff);

        for (h, _) in map.trees.grid.query_raw(screen.ll, screen.ur) {
            let (pos, t) = map.trees.grid.get(h).unwrap();

            self.tree_shadows_builder.push(
                pos + vec2(1.0, -1.0),
                t.dir,
                Z_TREE_SHADOW,
                LinearColor::WHITE.a(alpha_cutoff),
                (t.size, t.size),
            );

            self.tree_builder
                .sb(
                    (common::rand::rand3(pos.x, pos.y, 10.0) * self.tree_builder.n_texs() as f32)
                        as usize,
                )
                .push(pos, t.dir, Z_TREE, t.col * tree_col, (t.size, t.size));
        }

        (
            Rc::new(self.tree_builder.build(gfx)),
            self.tree_shadows_builder.build(gfx).map(Rc::new),
        )
    }

    pub fn render(&mut self, map: &Map, time: u32, tess: &mut Tesselator, ctx: &mut FrameContext) {
        let screen = tess
            .cull_rect
            .expect("no cull rectangle, might render far too many trees");

        let (trees, tree_shadows) = self.trees(map, screen, ctx.gfx);
        self.trees = Some(trees);
        self.tree_shadows = tree_shadows;

        if let Some(x) = self.meshb.latest_mesh(map, ctx.gfx).clone() {
            ctx.draw(x);
        }

        if let Some(x) = self.tree_shadows.clone() {
            ctx.draw(x);
        }

        if let Some(x) = self.trees.clone() {
            ctx.draw(x);
        }

        Self::signals_render(map, time, tess);

        self.last_cam = screen;
    }
}
