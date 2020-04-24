use cgmath::{vec2, Vector2};
use std::collections::HashSet;
use winit::event::{ElementState, KeyboardInput, MouseScrollDelta, VirtualKeyCode, WindowEvent};

#[derive(Default)]
pub struct InputContext {
    pub mouse: MouseInfo,
    pub keyboard: KeyboardInfo,
}

impl InputContext {
    pub fn end_frame(&mut self) {
        self.mouse.just_pressed.clear();
        self.keyboard.just_pressed.clear();
        self.keyboard.last_characters.clear();
        self.mouse.wheel_delta = 0.0;
    }

    pub fn handle(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::ReceivedCharacter(c) => {
                self.keyboard.last_characters.push(*c);
                true
            }
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        state,
                        virtual_keycode: Some(kc),
                        ..
                    },
                ..
            } => {
                let code = kc.into();
                match state {
                    ElementState::Pressed => {
                        self.keyboard.is_pressed.insert(code);
                        self.keyboard.just_pressed.insert(code);
                    }
                    ElementState::Released => {
                        self.keyboard.is_pressed.remove(&code);
                    }
                };
                true
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.mouse.screen = vec2(position.x as f32, position.y as f32);
                true
            }
            WindowEvent::MouseInput { button, state, .. } => {
                match state {
                    ElementState::Pressed => {
                        self.mouse.just_pressed.insert(button.into());
                        self.mouse.buttons.insert(button.into());
                    }
                    ElementState::Released => {
                        self.mouse.buttons.remove(&button.into());
                    }
                };
                true
            }
            WindowEvent::MouseWheel { delta, .. } => {
                match delta {
                    MouseScrollDelta::LineDelta(_, y) => self.mouse.wheel_delta = *y,
                    _ => {}
                }
                true
            }
            _ => false,
        }
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Other(u8),
}

impl From<&winit::event::MouseButton> for MouseButton {
    fn from(x: &winit::event::MouseButton) -> Self {
        match x {
            winit::event::MouseButton::Left => MouseButton::Left,
            winit::event::MouseButton::Right => MouseButton::Right,
            winit::event::MouseButton::Middle => MouseButton::Middle,
            winit::event::MouseButton::Other(v) => MouseButton::Other(*v),
        }
    }
}

pub struct MouseInfo {
    pub wheel_delta: f32,
    pub screen: Vector2<f32>,
    pub unprojected: Vector2<f32>,
    pub buttons: HashSet<MouseButton>,
    pub just_pressed: HashSet<MouseButton>,
}

impl Default for MouseInfo {
    fn default() -> Self {
        MouseInfo {
            wheel_delta: 0.0,
            screen: vec2(0.0, 0.0),
            unprojected: vec2(0.0, 0.0),
            buttons: HashSet::new(),
            just_pressed: HashSet::new(),
        }
    }
}

pub struct KeyboardInfo {
    pub just_pressed: HashSet<KeyCode>,
    pub is_pressed: HashSet<KeyCode>,
    pub last_characters: Vec<char>,
}

impl Default for KeyboardInfo {
    fn default() -> Self {
        KeyboardInfo {
            just_pressed: HashSet::new(),
            is_pressed: HashSet::new(),
            last_characters: Vec::with_capacity(4),
        }
    }
}

/// Symbolic name for a keyboard key.
#[derive(Debug, Hash, Ord, PartialOrd, PartialEq, Eq, Clone, Copy)]
#[repr(u32)]
pub enum KeyCode {
    /// The '1' key over the letters.
    Key1,
    /// The '2' key over the letters.
    Key2,
    /// The '3' key over the letters.
    Key3,
    /// The '4' key over the letters.
    Key4,
    /// The '5' key over the letters.
    Key5,
    /// The '6' key over the letters.
    Key6,
    /// The '7' key over the letters.
    Key7,
    /// The '8' key over the letters.
    Key8,
    /// The '9' key over the letters.
    Key9,
    /// The '0' key over the 'O' and 'P' keys.
    Key0,

    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,

    /// The Escape key, next to F1.
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

    /// Print Screen/SysRq.
    Snapshot,
    /// Scroll Lock.
    Scroll,
    /// Pause/Break key, next to Scroll lock.
    Pause,

    /// `Insert`, next to Backspace.
    Insert,
    Home,
    Delete,
    End,
    PageDown,
    PageUp,

    Left,
    Up,
    Right,
    Down,

    /// The Backspace key, right over Enter.
    Backspace,
    /// The Enter key.
    Return,
    /// The space bar.
    Space,

    /// The "Compose" key on Linux.
    Compose,

    Caret,

    Numlock,
    Numpad0,
    Numpad1,
    Numpad2,
    Numpad3,
    Numpad4,
    Numpad5,
    Numpad6,
    Numpad7,
    Numpad8,
    Numpad9,

