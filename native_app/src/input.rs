use common::FastSet;
use geom::{vec2, Vec2, Vec3};
use std::fmt::Debug;
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

    pub fn handle(&mut self, event: &WindowEvent<'_>) -> bool {
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
                let code = KeyCode::from(*kc);
                match state {
                    ElementState::Pressed => {
                        self.keyboard.pressed.insert(code);
                        self.keyboard.just_pressed.insert(code);
                    }
                    ElementState::Released => {
                        self.keyboard.pressed.remove(&code);
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
                        self.mouse.just_pressed.insert(b);
                        self.mouse.pressed.insert(b);
                    }
                    ElementState::Released => {
                        self.mouse.pressed.remove(&b);
                    }
                };
                true
            }
            WindowEvent::MouseWheel {
                delta: MouseScrollDelta::LineDelta(_, y),
                ..
            } => {
                self.mouse.wheel_delta = *y;
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
    pub unprojected: Option<Vec3>,
    pub pressed: FastSet<MouseButton>,
    pub just_pressed: FastSet<MouseButton>,
}

#[derive(Clone, Default)]
pub struct KeyboardInfo {
    pub just_pressed: FastSet<KeyCode>,
    pub pressed: FastSet<KeyCode>,
    pub last_characters: Vec<char>,
}

impl From<winit::event::MouseButton> for MouseButton {
    fn from(x: winit::event::MouseButton) -> MouseButton {
        match x {
            winit::event::MouseButton::Left => MouseButton::Left,
            winit::event::MouseButton::Right => MouseButton::Right,
            winit::event::MouseButton::Middle => MouseButton::Middle,
            winit::event::MouseButton::Other(v) => MouseButton::Other(v),
        }
    }
}

