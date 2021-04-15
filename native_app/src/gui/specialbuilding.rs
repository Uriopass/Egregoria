use super::Tool;
use crate::input::{MouseButton, MouseInfo};
use crate::rendering::immediate::{ImmediateDraw, ImmediateSound};
use crate::uiworld::UiWorld;
use common::{AudioKind, Z_TOOL};
use egregoria::Egregoria;
use geom::{Intersect, Vec2, OBB};
use map_model::{BuildingGen, BuildingKind, ProjectFilter, ProjectKind};
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};

register_resource_noserialize!(SpecialBuildingResource);
#[derive(Serialize, Deserialize)]
pub struct SpecialBuildingResource {
    pub opt: Option<(BuildingKind, BuildingGen, f32, String)>,
    pub last_obb: Option<OBB>,
}

impl Default for SpecialBuildingResource {
    fn default() -> Self {
        Self {
            opt: None,
            last_obb: None,
        }
    }
}

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
    let (kind, gen, size, asset) = unwrap_or!(&state.opt, return);
    let size = *size;

    let mpos = mouseinfo.unprojected;
    let roads = map.roads();

    let closest_road = map
        .spatial_map()
        .query_around(mpos, size, ProjectFilter::ROAD)
        .filter_map(|x| match x {
            ProjectKind::Road(id) => Some(&roads[id]),
            _ => None,
        })
        .min_by_key(move |p| OrderedFloat(p.points().project_dist2(mpos)));

    let hover_obb = OBB::new(mpos, Vec2::UNIT_Y, size, size);

    let mut draw_red = |obb| {
        draw.textured_obb(obb, asset.to_owned())
            .color(common::config().special_building_invalid_col)
            .z(Z_TOOL);
    };

    let closest_road = unwrap_or!(closest_road, return draw_red(hover_obb));

    let (proj, _, dir) = closest_road.points().project_segment_dir(mpos);

    if !proj.is_close(mpos, size + closest_road.width * 0.5) {
        return draw_red(hover_obb);
    }

    let side = if (mpos - proj).dot(dir.perpendicular()) > 0.0 {
        dir.perpendicular()
    } else {
        -dir.perpendicular()
    };

    let first = closest_road.points().first();
    let last = closest_road.points().last();

    let obb = OBB::new(
        proj + side * (size + closest_road.width + 0.5) * 0.5,
        side,
        size,
        size,
    );

    if proj.distance(first) < 0.5 * size
        || proj.distance(last) < 0.5 * size
        || closest_road.sidewalks(closest_road.src).incoming.is_none()
    {
        draw_red(obb);
        return;
    }

    if map.building_overlaps(obb) || state.last_obb.map(|x| x.intersects(&obb)).unwrap_or(false) {
        draw_red(obb);
        return;
    }

    let rid = closest_road.id;

    draw.textured_obb(obb, asset.to_owned())
        .color(common::config().special_building_col)
        .z(Z_TOOL);

    if mouseinfo.pressed.contains(&MouseButton::Left) {
        commands.map_build_special_building(rid, obb, *kind, *gen);
        sound.play("road_lay", AudioKind::Ui);
        state.last_obb = Some(obb);
    }
}
