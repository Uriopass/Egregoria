use common::timestep::Timestep;
use egregoria::engine_interaction::WorldCommands;
use egregoria::Egregoria;

pub type Client = std::sync::Mutex<networking::Client<Egregoria, WorldCommands>>;
pub type Server = std::sync::Mutex<networking::Server<Egregoria, WorldCommands>>;

#[allow(clippy::large_enum_variant)]
pub enum NetworkState {
    Singleplayer(Timestep),
    Client(Client),
    Server(Server),
}

impl Default for NetworkState {
    fn default() -> Self {
        Self::Singleplayer(Timestep::default())
    }
}
