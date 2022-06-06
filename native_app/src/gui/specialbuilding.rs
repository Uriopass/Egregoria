use super::Tool;
use crate::input::{MouseButton, MouseInfo};
use crate::rendering::immediate::{ImmediateDraw, ImmediateSound};
use crate::uiworld::UiWorld;
use common::AudioKind;
use egregoria::engine_interaction::WorldCommands;
use egregoria::Egregoria;
use geom::{Intersect, Vec2, OBB};
use map_model::{ProjectFilter, ProjectKind, RoadID};
use ordered_float::OrderedFloat;

pub struct SpecialBuildArgs {
    pub obb: OBB,
    pub road_id: Option<RoadID>,
}

pub struct SpecialBuildKind {
    pub make: Box<dyn Fn(&SpecialBuildArgs, &mut WorldCommands) + Send + Sync + 'static>,
    pub size: f32,
    pub asset: String,
    pub road_snap: bool,
}

#[derive(Default)]
pub struct SpecialBuildingResource {
    pub opt: Option<SpecialBuildKind>,
    pub last_obb: Option<OBB>,
}

#[profiling::function]
pub fn specialbuilding(goria: &Egregoria, uiworld: &mut UiWorld) {
    let mut state = uiworld.write::<SpecialBuildingResource>();
    let tool = *uiworld.read::<Tool>();
    let mouseinfo = uiworld.read::<MouseInfo>();
    let mut draw = uiworld.write::<ImmediateDraw>();
    let mut sound = uiworld.write::<ImmediateSound>();

    let map = goria.map();

    let commands = &mut *uiworld.commands();

    if !matches!(tool, Tool::SpecialBuilding) {
        return;
    }
    let SpecialBuildKind {
        size,
        ref asset,
        ref make,
        road_snap,
    } = *unwrap_or!(&state.opt, return);

    let mpos = unwrap_ret!(mouseinfo.unprojected);
    let roads = map.roads();

    let hover_obb = OBB::new(mpos.xy(), Vec2::Y, size, size);

    let mut draw = |obb, red| {
        let p = asset.to_string();
        let col = if red {
            common::config().special_building_invalid_col
        } else {
            common::config().special_building_col
        };

        if p.ends_with(".png") {
            draw.textured_obb(obb, p, mpos.z + 0.1).color(col);
        } else if p.ends_with(".glb") {
            draw.mesh(p, obb.center().z(mpos.z), obb.axis()[0].normalize().z0())
                .color(col);
        }
    };

    let mut rid = None;
    let mut obb = hover_obb;

    if road_snap {
        let closest_road = map
            .spatial_map()
            .query_around(mpos.xy(), size, ProjectFilter::ROAD)
            .filter_map(|x| match x {
                ProjectKind::Road(id) => Some(&roads[id]),
                _ => None,
            })
            .min_by_key(move |p| OrderedFloat(p.points().project_dist2(mpos)));
        let closest_road = unwrap_or!(closest_road, return draw(hover_obb, true));

        let (proj, _, dir) = closest_road.points().project_segment_dir(mpos);
        let dir = dir.xy();

        if !proj.is_close(mpos, size + closest_road.width * 0.5) {
            return draw(hover_obb, true);
        }

        let side = if (mpos.xy() - proj.xy()).dot(dir.perpendicular()) > 0.0 {
            dir.perpendicular()
        } else {
            -dir.perpendicular()
        };

        let first = closest_road.points().first();
        let last = closest_road.points().last();

        obb = OBB::new(
            proj.xy() + side * (size + closest_road.width + 0.5) * 0.5,
            side,
            size,
            size,
        );

        if proj.distance(first) < 0.5 * size
            || proj.distance(last) < 0.5 * size
            || closest_road.sidewalks(closest_road.src).incoming.is_none()
        {
            draw(obb, true);
            return;
        }

        if map.building_overlaps(obb) || state.last_obb.map(|x| x.intersects(&obb)).unwrap_or(false)
        {
            draw(obb, true);
            return;
        }

        rid = Some(closest_road.id);

        draw(obb, false);
    }

    if mouseinfo.pressed.contains(&MouseButton::Left) {
        make(&SpecialBuildArgs { obb, road_id: rid }, commands);
        sound.play("road_lay", AudioKind::Ui);
        state.last_obb = Some(obb);
    }
}
