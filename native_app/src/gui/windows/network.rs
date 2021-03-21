use crate::network::{Client, NetworkState, Server};
use crate::timestep::Timestep;
use crate::uiworld::UiWorld;
use egregoria::Egregoria;
use imgui::{im_str, ImString, Ui};
use networking::{ConnectConf, Frame, ServerConfiguration};
use std::net::{Ipv4Addr, ToSocketAddrs};
use std::time::Duration;

register_resource_noserialize!(NetworkConnectionInfo);
struct NetworkConnectionInfo {
    name: ImString,
    ip: ImString,
    error: String,
}

pub fn network(window: imgui::Window, ui: &Ui, uiworld: &mut UiWorld, goria: &Egregoria) {
    window.build(ui, || {
        let mut state = uiworld.write::<NetworkState>();
        let mut info = uiworld.write::<NetworkConnectionInfo>();

        match *state {
            NetworkState::Singleplayer(_) => {
                if !info.error.is_empty() {
                    ui.text_colored([1.0, 0.0, 0.0, 1.0], &info.error);
                    ui.separator();
                }

                ui.input_text(im_str!("name"), &mut info.name).build();

                if info.name.is_empty() {
                    ui.text("please enter your name");
                    return;
                }

                if ui.small_button(im_str!("Start server")) {
                    if let Some((client, server)) = start_server(&mut *info, goria) {
                        *state = NetworkState::Server { server, client };
                    }
                }

                ui.separator();
                ui.input_text(im_str!("IP"), &mut info.ip).build();
                if ui.small_button(im_str!("Connect")) {
                    if let Some(c) = start_client(&mut info) {
                        *state = NetworkState::Client { client: c };
                    }
                }
            }
            NetworkState::Client { ref client } => {
                ui.text(client.describe());
            }
            NetworkState::Server {
                ref client,
                ref server,
            } => {
                ui.text("Local client:");
                ui.text(client.describe());
                ui.separator();
                ui.text("Running server");
                ui.text(server.describe());
            }
        }
    })
}

fn start_server(info: &mut NetworkConnectionInfo, goria: &Egregoria) -> Option<(Client, Server)> {
    let server = match networking::Server::start(ServerConfiguration {
        start_frame: Frame(goria.get_tick()),
        period: Duration::from_secs_f64(Timestep::DT),
        port: None,
    }) {
        Ok(x) => x,
        Err(e) => {
            info.error = format!("{}", e);
            return None;
        }
    };

    let client = match networking::Client::connect(ConnectConf {
        name: format!("{}", info.name),
        addr: Ipv4Addr::LOCALHOST.into(),
        port: None,
        period: Duration::from_secs_f64(Timestep::DT),
        frame_buffer_advance: 1,
    }) {
        Ok(x) => x,
        Err(e) => {
            info.error = format!("{}", e);
            return None;
        }
    };

    Some((client, server))
}

fn start_client(info: &mut NetworkConnectionInfo) -> Option<Client> {
    let mut s = info.ip.to_string();
    if !s.contains(':') {
        s += ":80"
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
            info.error = format!("{}", e);
            return None;
        }
    };

    let port = parsed_addr.port();

    let client = match networking::Client::connect(ConnectConf {
        name: format!("{}", info.name),
        addr: parsed_addr.ip(),
        port: if port != 80 { Some(port) } else { None },
        period: Duration::from_secs_f64(Timestep::DT),
        frame_buffer_advance: 12,
    }) {
        Ok(x) => x,
        Err(e) => {
            info.error = format!("{}", e);
            return None;
        }
    };

    Some(client)
}

impl Default for NetworkConnectionInfo {
    fn default() -> Self {
        Self {
            name: ImString::with_capacity(100),
            ip: ImString::with_capacity(100),
            error: String::new(),
        }
    }
}
