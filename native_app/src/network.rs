pub(crate) use self::inner::*;
use crate::game_loop::{State, Timings};
use crate::gui::windows::settings::Settings;
use crate::uiworld::{ReceivedCommands, SaveLoadState};
use common::timestep::Timestep;
use egregoria::engine_interaction::{WorldCommand, WorldCommands};
use egregoria::utils::scheduler::SeqSchedule;
use egregoria::Egregoria;

impl Default for NetworkState {
    fn default() -> Self {
        Self::Singleplayer(Timestep::default())
    }
}

#[cfg(not(feature = "multiplayer"))]
mod inner {
    use crate::network::{State, Timestep};

    #[allow(clippy::large_enum_variant)]
    pub(crate) enum NetworkState {
        Singleplayer(Timestep),
    }

    pub(crate) fn goria_update(state: &mut State) {
        super::handle_singleplayer(state);
    }
}

#[allow(dead_code)]
fn handle_singleplayer(state: &mut State) {
    let mut goria = unwrap_orr!(state.goria.try_write(), return); // mut for tick

    let timewarp = state.uiw.read::<Settings>().time_warp;
    let mut commands = std::mem::take(&mut *state.uiw.write::<WorldCommands>());
    *state.uiw.write::<ReceivedCommands>() = ReceivedCommands::default();

    if handle_replay(
        &mut goria,
        &mut state.game_schedule,
        &mut state.uiw.write::<SaveLoadState>(),
    ) {
        return;
    }

    let sched = &mut state.game_schedule;
    let mut timings = state.uiw.write::<Timings>();

    let mut has_commands = !commands.is_empty();

    if has_commands && commands.iter().all(WorldCommand::is_instant) {
        for v in commands.iter() {
            v.apply(&mut goria);
        }
        commands = WorldCommands::default();
        has_commands = false;
    }

    let mut net_state = state.uiw.write::<NetworkState>();

    #[allow(irrefutable_let_patterns)]
    let NetworkState::Singleplayer(ref mut step) = *net_state else { return; };

    let mut commands_once = Some(commands.clone());
    step.prepare_frame(timewarp);
    while step.tick() || (has_commands && commands_once.is_some()) {
        let t = goria.tick(sched, commands_once.take().unwrap_or_default().as_ref());
        timings.world_update.add_value(t.as_secs_f32());
    }

    if commands_once.is_none() {
        *state.uiw.write::<ReceivedCommands>() = ReceivedCommands::new(commands);
    } else {
        *state.uiw.write::<WorldCommands>() = commands;
    }
}

fn handle_replay(
    goria: &mut Egregoria,
    schedule: &mut SeqSchedule,
    slstate: &mut SaveLoadState,
) -> bool {
    if let Some(new_goria) = slstate.please_load_goria.take() {
        *goria = new_goria;
        log::info!("replaced goria");
    }
    if let Some(ref mut replay) = slstate.please_load {
        if replay.advance(goria, schedule) {
            slstate.please_load = None;
            log::info!("finished loading replay");
        }
        return true;
    }
    false
}

#[cfg(feature = "multiplayer")]
mod inner {
    use crate::game_loop::{State, Timings, VERSION};
    use crate::gui::windows::network::NetworkConnectionInfo;
    use crate::network::handle_replay;
    use crate::uiworld::{ReceivedCommands, SaveLoadState};
    use common::timestep::Timestep;
    use egregoria::engine_interaction::WorldCommands;
    use egregoria::Egregoria;
    use networking::{
        ConnectConf, Frame, PollResult, ServerConfiguration, ServerPollResult, VirtualClientConf,
    };
    use std::net::ToSocketAddrs;
    use std::sync::Mutex;

    pub(crate) type Client = Mutex<networking::Client<Egregoria, WorldCommands>>;
    pub(crate) type Server = Mutex<networking::Server<Egregoria, WorldCommands>>;

    #[allow(clippy::large_enum_variant)]
    pub(crate) enum NetworkState {
        Singleplayer(Timestep),
        Client(Client),
        Server(Server),
    }

