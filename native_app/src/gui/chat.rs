use egui::panel::TopBottomSide;
use egui::{Align2, Color32, Frame, RichText, ScrollArea, TextBuffer, TopBottomPanel};

use geom::Color;
use prototypes::{GameDuration, GameTime};
use simulation::multiplayer::chat::{Message, MessageKind};
use simulation::multiplayer::MultiplayerState;
use simulation::world_command::WorldCommand;
use simulation::Simulation;

use crate::inputmap::{InputAction, InputMap};
use crate::uiworld::UiWorld;

#[derive(Default)]
pub struct GUIChatState {
    cur_msg: String,
    chat_bar_showed: bool,
}

pub fn chat(ui: &egui::Context, uiw: &UiWorld, sim: &Simulation) {
    const MAX_MESSAGES: usize = 30;
    let mut state = uiw.write::<GUIChatState>();
    let five_minute_ago = sim.read::<GameTime>().instant() - GameDuration::from_minutes(5);

    let mstate = sim.read::<MultiplayerState>();

    let just_opened = uiw
        .read::<InputMap>()
        .just_act
        .contains(&InputAction::OpenChat);

    if just_opened {
        state.chat_bar_showed = true;
    }

    let msgs: Vec<_> = mstate
        .chat
        .messages_since(five_minute_ago)
        .take(MAX_MESSAGES)
        .collect();

    if !state.chat_bar_showed && msgs.is_empty() {
        return;
    }

    egui::Window::new("Chat")
        .title_bar(false)
        .fixed_size(egui::Vec2::new(250.0, 300.0))
        .frame(Frame::default().fill(if state.chat_bar_showed {
            Color32::from_black_alpha(192)
        } else {
            Color32::from_black_alpha(64)
        }))
        .anchor(Align2::LEFT_BOTTOM, (0.0, -55.0))
        .show(ui, |ui| {
            ScrollArea::vertical().stick_to_bottom(true).show(ui, |ui| {
                ui.allocate_space(egui::Vec2::new(250.0, 0.0));

                if msgs.len() < 12 {
                    ui.add_space((12 - msgs.len()) as f32 * 24.0);
                }

                for message in msgs.iter().rev() {
                    let color = message.color;

                    let text = RichText::new(message.text.clone());

                    ui.horizontal_wrapped(|ui| {
                        ui.add_space(5.0);
                        ui.colored_label(
                            Color32::from_rgb(
                                (color.r * 255.0) as u8,
                                (color.g * 255.0) as u8,
                                (color.b * 255.0) as u8,
                            ),
                            text,
                        );
                    });
                    ui.add_space(2.0);
                }
            });

            TopBottomPanel::new(TopBottomSide::Bottom, "chat_bar")
                .frame(Frame::default())
                .show_separator_line(false)
                .show_inside(ui, |ui| {
                    if state.chat_bar_showed {
                        let response = ui.add(
                            egui::TextEdit::singleline(&mut state.cur_msg)
                                .desired_width(250.0)
                                .margin(egui::Vec2::new(8.0, 6.0)),
                        );

                        if just_opened {
                            response.request_focus();
                        }

                        if response.lost_focus() {
                            let msg = state.cur_msg.take();

                            if !msg.is_empty() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                                // let rng = common::rand::randu64(common::hash_u64(msg.as_bytes()));
                                // let color = Color::hsv(rng * 360.0, 0.8, 1.0, 1.0);

                                uiw.commands().push(WorldCommand::SendMessage {
                                    message: Message {
                                        name: "player".to_string(),
                                        text: msg,
                                        sent_at: sim.read::<GameTime>().instant(),
                                        color: Color::WHITE,
                                        kind: MessageKind::PlayerChat,
                                    },
                                })
                            }

                            state.chat_bar_showed = false;
                        }
                    } else {
                        ui.allocate_space(egui::Vec2::new(240.0, 26.0));
                    }
                });
        });
}
