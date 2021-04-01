use common::timestep::Timestep;
use egregoria::engine_interaction::WorldCommands;
use egregoria::SerPreparedEgregoria;

pub type Client = networking::Client<SerPreparedEgregoria, WorldCommands>;
pub type Server = networking::Server<SerPreparedEgregoria, WorldCommands>;

register_resource_noserialize!(NetworkState);
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
