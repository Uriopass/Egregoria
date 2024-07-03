use common::FastSet;
use geom::{vec2, Vec2};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::sync::{Arc, Mutex};
use winit::event::{ElementState, KeyEvent, MouseScrollDelta, WindowEvent};
use winit::keyboard::{NamedKey, PhysicalKey, SmolStr};
use winit::platform::scancode::PhysicalKeyExtScancode;
use winit::window::CursorIcon;

lazy_static! {
    static ref CURSOR_ICON: Arc<Mutex<(CursorIcon, bool)>> =
        Arc::new(Mutex::new((CursorIcon::Default, false)));
}

pub fn set_cursor_icon(icon: CursorIcon) {
    let old = &mut *CURSOR_ICON.lock().unwrap();
    *old = (icon, old.1 || (old.0 != icon));
}

pub fn get_cursor_icon() -> (CursorIcon, bool) {
    let v = &mut *CURSOR_ICON.lock().unwrap();
    let to_ret = *v;
    v.1 = false;
    to_ret
}

#[derive(Default)]
pub struct InputContext {
    pub mouse: MouseInfo,
    pub keyboard: KeyboardInfo,
    pub cursor_left: bool,
}

impl InputContext {
    pub fn end_frame(&mut self) {
        self.keyboard.last_characters.clear();
        self.mouse.wheel_delta = 0.0;
        self.mouse.screen_delta = Vec2::ZERO;
        self.cursor_left = false;
    }

    pub fn handle_device(&mut self, event: &winit::event::DeviceEvent) {
        #[allow(clippy::single_match)]
        match event {
            winit::event::DeviceEvent::MouseMotion { delta } => {
                self.mouse.screen_delta += vec2(delta.0 as f32, delta.1 as f32);
            }
            _ => {}
        }
    }

    pub fn handle(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::CursorLeft { .. } => {
                self.cursor_left = true;
                true
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        text,
                        logical_key,
                        physical_key,
                        state,
                        ..
                    },
                ..
            } => {
                if let Some(k) = text {
                    self.keyboard.last_characters.extend(k.chars());
                }

                let code = Key::from(logical_key.clone());
                match state {
                    ElementState::Pressed => {
                        self.keyboard.pressed.insert(code);

                        if let PhysicalKey::Code(scancode) = physical_key {
                            self.keyboard
                                .pressed_scancode
                                .insert(scancode.to_scancode().unwrap_or(0));
                        }
                    }
                    ElementState::Released => {
                        self.keyboard.pressed.remove(&code);
                        if let PhysicalKey::Code(scancode) = physical_key {
                            self.keyboard
                                .pressed_scancode
                                .remove(&scancode.to_scancode().unwrap_or(0));
                        }
                    }
                };
                true
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.mouse.screen = vec2(position.x as f32, position.y as f32);
                true
            }
            WindowEvent::MouseInput { button, state, .. } => {
                let b = MouseButton::from(*button);
                match state {
                    ElementState::Pressed => {
                        self.mouse.pressed.insert(b);
                    }
                    ElementState::Released => {
                        self.mouse.pressed.remove(&b);
                    }
                };
                true
            }
            WindowEvent::MouseWheel { delta, .. } => {
                match delta {
                    // Provided mainly by the scroll wheel of computer mouse devices
                    MouseScrollDelta::LineDelta(_, y) => {
                        self.mouse.wheel_delta = *y * 10.0;
                    }
                    // Provided by touchpads and drawing tablets
                    MouseScrollDelta::PixelDelta(pos) => {
                        self.mouse.wheel_delta = pos.y as f32;
                    }
                }

                true
            }
            _ => false,
        }
    }
}

#[derive(Clone, Default)]
pub struct MouseInfo {
    pub wheel_delta: f32,
    pub screen: Vec2,
    pub screen_delta: Vec2,
    pub pressed: FastSet<MouseButton>,
}

#[derive(Clone, Default)]
pub struct KeyboardInfo {
    pub pressed: FastSet<Key>,
    pub pressed_scancode: FastSet<u32>,
    pub last_characters: Vec<char>,
}

impl From<winit::event::MouseButton> for MouseButton {
    fn from(x: winit::event::MouseButton) -> MouseButton {
        match x {
            winit::event::MouseButton::Left => MouseButton::Left,
            winit::event::MouseButton::Right => MouseButton::Right,
            winit::event::MouseButton::Middle => MouseButton::Middle,
            winit::event::MouseButton::Other(v) => MouseButton::Other(v),
            winit::event::MouseButton::Back => MouseButton::Other(4),
            winit::event::MouseButton::Forward => MouseButton::Other(5),
        }
    }
}

