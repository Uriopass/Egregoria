use crate::network::{Client, NetworkState, Server};
use crate::uiworld::UiWorld;
use common::saveload::Encoder;
use egregoria::Egregoria;
use imgui::Ui;
use networking::{ConnectConf, Frame, ServerConfiguration, VirtualClientConf};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::BTreeMap;
use std::net::ToSocketAddrs;
use std::sync::Mutex;

pub struct NetworkConnectionInfo {
    name: String,
    ip: String,
    pub error: String,
    show_hashes: bool,
    hashes: BTreeMap<String, u64>,
}

pub fn network(
    window: imgui::Window<'_, &'static str>,
    ui: &Ui<'_>,
    uiworld: &mut UiWorld,
    goria: &Egregoria,
) {
    window.build(ui, || {
        let mut state = uiworld.write::<NetworkState>();
        let mut info = uiworld.write::<NetworkConnectionInfo>();
        common::saveload::JSON::save_silent(&*info, "netinfo");

        match *state {
            NetworkState::Singleplayer(_) => {
                if !info.error.is_empty() {
                    ui.text_colored([1.0, 0.0, 0.0, 1.0], &info.error);
                    ui.separator();
                }

                ui.input_text("name", &mut info.name).build();

                if info.name.is_empty() {
                    ui.text("please enter your name");
                    return;
                }

                if ui.small_button("Start server") {
                    if let Some(server) = start_server(&mut *info, goria) {
                        *state = NetworkState::Server(server);
                    }
                }

                ui.separator();
                ui.input_text("IP", &mut info.ip).build();
                if ui.small_button("Connect") {
                    if let Some(c) = start_client(&mut info) {
                        *state = NetworkState::Client(c);
                    }
                }
            }
            NetworkState::Client(ref client) => {
                ui.text(client.lock().unwrap().describe());
                show_hashes(ui, goria, &mut *info);
            }
            NetworkState::Server(ref server) => {
                ui.text("Running server");
                ui.text(server.lock().unwrap().describe());
                show_hashes(ui, goria, &mut *info);
            }
        }
    });
}

fn show_hashes(ui: &Ui<'_>, goria: &Egregoria, info: &mut NetworkConnectionInfo) {
    ui.checkbox("show hashes", &mut info.show_hashes);
    if !info.show_hashes {
        return;
    }

    if goria.get_tick() % 100 == 0 || info.hashes.is_empty() {
        info.hashes = goria.hashes();
    }

    for (name, hash) in &info.hashes {
        ui.text(format!("{}: {}", name, hash));
    }
}

fn start_server(info: &mut NetworkConnectionInfo, goria: &Egregoria) -> Option<Server> {
    let server = match networking::Server::start(ServerConfiguration {
        start_frame: Frame(goria.get_tick()),
        period: common::timestep::UP_DT,
        port: None,
        virtual_client: Some(VirtualClientConf {
            name: info.name.to_string(),
        }),
        version: goria_version::VERSION.to_string(),
        always_run: true,
    }) {
        Ok(x) => x,
        Err(e) => {
            info.error = format!("{}", e);
            return None;
        }
    };

    Some(Mutex::new(server))
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
        name: info.name.clone(),
        addr: parsed_addr.ip(),
        port: if port != 80 { Some(port) } else { None },
        frame_buffer_advance: 8,
        version: goria_version::VERSION.to_string(),
    }) {
        Ok(x) => x,
        Err(e) => {
            info.error = format!("{}", e);
            return None;
        }
    };

    Some(Mutex::new(client))
}

impl Default for NetworkConnectionInfo {
    fn default() -> Self {
        Self {
            name: String::with_capacity(100),
            ip: String::with_capacity(100),
            error: String::new(),
            show_hashes: false,
            hashes: Default::default(),
        }
    }
}

impl Serialize for NetworkConnectionInfo {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        (self.name.to_string(), self.ip.to_string()).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for NetworkConnectionInfo {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
    where
        D: Deserializer<'de>,
    {
        let (mut name, mut ip) = <(String, String) as Deserialize>::deserialize(deserializer)?;
        name.reserve(100);
        ip.reserve(100);
        Ok(Self {
            name,
            ip,
            ..Default::default()
        })
    }
}
