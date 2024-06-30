use egui::TextBuffer;
use yakui::{constrained, reflow, Alignment, Color, Constraints, Dim2, Pivot, Vec2};

use goryak::{
    blur_bg, fixed_spacer, mincolumn, padxy, secondary_container, text_edit, textc, VertScroll,
    VertScrollSize,
};
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

pub fn chat(uiw: &UiWorld, sim: &Simulation) {
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

    if uiw
        .read::<InputMap>()
        .just_act
        .contains(&InputAction::Close)
    {
        state.chat_bar_showed = false;
        state.cur_msg.clear();
    }

    let msgs: Vec<_> = mstate
        .chat
        .messages_since(five_minute_ago)
        .take(MAX_MESSAGES)
        .collect();

    if !state.chat_bar_showed && msgs.is_empty() {
        return;
    }

    reflow(
        Alignment::BOTTOM_LEFT,
        Pivot::BOTTOM_LEFT,
        Dim2::pixels(0.0, -192.0),
        || {
            let alpha = if state.chat_bar_showed { 0.7 } else { 0.2 };
            blur_bg(secondary_container().with_alpha(alpha), 0.0, || {
                mincolumn(0.0, || {
                    VertScroll {
                        size: VertScrollSize::Exact(300.0),
                        align_bot: true,
                    }
                    .show(|| {
                        constrained(
                            Constraints {
                                min: Vec2::new(250.0, 0.0),
                                max: Vec2::new(250.0, f32::INFINITY),
                            },
                            || {
                                padxy(8.0, 8.0, || {
                                    mincolumn(8.0, || {
                                        for message in msgs.iter().rev() {
                                            let color = message.color;

                                            let text = message.text.clone();

                                            textc(
                                                Color::rgb(
                                                    (color.r * 255.0) as u8,
                                                    (color.g * 255.0) as u8,
                                                    (color.b * 255.0) as u8,
                                                ),
                                                text,
                                            );
                                        }
                                    });
                                });
                            },
                        );
                    });
                    if state.chat_bar_showed {
                        if text_edit(250.0, &mut state.cur_msg, "") && !state.cur_msg.is_empty() {
                            uiw.commands().push(WorldCommand::SendMessage {
                                message: Message {
                                    name: "player".to_string(),
                                    text: state.cur_msg.take(),
                                    sent_at: sim.read::<GameTime>().instant(),
                                    color: geom::Color::WHITE,
                                    kind: MessageKind::PlayerChat,
                                },
                            });
                            state.chat_bar_showed = false;
                        }
                    } else {
                        fixed_spacer((0.0, 30.0));
                    }
                });
            });
        },
    );
}