impl From<winit::keyboard::Key> for Key {
    fn from(x: winit::keyboard::Key) -> Key {
        match x {
            winit::keyboard::Key::Named(named) => match named {
                NamedKey::Escape => Key::Escape,
                NamedKey::F1 => Key::F1,
                NamedKey::F2 => Key::F2,
                NamedKey::F3 => Key::F3,
                NamedKey::F4 => Key::F4,
                NamedKey::F5 => Key::F5,
                NamedKey::F6 => Key::F6,
                NamedKey::F7 => Key::F7,
                NamedKey::F8 => Key::F8,
                NamedKey::F9 => Key::F9,
                NamedKey::F10 => Key::F10,
                NamedKey::F11 => Key::F11,
                NamedKey::F12 => Key::F12,
                NamedKey::F13 => Key::F13,
                NamedKey::F14 => Key::F14,
                NamedKey::F15 => Key::F15,
                NamedKey::F16 => Key::F16,
                NamedKey::F17 => Key::F17,
                NamedKey::F18 => Key::F18,
                NamedKey::F19 => Key::F19,
                NamedKey::F20 => Key::F20,
                NamedKey::F21 => Key::F21,
                NamedKey::F22 => Key::F22,
                NamedKey::F23 => Key::F23,
                NamedKey::F24 => Key::F24,
                NamedKey::Pause => Key::Pause,
                NamedKey::Insert => Key::Insert,
                NamedKey::Home => Key::Home,
                NamedKey::Delete => Key::Delete,
                NamedKey::End => Key::End,
                NamedKey::PageDown => Key::PageDown,
                NamedKey::PageUp => Key::PageUp,
                NamedKey::ArrowLeft => Key::ArrowLeft,
                NamedKey::ArrowUp => Key::ArrowUp,
                NamedKey::ArrowRight => Key::ArrowRight,
                NamedKey::ArrowDown => Key::ArrowDown,
                NamedKey::Backspace => Key::Backspace,
                NamedKey::Enter => Key::Return,
                NamedKey::Space => Key::Space,
                NamedKey::Compose => Key::Compose,
                NamedKey::NumLock => Key::Numlock,
                NamedKey::Convert => Key::Convert,
                NamedKey::MediaPlayPause => Key::MediaSelect,
                NamedKey::MediaStop => Key::MediaStop,
                NamedKey::Power => Key::Power,
                NamedKey::Tab => Key::Tab,
                NamedKey::Copy => Key::Copy,
                NamedKey::Paste => Key::Paste,
                NamedKey::Cut => Key::Cut,
                NamedKey::Control => Key::Control,
                NamedKey::Shift => Key::Shift,
                NamedKey::Alt => Key::Alt,
                NamedKey::PrintScreen => Key::PrintScreen,
                NamedKey::ScrollLock => Key::ScrollLock,
                _ => Key::Unlabeled,
            },
            winit::keyboard::Key::Character(c) => Key::Char(SmolStr::new(c.to_uppercase())),
            winit::keyboard::Key::Unidentified(_) => Key::Unlabeled,
            winit::keyboard::Key::Dead(_) => Key::Unlabeled,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Other(u16),
}

impl Key {
    pub fn is_modifier(&self) -> bool {
        use Key::*;
        matches!(self, Control | Shift | Alt)
    }

    pub const fn c(s: &str) -> Key {
        Key::Char(SmolStr::new_inline(s))
    }
}

/// Symbolic name for a keyboard key.
/// We copy this from winit because we want to serialize it without implementing serialize for ALL
/// of winit's structs which would eat up compile time.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Key {
    // Make sure modifiers are put at the beginning when sorting
    Alt,
    Control,
    Shift,

    Char(SmolStr),

    Escape,

    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    F13,
    F14,
    F15,
    F16,
    F17,
    F18,
    F19,
    F20,
    F21,
    F22,
    F23,
    F24,

    PrintScreen,
    ScrollLock,
    Pause,

    Insert,
    Home,
    Delete,
    End,
    PageDown,
    PageUp,

    ArrowLeft,
    ArrowUp,
    ArrowRight,
    ArrowDown,

    Backspace,
    Return,
    Space,

    Compose,

    Numlock,

    Convert,
    MediaSelect,
    MediaStop,
    Mute,
    Power,
    Tab,
    Unlabeled,
    Copy,
    Paste,
    Cut,
}