impl From<VirtualKeyCode> for KeyCode {
    fn from(x: VirtualKeyCode) -> KeyCode {
        match x {
            VirtualKeyCode::Key1 => KeyCode::Key1,
            VirtualKeyCode::Key2 => KeyCode::Key2,
            VirtualKeyCode::Key3 => KeyCode::Key3,
            VirtualKeyCode::Key4 => KeyCode::Key4,
            VirtualKeyCode::Key5 => KeyCode::Key5,
            VirtualKeyCode::Key6 => KeyCode::Key6,
            VirtualKeyCode::Key7 => KeyCode::Key7,
            VirtualKeyCode::Key8 => KeyCode::Key8,
            VirtualKeyCode::Key9 => KeyCode::Key9,
            VirtualKeyCode::Key0 => KeyCode::Key0,
            VirtualKeyCode::A => KeyCode::A,
            VirtualKeyCode::B => KeyCode::B,
            VirtualKeyCode::C => KeyCode::C,
            VirtualKeyCode::D => KeyCode::D,
            VirtualKeyCode::E => KeyCode::E,
            VirtualKeyCode::F => KeyCode::F,
            VirtualKeyCode::G => KeyCode::G,
            VirtualKeyCode::H => KeyCode::H,
            VirtualKeyCode::I => KeyCode::I,
            VirtualKeyCode::J => KeyCode::J,
            VirtualKeyCode::K => KeyCode::K,
            VirtualKeyCode::L => KeyCode::L,
            VirtualKeyCode::M => KeyCode::M,
            VirtualKeyCode::N => KeyCode::N,
            VirtualKeyCode::O => KeyCode::O,
            VirtualKeyCode::P => KeyCode::P,
            VirtualKeyCode::Q => KeyCode::Q,
            VirtualKeyCode::R => KeyCode::R,
            VirtualKeyCode::S => KeyCode::S,
            VirtualKeyCode::T => KeyCode::T,
            VirtualKeyCode::U => KeyCode::U,
            VirtualKeyCode::V => KeyCode::V,
            VirtualKeyCode::W => KeyCode::W,
            VirtualKeyCode::X => KeyCode::X,
            VirtualKeyCode::Y => KeyCode::Y,
            VirtualKeyCode::Z => KeyCode::Z,
            VirtualKeyCode::Escape => KeyCode::Escape,
            VirtualKeyCode::F1 => KeyCode::F1,
            VirtualKeyCode::F2 => KeyCode::F2,
            VirtualKeyCode::F3 => KeyCode::F3,
            VirtualKeyCode::F4 => KeyCode::F4,
            VirtualKeyCode::F5 => KeyCode::F5,
            VirtualKeyCode::F6 => KeyCode::F6,
            VirtualKeyCode::F7 => KeyCode::F7,
            VirtualKeyCode::F8 => KeyCode::F8,
            VirtualKeyCode::F9 => KeyCode::F9,
            VirtualKeyCode::F10 => KeyCode::F10,
            VirtualKeyCode::F11 => KeyCode::F11,
            VirtualKeyCode::F12 => KeyCode::F12,
            VirtualKeyCode::F13 => KeyCode::F13,
            VirtualKeyCode::F14 => KeyCode::F14,
            VirtualKeyCode::F15 => KeyCode::F15,
            VirtualKeyCode::F16 => KeyCode::F16,
            VirtualKeyCode::F17 => KeyCode::F17,
            VirtualKeyCode::F18 => KeyCode::F18,
            VirtualKeyCode::F19 => KeyCode::F19,
            VirtualKeyCode::F20 => KeyCode::F20,
            VirtualKeyCode::F21 => KeyCode::F21,
            VirtualKeyCode::F22 => KeyCode::F22,
            VirtualKeyCode::F23 => KeyCode::F23,
            VirtualKeyCode::Snapshot => KeyCode::Snapshot,
            VirtualKeyCode::F24 => KeyCode::F24,
            VirtualKeyCode::Scroll => KeyCode::Scroll,
            VirtualKeyCode::Pause => KeyCode::Pause,
            VirtualKeyCode::Insert => KeyCode::Insert,
            VirtualKeyCode::Home => KeyCode::Home,
            VirtualKeyCode::Delete => KeyCode::Delete,
            VirtualKeyCode::End => KeyCode::End,
            VirtualKeyCode::PageDown => KeyCode::PageDown,
            VirtualKeyCode::PageUp => KeyCode::PageUp,
            VirtualKeyCode::Left => KeyCode::Left,
            VirtualKeyCode::Up => KeyCode::Up,
            VirtualKeyCode::Right => KeyCode::Right,
            VirtualKeyCode::Down => KeyCode::Down,
            VirtualKeyCode::Back => KeyCode::Backspace,
            VirtualKeyCode::Return => KeyCode::Return,
            VirtualKeyCode::Space => KeyCode::Space,
            VirtualKeyCode::Compose => KeyCode::Compose,
            VirtualKeyCode::Caret => KeyCode::Caret,
            VirtualKeyCode::Numlock => KeyCode::Numlock,
            VirtualKeyCode::Numpad0 => KeyCode::Numpad0,
            VirtualKeyCode::Numpad1 => KeyCode::Numpad1,
            VirtualKeyCode::Numpad2 => KeyCode::Numpad2,
            VirtualKeyCode::Numpad3 => KeyCode::Numpad3,
            VirtualKeyCode::Numpad4 => KeyCode::Numpad4,
            VirtualKeyCode::Numpad5 => KeyCode::Numpad5,
            VirtualKeyCode::Numpad6 => KeyCode::Numpad6,
            VirtualKeyCode::Numpad7 => KeyCode::Numpad7,
            VirtualKeyCode::Numpad8 => KeyCode::Numpad8,
            VirtualKeyCode::Numpad9 => KeyCode::Numpad9,
            VirtualKeyCode::AbntC1 => KeyCode::AbntC1,
            VirtualKeyCode::AbntC2 => KeyCode::AbntC2,
            VirtualKeyCode::NumpadAdd => KeyCode::Add,
            VirtualKeyCode::Apostrophe => KeyCode::Apostrophe,
            VirtualKeyCode::Apps => KeyCode::Apps,
            VirtualKeyCode::At => KeyCode::At,
            VirtualKeyCode::Ax => KeyCode::Ax,
            VirtualKeyCode::Backslash => KeyCode::Backslash,
            VirtualKeyCode::Calculator => KeyCode::Calculator,
            VirtualKeyCode::Capital => KeyCode::Capital,
            VirtualKeyCode::Colon => KeyCode::Colon,
            VirtualKeyCode::Comma => KeyCode::Comma,
            VirtualKeyCode::Convert => KeyCode::Convert,
            VirtualKeyCode::NumpadDecimal => KeyCode::Decimal,
            VirtualKeyCode::NumpadDivide => KeyCode::Divide,
            VirtualKeyCode::Equals => KeyCode::Equals,
            VirtualKeyCode::Grave => KeyCode::Grave,
            VirtualKeyCode::Kana => KeyCode::Kana,
            VirtualKeyCode::Kanji => KeyCode::Kanji,
            VirtualKeyCode::LAlt => KeyCode::LAlt,
            VirtualKeyCode::LBracket => KeyCode::LBracket,
            VirtualKeyCode::LControl => KeyCode::LControl,
            VirtualKeyCode::LShift => KeyCode::LShift,
            VirtualKeyCode::LWin => KeyCode::LWin,
            VirtualKeyCode::Mail => KeyCode::Mail,
            VirtualKeyCode::MediaSelect => KeyCode::MediaSelect,
            VirtualKeyCode::MediaStop => KeyCode::MediaStop,
            VirtualKeyCode::Minus => KeyCode::Minus,
            VirtualKeyCode::NumpadMultiply => KeyCode::Multiply,
            VirtualKeyCode::Mute => KeyCode::Mute,
            VirtualKeyCode::MyComputer => KeyCode::MyComputer,
            VirtualKeyCode::NavigateForward => KeyCode::NavigateForward,
            VirtualKeyCode::NavigateBackward => KeyCode::NavigateBackward,
            VirtualKeyCode::NextTrack => KeyCode::NextTrack,
            VirtualKeyCode::NoConvert => KeyCode::NoConvert,
            VirtualKeyCode::NumpadComma => KeyCode::NumpadComma,
            VirtualKeyCode::NumpadEnter => KeyCode::NumpadEnter,
            VirtualKeyCode::NumpadEquals => KeyCode::NumpadEquals,
            VirtualKeyCode::OEM102 => KeyCode::OEM102,
            VirtualKeyCode::Period => KeyCode::Period,
            VirtualKeyCode::PlayPause => KeyCode::PlayPause,
            VirtualKeyCode::Power => KeyCode::Power,
            VirtualKeyCode::PrevTrack => KeyCode::PrevTrack,
            VirtualKeyCode::RAlt => KeyCode::RAlt,
            VirtualKeyCode::RBracket => KeyCode::RBracket,
            VirtualKeyCode::RControl => KeyCode::RControl,
            VirtualKeyCode::RShift => KeyCode::RShift,
            VirtualKeyCode::RWin => KeyCode::RWin,
            VirtualKeyCode::Semicolon => KeyCode::Semicolon,
            VirtualKeyCode::Slash => KeyCode::Slash,
            VirtualKeyCode::Sleep => KeyCode::Sleep,
            VirtualKeyCode::Stop => KeyCode::Stop,
            VirtualKeyCode::NumpadSubtract => KeyCode::Subtract,
            VirtualKeyCode::Sysrq => KeyCode::Sysrq,
            VirtualKeyCode::Tab => KeyCode::Tab,
            VirtualKeyCode::Underline => KeyCode::Underline,
            VirtualKeyCode::Unlabeled => KeyCode::Unlabeled,
            VirtualKeyCode::VolumeDown => KeyCode::VolumeDown,
            VirtualKeyCode::VolumeUp => KeyCode::VolumeUp,
            VirtualKeyCode::Wake => KeyCode::Wake,
            VirtualKeyCode::WebBack => KeyCode::WebBack,
            VirtualKeyCode::WebFavorites => KeyCode::WebFavorites,
            VirtualKeyCode::WebForward => KeyCode::WebForward,
            VirtualKeyCode::WebHome => KeyCode::WebHome,
            VirtualKeyCode::WebRefresh => KeyCode::WebRefresh,
            VirtualKeyCode::WebSearch => KeyCode::WebSearch,
            VirtualKeyCode::WebStop => KeyCode::WebStop,
            VirtualKeyCode::Yen => KeyCode::Yen,
            VirtualKeyCode::Copy => KeyCode::Copy,
            VirtualKeyCode::Paste => KeyCode::Paste,
            VirtualKeyCode::Cut => KeyCode::Cut,
            VirtualKeyCode::Plus => KeyCode::Plus,
            VirtualKeyCode::Asterisk => KeyCode::Asterisk,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Other(u16),
}

impl KeyCode {
    #[allow(dead_code)]
    fn is_modifier(&self) -> bool {
        use KeyCode::*;
        matches!(self, LShift | RShift | LControl | RControl | LAlt | RAlt)
    }
}

/// Symbolic name for a keyboard key.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u32)]
pub enum KeyCode {
    Key1,
    Key2,
    Key3,
    Key4,
    Key5,
    Key6,
    Key7,
    Key8,
    Key9,
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

    Plus,
    Asterisk,

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
