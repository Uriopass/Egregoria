use crate::input::{InputContext, KeyCode, MouseButton};
use common::{FastMap, FastSet};
use std::fmt::{Debug, Display, Formatter};

pub struct InputCombinations(pub(crate) Vec<InputCombination>);

pub struct InputMap {
    pub just_act: FastSet<InputAction>,
    pub act: FastSet<InputAction>,
    pub input_mapping: FastMap<InputAction, InputCombinations>,
}

impl InputMap {
    #[rustfmt::skip]
    pub fn default_mapping() -> FastMap<InputAction, InputCombinations> {
        let mut m = FastMap::default();
        use InputAction::*;
        use InputCombination::*;

        let ic = |x| InputCombinations(x);

        for (k, v) in vec![
            (GoForward,     ic(vec![Key(KeyCode::Z), Key(KeyCode::Up)])),
            (GoBackward,    ic(vec![Key(KeyCode::S), Key(KeyCode::Down)])),
            (GoLeft,        ic(vec![Key(KeyCode::Q), Key(KeyCode::Left)])),
            (GoRight,       ic(vec![Key(KeyCode::D), Key(KeyCode::Right)])),
            (CameraMove,    ic(vec![Mouse(MouseButton::Right)])),
            (CameraRotate,  ic(vec![MouseModifier(KeyCode::LShift, MouseButton::Right), Mouse(MouseButton::Middle)])),
            (Zoom,          ic(vec![Key(KeyCode::Plus), WheelUp])),
            (Dezoom,        ic(vec![Key(KeyCode::Minus), WheelDown])),
            (Close,         ic(vec![Key(KeyCode::Escape)])),
            (Select,        ic(vec![Mouse(MouseButton::Left)])),
            (HideInterface, ic(vec![Key(KeyCode::H)])),
        ] {
            m.insert(k, v);
        }

        m
    }

    pub fn prepare_frame(&mut self, input: &InputContext) {
        self.just_act.clear();
        let kb = &input.keyboard;
        let mouse = &input.mouse;
        for (act, comb) in &self.input_mapping {
            let is_match = comb.0.iter().any(|x| match x {
                InputCombination::Key(code) => kb.pressed.contains(code),
                InputCombination::KeyModifier(modif, code) => {
                    kb.pressed.contains(modif) && kb.pressed.contains(code)
                }
                InputCombination::Mouse(mb) => mouse.pressed.contains(mb),
                InputCombination::MouseModifier(modif, mb) => {
                    kb.pressed.contains(modif) && mouse.pressed.contains(mb)
                }
                InputCombination::WheelUp => mouse.wheel_delta > 0.0,
                InputCombination::WheelDown => mouse.wheel_delta < 0.0,
            });

            if is_match {
                if self.act.insert(*act) {
                    self.just_act.insert(*act);
                }
            } else {
                self.act.remove(act);
            }
        }
    }
}

#[allow(dead_code)]
pub enum InputCombination {
    Key(KeyCode),
    KeyModifier(KeyCode, KeyCode),
    Mouse(MouseButton),
    MouseModifier(KeyCode, MouseButton),
    WheelUp,
    WheelDown,
}

impl Display for InputCombinations {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for (i, x) in self.0.iter().enumerate() {
            x.fmt(f)?;
            if i < self.0.len() - 1 {
                f.write_str(", ")?;
            }
        }
        Ok(())
    }
}

impl Display for InputCombination {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            InputCombination::Key(code) => Debug::fmt(code, f),
            InputCombination::KeyModifier(modif, code) => {
                Debug::fmt(modif, f)?;
                write!(f, " + ")?;
                Debug::fmt(code, f)
            }
            InputCombination::Mouse(mb) => Debug::fmt(mb, f),
            InputCombination::MouseModifier(modif, mb) => {
                Debug::fmt(modif, f)?;
                write!(f, " + ")?;
                Debug::fmt(mb, f)
            }
            InputCombination::WheelUp => {
                write!(f, "Scroll Up")
            }
            InputCombination::WheelDown => {
                write!(f, "Scroll Down")
            }
        }
    }
}

impl Default for InputMap {
    fn default() -> Self {
        Self {
            just_act: Default::default(),
            act: Default::default(),
            input_mapping: Self::default_mapping(),
        }
    }
}

#[derive(Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub enum InputAction {
    GoLeft,
    GoRight,
    GoForward,
    GoBackward,
    CameraMove,
    CameraRotate,
    Zoom,
    Dezoom,
    Close,
    Select,
    HideInterface,
}

impl Display for InputAction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                InputAction::GoLeft => "Go Left",
                InputAction::GoRight => "Go Right",
                InputAction::GoForward => "Go Forward",
                InputAction::GoBackward => "Go Backward",
                InputAction::CameraMove => "Camera Move",
                InputAction::CameraRotate => "Camera Rotate",
                InputAction::Zoom => "Zoom",
                InputAction::Dezoom => "Dezoom",
                InputAction::Close => "Close",
                InputAction::Select => "Select",
                InputAction::HideInterface => "Hide interface",
            }
        )
    }
}