    AbntC1,
    AbntC2,
    Add,
    Apostrophe,
    Apps,
    At,
    Ax,
    Backslash,
    Calculator,
    Capital,
    Colon,
    Comma,
    Convert,
    Decimal,
    Divide,
    Equals,
    Grave,
    Kana,
    Kanji,
    LAlt,
    LBracket,
    LControl,
    LShift,
    LWin,
    Mail,
    MediaSelect,
    MediaStop,
    Minus,
    Multiply,
    Mute,
    MyComputer,
    NavigateForward,  // also called "Prior"
    NavigateBackward, // also called "Next"
    NextTrack,
    NoConvert,
    NumpadComma,
    NumpadEnter,
    NumpadEquals,
    OEM102,
    Period,
    PlayPause,
    Power,
    PrevTrack,
    RAlt,
    RBracket,
    RControl,
    RShift,
    RWin,
    Semicolon,
    Slash,
    Sleep,
    Stop,
    Subtract,
    Sysrq,
    Tab,
    Underline,
    Unlabeled,
    VolumeDown,
    VolumeUp,
    Wake,
    WebBack,
    WebFavorites,
    WebForward,
    WebHome,
    WebRefresh,
    WebSearch,
    WebStop,
    Yen,
    Copy,
    Paste,
    Cut,
}

