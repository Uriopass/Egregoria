use crate::timestep::Timestep;
use egregoria::engine_interaction::WorldCommands;
use egregoria::SerPreparedEgregoria;

pub type Client = networking::Client<SerPreparedEgregoria, WorldCommands>;
pub type Server = networking::Server<SerPreparedEgregoria>;

register_resource_noserialize!(NetworkState);
#[allow(clippy::large_enum_variant)]
pub enum NetworkState {
    Singleplayer(Timestep),
    Client { client: Client },
    Server { server: Server, client: Client },
}

impl Default for NetworkState {
    fn default() -> Self {
        Self::Singleplayer(Timestep::new())
    }
}
