use crate::audio::ambient::Ambient;
use crate::audio::car_sounds::CarSounds;
use crate::audio::music::Music;
use crate::uiworld::UiWorld;
use engine::AudioContext;
use simulation::Simulation;

mod ambient;
mod car_sounds;
mod music;

pub static SOUNDS_LIST: include_dir::Dir = include_dir::include_dir!("assets/sounds");

pub struct GameAudio {
    music: Music,
    ambiant: Ambient,
    carsounds: CarSounds,
}

impl GameAudio {
    pub fn new(ctx: &mut AudioContext) -> Self {
        defer!(log::info!("finished init of game audio"));

        ctx.preload(
            SOUNDS_LIST
                .files()
                .flat_map(|x| x.path().file_name())
                .flat_map(|x| x.to_str())
                .map(|x| x.trim_end_matches(".ogg")),
        );

        Self {
            music: Music::new(),
            ambiant: Ambient::new(ctx),
            carsounds: CarSounds::new(ctx),
        }
    }

    pub fn update(&mut self, sim: &Simulation, uiworld: &UiWorld, ctx: &mut AudioContext) {
        self.music.update(ctx);
        self.ambiant.update(sim, uiworld);
        self.carsounds.update(sim, uiworld, ctx);
    }
}
