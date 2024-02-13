use crate::uiworld::UiWorld;
use common::FastMap;
use engine::wgpu::TextureFormat;
use engine::yakui::YakuiWrapper;
use engine::{GfxContext, TextureBuilder};
use geom::{Camera, Degrees, Vec3};
use prototypes::{BuildingPrototypeID, RenderAsset};
use simulation::map::BuildingID;
use simulation::world_command::WorldCommand;
use simulation::{AnyEntity, Simulation};
use std::borrow::Cow;
use yakui::TextureId;

mod hud;
mod tools;

pub use hud::*;
pub use tools::*;

#[derive(Default)]
pub struct IconTextures {
    texs: FastMap<BuildingPrototypeID, engine::Texture>,
    ids: Vec<TextureId>,
}

pub fn do_icons(gfx: &mut GfxContext, uiw: &mut UiWorld, yakui: &mut YakuiWrapper) {
    let mut state = uiw.write::<IconTextures>();

    let mut cam = Camera::new(Vec3::new(0.0, 0.0, 0.0), 256.0, 256.0);

    cam.fovy = tweak!(30.0);
    cam.pitch = Degrees(35.0).into();
    cam.yaw = Degrees(tweak!(-130.0)).into();

    state.ids.clear();

    for building in prototypes::BuildingPrototype::iter() {
        let RenderAsset::Mesh { ref path } = building.asset else {
            continue;
        };
        //if state.texs.contains_key(&building.id) {
        //    continue;
        //}
        let Ok(mesh) = gfx.mesh(path.as_ref()) else {
            continue;
        };

        let t = TextureBuilder::empty(128, 128, 1, TextureFormat::Rgba8UnormSrgb)
            .with_label("building icon")
            .with_usage(
                engine::wgpu::TextureUsages::COPY_DST
                    | engine::wgpu::TextureUsages::RENDER_ATTACHMENT
                    | engine::wgpu::TextureUsages::TEXTURE_BINDING,
            )
            .build_no_queue(&gfx.device);

        let t_msaa = TextureBuilder::empty(128, 128, 1, TextureFormat::Rgba8UnormSrgb)
            .with_label("building icon msaa")
            .with_usage(engine::wgpu::TextureUsages::RENDER_ATTACHMENT)
            .with_sample_count(4)
            .build_no_queue(&gfx.device);

        let aabb3 = mesh.lods[0].aabb3;
        cam.pos = aabb3.center();
        cam.dist = aabb3.ll.distance(aabb3.ur);
        cam.update();

        mesh.render_to_texture(&cam, gfx, &t, &t_msaa);
        let tex_id = yakui.add_texture(&t);

        state.texs.insert(building.id, t);
        state.ids.push(tex_id);
    }
}

pub fn run_ui_systems(sim: &Simulation, uiworld: &UiWorld) {
    profiling::scope!("gui::run_ui_systems");
    bulldozer::bulldozer(sim, uiworld);
    inspected_aura::inspected_aura(sim, uiworld);
    lotbrush::lotbrush(sim, uiworld);
    roadbuild::roadbuild(sim, uiworld);
    roadeditor::roadeditor(sim, uiworld);
    specialbuilding::specialbuilding(sim, uiworld);
    addtrain::addtrain(sim, uiworld);
    zoneedit::zoneedit(sim, uiworld);
    terraforming::terraforming(sim, uiworld);

    // run last so other systems can have the chance to cancel select
    selectable::selectable(sim, uiworld);
}

#[derive(Default, Clone, Debug)]
pub struct ErrorTooltip {
    pub msg: Option<Cow<'static, str>>,
    // Whether this tooltip is about something happening in the game world
    // Avoid showing tooltip when the UI is hovered
    pub isworld: bool,
}

impl ErrorTooltip {
    pub fn new(msg: impl Into<Cow<'static, str>>) -> Self {
        Self {
            msg: Some(msg.into()),
            isworld: true,
        }
    }

    #[allow(unused)]
    pub fn new_ui(msg: impl Into<Cow<'static, str>>) -> Self {
        Self {
            msg: Some(msg.into()),
            isworld: false,
        }
    }
}

#[derive(Default, Clone, Debug)]
pub struct PotentialCommands(pub Vec<WorldCommand>);

impl PotentialCommands {
    pub fn set(&mut self, cmd: WorldCommand) {
        self.0.clear();
        self.0.push(cmd);
    }
}

#[derive(Copy, Clone, Debug, Default)]
pub struct InspectedBuilding {
    pub e: Option<BuildingID>,
    pub dontclear: bool,
}

#[derive(Copy, Clone, Debug)]
pub struct InspectedEntity {
    pub e: Option<AnyEntity>,
    pub dist2: f32,
    pub dontclear: bool,
}

impl Default for InspectedEntity {
    fn default() -> Self {
        Self {
            e: None,
            dist2: 0.0,
            dontclear: false,
        }
    }
}

/// Time that always progresses even when the game is paused
/// Is moduloed to 3600
#[derive(Copy, Clone, Debug, Default)]
pub struct TimeAlways(pub f32);

#[derive(Copy, Clone, Default, Serialize, Deserialize, Eq, PartialEq)]
pub enum Tool {
    #[default]
    Hand,
    RoadbuildStraight,
    RoadbuildCurved,
    RoadEditor,
    Bulldozer,
    LotBrush,
    SpecialBuilding,
    Train,
    Terraforming,
}

impl Tool {
    pub fn is_roadbuild(&self) -> bool {
        matches!(self, Tool::RoadbuildStraight | Tool::RoadbuildCurved)
    }

    pub fn show_arrows(&self) -> bool {
        matches!(
            self,
            Tool::RoadbuildStraight
                | Tool::RoadbuildCurved
                | Tool::RoadEditor
                | Tool::Bulldozer
                | Tool::Train
        )
    }

    pub fn show_lots(&self) -> bool {
        matches!(
            self,
            Tool::RoadbuildStraight | Tool::RoadbuildCurved | Tool::Bulldozer | Tool::LotBrush
        )
    }
}