    pub(crate) fn goria_update(state: &mut State) {
        if matches!(
            *state.uiw.read::<NetworkState>(),
            NetworkState::Singleplayer(_)
        ) {
            super::handle_singleplayer(state);
            return;
        }

        let mut goria = unwrap_orr!(state.goria.try_write(), return); // mut for tick

        let commands = std::mem::take(&mut *state.uiw.write::<WorldCommands>());
        *state.uiw.write::<ReceivedCommands>() = ReceivedCommands::default();

        if handle_replay(
            &mut goria,
            &mut state.game_schedule,
            &mut state.uiw.write::<SaveLoadState>(),
        ) {
            return;
        }

        let mut net_state = state.uiw.write::<NetworkState>();

        let mut inputs_to_apply = None;
        match &mut *net_state {
            NetworkState::Singleplayer(_) => unreachable!(),
            NetworkState::Server(ref mut server) => {
                let polled =
                    server
                        .get_mut()
                        .unwrap()
                        .poll(&goria, Frame(goria.get_tick()), Some(commands));
                match polled {
                    ServerPollResult::Wait(commands) => {
                        if let Some(commands) = commands {
                            *state.uiw.write::<WorldCommands>() = commands;
                        }
                    }
                    ServerPollResult::Input(inputs) => {
                        inputs_to_apply = Some(inputs);
                    }
                }
            }
            NetworkState::Client(ref mut client) => {
                let polled = client.get_mut().unwrap().poll(commands);
                match polled {
                    PollResult::Wait(commands) => {
                        *state.uiw.write::<WorldCommands>() = commands;
                    }
                    PollResult::Input(inputs) => {
                        inputs_to_apply = Some(inputs);
                    }
                    PollResult::GameWorld(commands, prepared_goria) => {
                        *goria = prepared_goria;
                        *state.uiw.write::<WorldCommands>() = commands;
                    }
                    PollResult::Disconnect(reason) => {
                        log::error!(
                            "got disconnected :-( continuing with server world but it's sad"
                        );
                        *net_state = NetworkState::Singleplayer(Timestep::default());
                        state.uiw.write::<NetworkConnectionInfo>().error = reason;
                    }
                }
            }
        }

        if let Some(inputs) = inputs_to_apply {
            let mut merged = WorldCommands::default();
            for frame_commands in inputs {
                assert_eq!(frame_commands.frame.0, goria.get_tick() + 1);
                let commands: WorldCommands = frame_commands
                    .inputs
                    .iter()
                    .map(|x| x.inp.clone())
                    .collect();
                let t = goria.tick(&mut state.game_schedule, commands.as_ref());
                state
                    .uiw
                    .write::<Timings>()
                    .world_update
                    .add_value(t.as_secs_f32());
                merged.merge(
                    &frame_commands
                        .inputs
                        .into_iter()
                        .filter(|x| x.sent_by_me)
                        .map(|x| x.inp)
                        .collect::<WorldCommands>(),
                );
            }
            *state.uiw.write::<ReceivedCommands>() = ReceivedCommands::new(merged);
        }
    }

    pub(crate) fn start_server(
        info: &mut NetworkConnectionInfo,
        goria: &Egregoria,
    ) -> Option<Server> {
        let server = match networking::Server::start(ServerConfiguration {
            start_frame: Frame(goria.get_tick()),
            period: common::timestep::UP_DT,
            port: None,
            virtual_client: Some(VirtualClientConf {
                name: info.name.to_string(),
            }),
            version: VERSION.to_string(),
            always_run: true,
        }) {
            Ok(x) => x,
            Err(e) => {
                info.error = format!("{:?}", e);
                return None;
            }
        };

        Some(Mutex::new(server))
    }

    pub(crate) fn start_client(info: &mut NetworkConnectionInfo) -> Option<Client> {
        let mut s = info.ip.to_string();
        if !s.contains(':') {
            s += ":23019"
        }
        let parsed_addr = match s.to_socket_addrs() {
            Ok(x) => match x.into_iter().next() {
                Some(x) => x,
                None => {
                    info.error = "no ip found with given address".to_string();
                    return None;
                }
            },
            Err(e) => {
                info.error = e.to_string();
                return None;
            }
        };

        let port = parsed_addr.port();

        let client = match networking::Client::connect(ConnectConf {
            name: info.name.clone(),
            addr: parsed_addr.ip(),
            port: if port != 23019 { Some(port) } else { None },
            frame_buffer_advance: 8,
            version: VERSION.to_string(),
        }) {
            Ok(x) => x,
            Err(e) => {
                info.error = format!("{:?}", e);
                return None;
            }
        };

        Some(Mutex::new(client))
    }
}
