use crate::gui::{ErrorTooltip, InspectedBuilding, PotentialCommands, Tool};
use crate::inputmap::{InputAction, InputMap};
use crate::rendering::immediate::{ImmediateDraw, ImmediateSound};
use crate::uiworld::UiWorld;
use engine::AudioKind;
use geom::{Degrees, Intersect, OBB};
use ordered_float::OrderedFloat;
use prototypes::{RenderAsset, Size2D};
use simulation::map::{ProjectFilter, ProjectKind, RoadID};
use simulation::world_command::WorldCommand;
use simulation::Simulation;
use std::borrow::Cow;

pub struct SpecialBuildArgs {
    pub obb: OBB,
    pub connected_road: Option<RoadID>,
}

pub struct SpecialBuildKind {
    pub make: Box<dyn Fn(&SpecialBuildArgs) -> Vec<WorldCommand> + Send + Sync + 'static>,
    pub size: Size2D,
    pub asset: RenderAsset,
    pub road_snap: bool,
}

#[derive(Default)]
pub struct SpecialBuildingResource {
    pub opt: Option<SpecialBuildKind>,
    pub last_obb: Option<OBB>,
    pub rotation: Degrees,
}

/// SpecialBuilding tool
/// Allows to build special buildings like farms, factories, etc.
pub fn specialbuilding(sim: &Simulation, uiworld: &UiWorld) {
    profiling::scope!("gui::specialbuilding");
    let mut state = uiworld.write::<SpecialBuildingResource>();
    let tool = *uiworld.read::<Tool>();
    let inp = uiworld.read::<InputMap>();
    let mut draw = uiworld.write::<ImmediateDraw>();
    let mut sound = uiworld.write::<ImmediateSound>();

    let map = sim.map();

    let commands = &mut *uiworld.commands();

    if !matches!(tool, Tool::SpecialBuilding) {
        return;
    }

    for command in uiworld.received_commands().iter() {
        if let WorldCommand::MapBuildSpecialBuilding { pos, kind, .. } = command {
            if let Some(ProjectKind::Building(bid)) = map
                .spatial_map()
                .query(pos.center(), ProjectFilter::BUILDING)
                .next()
            {
                if let Some(b) = map.buildings().get(bid) {
                    if b.kind == *kind {
                        uiworld.write::<InspectedBuilding>().e = Some(bid);
                    }
                }
            }
        }
    }

    if inp.act.contains(&InputAction::Rotate) {
        state.rotation += Degrees(inp.wheel);
        state.rotation.normalize();
    }

    let SpecialBuildKind {
        size,
        ref asset,
        ref make,
        road_snap,
    } = *unwrap_or!(&state.opt, return);

    let mpos = unwrap_ret!(inp.unprojected);
    let roads = map.roads();

    let half_diag = 0.5 * size.diag();
    let hover_obb = OBB::new(mpos.xy(), state.rotation.vec2(), size.w, size.h);

    let mut draw = |obb: OBB, red| {
        let col = if red {
            simulation::colors().gui_danger.adjust_luminosity(1.3)
        } else {
            simulation::colors().gui_primary.adjust_luminosity(1.5)
        };

        match asset {
            RenderAsset::Mesh { path } => {
                draw.mesh(
                    path.to_string_lossy().to_string(),
                    obb.center().z(mpos.z),
                    obb.axis()[0].normalize().z0(),
                )
                .color(col);
            }
            RenderAsset::Sprite { path } => {
                draw.textured_obb(obb, path.to_string_lossy().to_string(), mpos.z + 0.1)
                    .color(col);
            }
        }
    };

    let mut rid = None;
    let mut obb = hover_obb;

    if road_snap {
        let closest_road = map
            .spatial_map()
            .query_around(mpos.xy(), half_diag, ProjectFilter::ROAD)
            .filter_map(|x| match x {
                ProjectKind::Road(id) => Some(&roads[id]),
                _ => None,
            })
            .min_by_key(move |p| OrderedFloat(p.points().project_dist2(mpos)));
        let Some(closest_road) = closest_road else {
            *uiworld.write::<ErrorTooltip>() = ErrorTooltip::new(Cow::Borrowed("No road nearby"));
            return draw(hover_obb, true);
        };

        let (proj, _, dir) = closest_road.points().project_segment_dir(mpos);
        let dir = dir.xy();

        if !proj.is_close(mpos, half_diag + closest_road.width * 0.5) {
            *uiworld.write::<ErrorTooltip>() = ErrorTooltip::new(Cow::Borrowed("No road nearby"));
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
            proj.xy() + side * (size.h + closest_road.width + 0.5) * 0.5,
            side,
            size.w,
            size.h,
        );

        if proj.distance(first) < half_diag || proj.distance(last) < half_diag {
            *uiworld.write::<ErrorTooltip>() =
                ErrorTooltip::new(Cow::Borrowed("Too close to side"));
            draw(obb, true);
            return;
        }

        if closest_road.sidewalks(closest_road.src).incoming.is_none() {
            *uiworld.write::<ErrorTooltip>() =
                ErrorTooltip::new(Cow::Borrowed("Sidewalk required"));
            draw(obb, true);
            return;
        }

        rid = Some(closest_road.id);
    }

    if map
        .spatial_map()
        .query(
            obb,
            ProjectFilter::ROAD | ProjectFilter::INTER | ProjectFilter::BUILDING,
        )
        .any(|x| {
            if let Some(rid) = rid {
                ProjectKind::Road(rid) != x
            } else {
                true
            }
        })
        || state.last_obb.map(|x| x.intersects(&obb)).unwrap_or(false)
    {
        *uiworld.write::<ErrorTooltip>() =
            ErrorTooltip::new(Cow::Borrowed("Intersecting with something"));
        draw(obb, true);
        return;
    }

    draw(obb, false);

    let cmds: Vec<WorldCommand> = make(&SpecialBuildArgs {
        obb,
        connected_road: rid,
    });
    if inp.act.contains(&InputAction::Select) {
        commands.extend(cmds);
        sound.play("road_lay", AudioKind::Ui);
        state.last_obb = Some(obb);
    } else if let Some(last) = cmds.last() {
        uiworld.write::<PotentialCommands>().set(last.clone());
    }
}
