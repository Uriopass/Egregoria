use crate::input::{InputContext, KeyCode, KeyboardInfo, MouseButton, MouseInfo};
use common::{FastMap, FastSet};
use std::collections::hash_map::Entry;
use std::collections::HashSet;
use std::fmt::{Debug, Display, Formatter};

// Either combinations can work
pub struct InputCombinations(pub(crate) Vec<InputCombination>);

#[derive(Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
enum UnitInput {
    Key(KeyCode),
    Mouse(MouseButton),
    WheelUp,
    WheelDown,
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
    Rotate,
    Close,
    Select,
    HideInterface,
}

// All unit inputs need to match
pub struct InputCombination(Vec<UnitInput>);

struct InputTree {
    action: Option<InputAction>,
    childs: FastMap<UnitInput, Box<InputTree>>,
}
pub struct InputMap {
    pub just_act: FastSet<InputAction>,
    pub act: FastSet<InputAction>,
    pub input_mapping: FastMap<InputAction, InputCombinations>,
    pub wheel: f32,
    input_tree: InputTree,
}

impl InputMap {
    #[rustfmt::skip]
    pub fn default_mapping() -> FastMap<InputAction, InputCombinations> {
        let mut m = FastMap::default();
        use InputAction::*;
        use UnitInput::*;
        use MouseButton::*;
        use KeyCode as K;

        macro_rules! ics {
            [$($($v:expr),+);+] => {
                InputCombinations(vec![$(InputCombination(vec![$($v),+])),+])
            }
        }

        for (k, v) in vec![
            (GoForward,     ics![Key(K::Z) ; Key(K::Up)]),
            (GoBackward,    ics![Key(K::S) ; Key(K::Down)]),
            (GoLeft,        ics![Key(K::Q) ; Key(K::Left)]),
            (GoRight,       ics![Key(K::D) ; Key(K::Right)]),
            (CameraMove,    ics![Mouse(Right)]),
            (CameraRotate,  ics![Key(K::LShift), Mouse(Right) ; Mouse(Middle)]),
            (Zoom,          ics![Key(K::Plus) ; WheelUp]),
            (Dezoom,        ics![Key(K::Minus) ; WheelDown]),
            (Rotate,        ics![Key(K::LControl), WheelUp ; Key(K::LControl), WheelDown]),
            (Close,         ics![Key(K::Escape)]),
            (Select,        ics![Mouse(Left)]),
            (HideInterface, ics![Key(K::H)]),
        ] {
            if m.insert(k, v).is_some() {
                log::error!("inserting same action twice!");
            }
        }

        m
    }

    fn build_input_tree(&mut self) {
        for v in &mut self.input_mapping.values_mut() {
            for x in &mut v.0 {
                x.0.sort()
            }
        }
        self.input_tree = InputTree::new(&self.input_mapping);
    }

    pub fn prepare_frame(&mut self, input: &InputContext) {
        self.just_act.clear();
        let kb = &input.keyboard;
        let mouse = &input.mouse;
        let mut acts: FastSet<_> = self.input_tree.query(kb, mouse).collect();
        std::mem::swap(&mut self.act, &mut acts);
        for v in &self.act {
            if !acts.contains(v) {
                self.just_act.insert(v.clone());
            }
        }
        self.wheel = input.mouse.wheel_delta;
    }
}

impl Display for InputCombinations {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for (i, x) in self.0.iter().enumerate() {
            Display::fmt(x, f)?;
            if i < self.0.len() - 1 {
                f.write_str(", ")?;
            }
        }
        Ok(())
    }
}

impl Display for InputCombination {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for (i, x) in self.0.iter().enumerate() {
            Display::fmt(x, f)?;
            if i < self.0.len() - 1 {
                f.write_str(" + ")?;
            }
        }
        Ok(())
    }
}

impl Display for UnitInput {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            UnitInput::Key(code) => Debug::fmt(code, f),
            UnitInput::Mouse(mb) => Debug::fmt(mb, f),
            UnitInput::WheelUp => {
                write!(f, "Scroll Up")
            }
            UnitInput::WheelDown => {
                write!(f, "Scroll Down")
            }
        }
    }
}

impl Debug for UnitInput {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

impl Debug for InputCombinations {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

impl Debug for InputCombination {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

impl Debug for InputAction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

impl Default for InputMap {
    fn default() -> Self {
        let mut s = Self {
            just_act: Default::default(),
            act: Default::default(),
            input_mapping: Self::default_mapping(),
            wheel: 0.0,
            input_tree: InputTree {
                childs: Default::default(),
                action: None,
            },
        };
        s.build_input_tree();
        s
    }
}

impl InputTree {
    pub fn new(mapping: &FastMap<InputAction, InputCombinations>) -> Self {
        let mut root = Self {
            action: None,
            childs: Default::default(),
        };

        for (act, combs) in mapping {
            log::info!("{} {}", act, combs);
            for comb in &combs.0 {
                let mut cur: &mut InputTree = &mut root;
                for inp in &comb.0 {
                    let ent = cur.childs.entry(*inp);
                    match ent {
                        Entry::Occupied(_) => {}
                        Entry::Vacant(v) => {
                            v.insert(Box::new(InputTree {
                                action: None,
                                childs: Default::default(),
                            }));
                        }
                    }
                    cur = &mut **cur.childs.get_mut(inp).unwrap();
                }
                if cur.action.is_some() {
                    log::error!("two inputs match to the same action, ignoring: {}", act);
                    continue;
                }
                cur.action = Some(*act);
            }
        }

        root
    }

    pub fn query(&self, kb: &KeyboardInfo, mouse: &MouseInfo) -> impl Iterator<Item = InputAction> {
        let mut units: HashSet<UnitInput> =
            HashSet::with_capacity(kb.pressed.len() + mouse.pressed.len() + 1);

        units.extend(kb.pressed.iter().map(|x| UnitInput::Key(*x)));
        units.extend(mouse.pressed.iter().map(|x| UnitInput::Mouse(*x)));
        if mouse.wheel_delta > 0.0 {
            units.insert(UnitInput::WheelUp);
        }
        if mouse.wheel_delta < 0.0 {
            units.insert(UnitInput::WheelDown);
        }

        let mut matches = vec![];
        let mut queue = vec![(vec![], self)];

        while !queue.is_empty() {
            for (input_stack, q) in std::mem::take(&mut queue) {
                for key in units.iter().copied() {
                    if let Some(inp) = q.childs.get(&key) {
                        let mut newstack = input_stack.clone();
                        newstack.push(key);
                        if let Some(x) = inp.action {
                            matches.push((newstack.clone(), x));
                        }
                        queue.push((newstack, &**inp));
                    }
                }
            }
        }

        matches.into_iter().rev().filter_map(move |(inp, act)| {
            if !inp.iter().all(|x| units.contains(x)) {
                return None;
            }
            for v in inp {
                units.remove(&v);
            }
            Some(act)
        })
    }
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
                InputAction::Rotate => "Rotate",
                InputAction::Close => "Close",
                InputAction::Select => "Select",
                InputAction::HideInterface => "Hide interface",
            }
        )
    }
}