impl From<&VirtualKeyCode> for KeyCode {
    fn from(x: &VirtualKeyCode) -> Self {
        match x {
            winit::event::VirtualKeyCode::Key1 => KeyCode::Key1,
            winit::event::VirtualKeyCode::Key2 => KeyCode::Key2,
            winit::event::VirtualKeyCode::Key3 => KeyCode::Key3,
            winit::event::VirtualKeyCode::Key4 => KeyCode::Key4,
            winit::event::VirtualKeyCode::Key5 => KeyCode::Key5,
            winit::event::VirtualKeyCode::Key6 => KeyCode::Key6,
            winit::event::VirtualKeyCode::Key7 => KeyCode::Key7,
            winit::event::VirtualKeyCode::Key8 => KeyCode::Key8,
            winit::event::VirtualKeyCode::Key9 => KeyCode::Key9,
            winit::event::VirtualKeyCode::Key0 => KeyCode::Key0,
            winit::event::VirtualKeyCode::A => KeyCode::A,
            winit::event::VirtualKeyCode::B => KeyCode::B,
            winit::event::VirtualKeyCode::C => KeyCode::C,
            winit::event::VirtualKeyCode::D => KeyCode::D,
            winit::event::VirtualKeyCode::E => KeyCode::E,
            winit::event::VirtualKeyCode::F => KeyCode::F,
            winit::event::VirtualKeyCode::G => KeyCode::G,
            winit::event::VirtualKeyCode::H => KeyCode::H,
            winit::event::VirtualKeyCode::I => KeyCode::I,
            winit::event::VirtualKeyCode::J => KeyCode::J,
            winit::event::VirtualKeyCode::K => KeyCode::K,
            winit::event::VirtualKeyCode::L => KeyCode::L,
            winit::event::VirtualKeyCode::M => KeyCode::M,
            winit::event::VirtualKeyCode::N => KeyCode::N,
            winit::event::VirtualKeyCode::O => KeyCode::O,
            winit::event::VirtualKeyCode::P => KeyCode::P,
            winit::event::VirtualKeyCode::Q => KeyCode::Q,
            winit::event::VirtualKeyCode::R => KeyCode::R,
            winit::event::VirtualKeyCode::S => KeyCode::S,
            winit::event::VirtualKeyCode::T => KeyCode::T,
            winit::event::VirtualKeyCode::U => KeyCode::U,
            winit::event::VirtualKeyCode::V => KeyCode::V,
            winit::event::VirtualKeyCode::W => KeyCode::W,
            winit::event::VirtualKeyCode::X => KeyCode::X,
            winit::event::VirtualKeyCode::Y => KeyCode::Y,
            winit::event::VirtualKeyCode::Z => KeyCode::Z,
            winit::event::VirtualKeyCode::Escape => KeyCode::Escape,
            winit::event::VirtualKeyCode::F1 => KeyCode::F1,
            winit::event::VirtualKeyCode::F2 => KeyCode::F2,
            winit::event::VirtualKeyCode::F3 => KeyCode::F3,
            winit::event::VirtualKeyCode::F4 => KeyCode::F4,
            winit::event::VirtualKeyCode::F5 => KeyCode::F5,
            winit::event::VirtualKeyCode::F6 => KeyCode::F6,
            winit::event::VirtualKeyCode::F7 => KeyCode::F7,
            winit::event::VirtualKeyCode::F8 => KeyCode::F8,
            winit::event::VirtualKeyCode::F9 => KeyCode::F9,
            winit::event::VirtualKeyCode::F10 => KeyCode::F10,
            winit::event::VirtualKeyCode::F11 => KeyCode::F11,
            winit::event::VirtualKeyCode::F12 => KeyCode::F12,
            winit::event::VirtualKeyCode::F13 => KeyCode::F13,
            winit::event::VirtualKeyCode::F14 => KeyCode::F14,
            winit::event::VirtualKeyCode::F15 => KeyCode::F15,
            winit::event::VirtualKeyCode::F16 => KeyCode::F16,
            winit::event::VirtualKeyCode::F17 => KeyCode::F17,
            winit::event::VirtualKeyCode::F18 => KeyCode::F18,
            winit::event::VirtualKeyCode::F19 => KeyCode::F19,
            winit::event::VirtualKeyCode::F20 => KeyCode::F20,
            winit::event::VirtualKeyCode::F21 => KeyCode::F21,
            winit::event::VirtualKeyCode::F22 => KeyCode::F22,
            winit::event::VirtualKeyCode::F23 => KeyCode::F23,
            winit::event::VirtualKeyCode::Snapshot => KeyCode::Snapshot,
            winit::event::VirtualKeyCode::F24 => KeyCode::F24,
            winit::event::VirtualKeyCode::Scroll => KeyCode::Scroll,
            winit::event::VirtualKeyCode::Pause => KeyCode::Pause,
            winit::event::VirtualKeyCode::Insert => KeyCode::Insert,
            winit::event::VirtualKeyCode::Home => KeyCode::Home,
            winit::event::VirtualKeyCode::Delete => KeyCode::Delete,
            winit::event::VirtualKeyCode::End => KeyCode::End,
            winit::event::VirtualKeyCode::PageDown => KeyCode::PageDown,
            winit::event::VirtualKeyCode::PageUp => KeyCode::PageUp,
            winit::event::VirtualKeyCode::Left => KeyCode::Left,
            winit::event::VirtualKeyCode::Up => KeyCode::Up,
            winit::event::VirtualKeyCode::Right => KeyCode::Right,
            winit::event::VirtualKeyCode::Down => KeyCode::Down,
            winit::event::VirtualKeyCode::Back => KeyCode::Backspace,
            winit::event::VirtualKeyCode::Return => KeyCode::Return,
            winit::event::VirtualKeyCode::Space => KeyCode::Space,
            winit::event::VirtualKeyCode::Compose => KeyCode::Compose,
            winit::event::VirtualKeyCode::Caret => KeyCode::Caret,
            winit::event::VirtualKeyCode::Numlock => KeyCode::Numlock,
            winit::event::VirtualKeyCode::Numpad0 => KeyCode::Numpad0,
            winit::event::VirtualKeyCode::Numpad1 => KeyCode::Numpad1,
            winit::event::VirtualKeyCode::Numpad2 => KeyCode::Numpad2,
            winit::event::VirtualKeyCode::Numpad3 => KeyCode::Numpad3,
            winit::event::VirtualKeyCode::Numpad4 => KeyCode::Numpad4,
            winit::event::VirtualKeyCode::Numpad5 => KeyCode::Numpad5,
            winit::event::VirtualKeyCode::Numpad6 => KeyCode::Numpad6,
            winit::event::VirtualKeyCode::Numpad7 => KeyCode::Numpad7,
            winit::event::VirtualKeyCode::Numpad8 => KeyCode::Numpad8,
            winit::event::VirtualKeyCode::Numpad9 => KeyCode::Numpad9,
            winit::event::VirtualKeyCode::AbntC1 => KeyCode::AbntC1,
            winit::event::VirtualKeyCode::AbntC2 => KeyCode::AbntC2,
            winit::event::VirtualKeyCode::Add => KeyCode::Add,
            winit::event::VirtualKeyCode::Apostrophe => KeyCode::Apostrophe,
            winit::event::VirtualKeyCode::Apps => KeyCode::Apps,
            winit::event::VirtualKeyCode::At => KeyCode::At,
            winit::event::VirtualKeyCode::Ax => KeyCode::Ax,
            winit::event::VirtualKeyCode::Backslash => KeyCode::Backslash,
            winit::event::VirtualKeyCode::Calculator => KeyCode::Calculator,
            winit::event::VirtualKeyCode::Capital => KeyCode::Capital,
            winit::event::VirtualKeyCode::Colon => KeyCode::Colon,
            winit::event::VirtualKeyCode::Comma => KeyCode::Comma,
            winit::event::VirtualKeyCode::Convert => KeyCode::Convert,
            winit::event::VirtualKeyCode::Decimal => KeyCode::Decimal,
            winit::event::VirtualKeyCode::Divide => KeyCode::Divide,
            winit::event::VirtualKeyCode::Equals => KeyCode::Equals,
            winit::event::VirtualKeyCode::Grave => KeyCode::Grave,
            winit::event::VirtualKeyCode::Kana => KeyCode::Kana,
            winit::event::VirtualKeyCode::Kanji => KeyCode::Kanji,
            winit::event::VirtualKeyCode::LAlt => KeyCode::LAlt,
            winit::event::VirtualKeyCode::LBracket => KeyCode::LBracket,
            winit::event::VirtualKeyCode::LControl => KeyCode::LControl,
            winit::event::VirtualKeyCode::LShift => KeyCode::LShift,
            winit::event::VirtualKeyCode::LWin => KeyCode::LWin,
            winit::event::VirtualKeyCode::Mail => KeyCode::Mail,
            winit::event::VirtualKeyCode::MediaSelect => KeyCode::MediaSelect,
            winit::event::VirtualKeyCode::MediaStop => KeyCode::MediaStop,
            winit::event::VirtualKeyCode::Minus => KeyCode::Minus,
            winit::event::VirtualKeyCode::Multiply => KeyCode::Multiply,
            winit::event::VirtualKeyCode::Mute => KeyCode::Mute,
            winit::event::VirtualKeyCode::MyComputer => KeyCode::MyComputer,
            winit::event::VirtualKeyCode::NavigateForward => KeyCode::NavigateForward,
            winit::event::VirtualKeyCode::NavigateBackward => KeyCode::NavigateBackward,
            winit::event::VirtualKeyCode::NextTrack => KeyCode::NextTrack,
            winit::event::VirtualKeyCode::NoConvert => KeyCode::NoConvert,
            winit::event::VirtualKeyCode::NumpadComma => KeyCode::NumpadComma,
            winit::event::VirtualKeyCode::NumpadEnter => KeyCode::NumpadEnter,
            winit::event::VirtualKeyCode::NumpadEquals => KeyCode::NumpadEquals,
            winit::event::VirtualKeyCode::OEM102 => KeyCode::OEM102,
            winit::event::VirtualKeyCode::Period => KeyCode::Period,
            winit::event::VirtualKeyCode::PlayPause => KeyCode::PlayPause,
            winit::event::VirtualKeyCode::Power => KeyCode::Power,
            winit::event::VirtualKeyCode::PrevTrack => KeyCode::PrevTrack,
            winit::event::VirtualKeyCode::RAlt => KeyCode::RAlt,
            winit::event::VirtualKeyCode::RBracket => KeyCode::RBracket,
            winit::event::VirtualKeyCode::RControl => KeyCode::RControl,
            winit::event::VirtualKeyCode::RShift => KeyCode::RShift,
            winit::event::VirtualKeyCode::RWin => KeyCode::RWin,
            winit::event::VirtualKeyCode::Semicolon => KeyCode::Semicolon,
            winit::event::VirtualKeyCode::Slash => KeyCode::Slash,
            winit::event::VirtualKeyCode::Sleep => KeyCode::Sleep,
            winit::event::VirtualKeyCode::Stop => KeyCode::Stop,
            winit::event::VirtualKeyCode::Subtract => KeyCode::Subtract,
            winit::event::VirtualKeyCode::Sysrq => KeyCode::Sysrq,
            winit::event::VirtualKeyCode::Tab => KeyCode::Tab,
            winit::event::VirtualKeyCode::Underline => KeyCode::Underline,
            winit::event::VirtualKeyCode::Unlabeled => KeyCode::Unlabeled,
            winit::event::VirtualKeyCode::VolumeDown => KeyCode::VolumeDown,
            winit::event::VirtualKeyCode::VolumeUp => KeyCode::VolumeUp,
            winit::event::VirtualKeyCode::Wake => KeyCode::Wake,
            winit::event::VirtualKeyCode::WebBack => KeyCode::WebBack,
            winit::event::VirtualKeyCode::WebFavorites => KeyCode::WebFavorites,
            winit::event::VirtualKeyCode::WebForward => KeyCode::WebForward,
            winit::event::VirtualKeyCode::WebHome => KeyCode::WebHome,
            winit::event::VirtualKeyCode::WebRefresh => KeyCode::WebRefresh,
            winit::event::VirtualKeyCode::WebSearch => KeyCode::WebSearch,
            winit::event::VirtualKeyCode::WebStop => KeyCode::WebStop,
            winit::event::VirtualKeyCode::Yen => KeyCode::Yen,
            winit::event::VirtualKeyCode::Copy => KeyCode::Copy,
            winit::event::VirtualKeyCode::Paste => KeyCode::Paste,
            winit::event::VirtualKeyCode::Cut => KeyCode::Cut,
        }
    }
}
