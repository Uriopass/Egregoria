use cgmath::{vec2, Vector2};
use std::collections::HashSet;
use winit::event::{ElementState, MouseScrollDelta, WindowEvent};

#[derive(Default)]
pub struct InputContext {
    pub mouse: MouseInfo,
    pub keyboard: KeyboardInfo,
}

impl InputContext {
    pub fn end_frame(&mut self) {
        self.mouse.just_pressed.clear();
        self.keyboard.just_pressed.clear();
        self.mouse.wheel_delta = 0.0;
    }
    pub fn handle(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::CursorMoved { position, .. } => {
                self.mouse.screen = vec2(position.x as f32, position.y as f32);
                true
            }
            WindowEvent::MouseInput { button, state, .. } => {
                match state {
                    ElementState::Pressed => self.mouse.buttons.insert(button.into()),
                    ElementState::Released => self.mouse.buttons.remove(&button.into()),
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
}

impl Default for KeyboardInfo {
    fn default() -> Self {
        KeyboardInfo {
            just_pressed: HashSet::new(),
            is_pressed: HashSet::new(),
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
