use crate::game_loop::Timings;
use crate::gui::bulldozer::BulldozerState;
use crate::gui::lotbrush::LotBrushResource;
use crate::gui::roadbuild::RoadBuildResource;
use crate::gui::roadeditor::RoadEditorResource;
use crate::gui::specialbuilding::SpecialBuildingResource;
use crate::gui::windows::debug::{DebugObjs, DebugState};
use crate::gui::windows::settings::Settings;
use crate::gui::{ExitState, FollowEntity, InspectedEntity, Tool};
use crate::input::{KeyboardInfo, MouseInfo};
use crate::inputmap::InputMap;
use crate::network::NetworkState;
use crate::rendering::immediate::{ImmediateDraw, ImmediateSound};
use crate::uiworld::{ReceivedCommands, UiWorld};
use common::saveload::Encoder;
use egregoria::engine_interaction::WorldCommands;
use serde::de::DeserializeOwned;
use serde::Serialize;

pub static SOUNDS_LIST: include_dir::Dir = include_dir::include_dir!("assets/sounds");

pub fn init() {
    egregoria::init::init();
    register_resource::<Settings>("settings");
    #[cfg(feature = "multiplayer")]
    register_resource::<crate::gui::windows::network::NetworkConnectionInfo>("netinfo");
    register_resource::<LotBrushResource>("lot_brush");

    register_resource_noserialize::<MouseInfo>();
    register_resource_noserialize::<KeyboardInfo>();
    register_resource_noserialize::<InputMap>();
    register_resource_noserialize::<WorldCommands>();
    register_resource_noserialize::<Timings>();
    register_resource_noserialize::<ReceivedCommands>();
    register_resource_noserialize::<RoadEditorResource>();
    register_resource_noserialize::<RoadBuildResource>();
    register_resource_noserialize::<DebugState>();
    register_resource_noserialize::<ExitState>();
    register_resource_noserialize::<DebugObjs>();
    register_resource_noserialize::<FollowEntity>();
    register_resource_noserialize::<BulldozerState>();
    register_resource_noserialize::<InspectedEntity>();
    register_resource_noserialize::<Tool>();
    register_resource_noserialize::<SpecialBuildingResource>();
    register_resource_noserialize::<NetworkState>();
    register_resource_noserialize::<ImmediateSound>();
    register_resource_noserialize::<ImmediateDraw>();
}

pub struct InitFunc {
    pub f: Box<dyn Fn(&mut UiWorld) + 'static>,
}

pub struct SaveLoadFunc {
    pub save: Box<dyn Fn(&UiWorld) + 'static>,
    pub load: Box<dyn Fn(&mut UiWorld) + 'static>,
}

pub static mut INIT_FUNCS: Vec<InitFunc> = Vec::new();
pub static mut SAVELOAD_FUNCS: Vec<SaveLoadFunc> = Vec::new();

fn register_resource_noserialize<T: 'static + Default + Send + Sync>() {
    unsafe {
        INIT_FUNCS.push(InitFunc {
            f: Box::new(|uiw| uiw.insert(T::default())),
        });
    }
}

fn register_resource<T: 'static + Default + Send + Sync + Serialize + DeserializeOwned>(
    name: &'static str,
) {
    unsafe {
        INIT_FUNCS.push(InitFunc {
            f: Box::new(|uiw| uiw.insert(T::default())),
        });
        SAVELOAD_FUNCS.push(SaveLoadFunc {
            save: Box::new(move |uiworld| {
                <common::saveload::JSON as Encoder>::save(&*uiworld.read::<T>(), name);
            }),
            load: Box::new(move |uiworld| {
                if let Some(res) = <common::saveload::JSON as Encoder>::load::<T>(name) {
                    uiworld.insert(res);
                }
            }),
        });
    }
}
