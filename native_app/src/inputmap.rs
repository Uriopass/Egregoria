use crate::input::{InputContext, KeyCode, MouseButton};
use common::{FastMap, FastSet};
use geom::{Vec2, Vec3};
use std::collections::hash_map::Entry;
use std::collections::{BTreeMap, HashSet};
use std::fmt::{Debug, Display, Formatter};
use winit::event::ScanCode;

// Either combinations can work
#[derive(Serialize, Deserialize)]
pub(crate) struct InputCombinations(pub(crate) Vec<InputCombination>);

#[derive(Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
enum UnitInput {
    Key(KeyCode),
    KeyScan(ScanCode),
    Mouse(MouseButton),
    WheelUp,
    WheelDown,
}

#[derive(Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub(crate) enum InputAction {
    GoLeft,
    GoRight,
    GoForward,
    GoBackward,
    CameraRotate,
    Zoom,
    Dezoom,
    Rotate,
    Close,
    Select,
    NoSnapping,
    HideInterface,
    UpElevation,
    DownElevation,
    OpenEconomyMenu,
    PausePlay,
}

// All unit inputs need to match
#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct InputCombination(Vec<UnitInput>);

#[derive(Default)]
struct InputTree {
    actions: Vec<InputAction>,
    childs: FastMap<UnitInput, Box<InputTree>>,
}

#[derive(Default)]
pub(crate) struct InputMap {
    pub(crate) just_act: FastSet<InputAction>,
    pub(crate) act: FastSet<InputAction>,
    pub(crate) wheel: f32,
    pub(crate) unprojected: Option<Vec3>,
    pub(crate) screen: Vec2,
    input_tree: InputTree,
}

#[derive(Serialize, Deserialize)]
pub struct Bindings(pub(crate) BTreeMap<InputAction, InputCombinations>);

impl Default for Bindings {
    #[rustfmt::skip]
    fn default() -> Self {
        let mut m = BTreeMap::default();
        use InputAction::*;
        use UnitInput::*;
        use MouseButton::*;
        use KeyCode as K;

        macro_rules! ics {
            [$($($v:expr),+);+] => {
                InputCombinations(vec![$(InputCombination(vec![$($v),+])),+])
            }
        }

        // https://stackoverflow.com/a/38068969/5000800
        for (k, v) in vec![
            (GoForward,       ics![KeyScan(17) ; Key(K::Up)]),
            (GoBackward,      ics![KeyScan(31) ; Key(K::Down)]),
            (GoLeft,          ics![KeyScan(30) ; Key(K::Left)]),
            (GoRight,         ics![KeyScan(32) ; Key(K::Right)]),
            (CameraRotate,    ics![Mouse(Right)]),
            (Zoom,            ics![Key(K::Plus) ; WheelUp]),
            (Dezoom,          ics![Key(K::Minus) ; WheelDown]),
            (Rotate,          ics![Key(K::LControl), WheelUp ; Key(K::LControl), WheelDown]),
            (Close,           ics![Key(K::Escape) ; Mouse(Right)]),
            (Select,          ics![Mouse(Left)]),
            (NoSnapping,      ics![Key(K::LControl)]),
            (HideInterface,   ics![Key(K::H)]),
            (UpElevation,     ics![Key(K::LControl), WheelUp]),
            (DownElevation,   ics![Key(K::LControl), WheelDown]),
            (OpenEconomyMenu, ics![Key(K::E)]),
            (PausePlay,       ics![Key(K::Space)]),
        ] {
            if m.insert(k, v).is_some() {
                log::error!("inserting same action twice!");
            }
        }

        Bindings(m)
    }
}

impl InputMap {
    pub fn build_input_tree(&mut self, bindings: &mut Bindings) {
        for v in &mut bindings.0.values_mut() {
            for x in &mut v.0 {
                x.0.sort()
            }
        }
        self.input_tree = InputTree::new(bindings);
    }

    pub(crate) fn prepare_frame(&mut self, input: &InputContext, kb: bool, mouse: bool) {
        self.just_act.clear();
        let empty1 = FastSet::default();
        let empty2 = FastSet::default();
        let empty3 = FastSet::default();
        let mut acts: FastSet<_> = self
            .input_tree
            .query(
                if kb { &input.keyboard.pressed } else { &empty1 },
                if kb {
                    &input.keyboard.pressed_scancode
                } else {
                    &empty2
                },
                if mouse { &input.mouse.pressed } else { &empty3 },
                if mouse { input.mouse.wheel_delta } else { 0.0 },
            )
            .collect();
        std::mem::swap(&mut self.act, &mut acts);
        for v in &self.act {
            if !acts.contains(v) {
                self.just_act.insert(*v);
            }
        }
        self.screen = input.mouse.screen;
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
            UnitInput::KeyScan(scan) => write!(f, "ScanCode({scan})"),
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

impl InputTree {
    pub(crate) fn new(bindings: &Bindings) -> Self {
        let mut root = Self {
            actions: Vec::new(),
            childs: Default::default(),
        };

        for (act, combs) in &bindings.0 {
            log::info!("{} {}", act, combs);
            for comb in &combs.0 {
                let mut cur: &mut InputTree = &mut root;
                for inp in &comb.0 {
                    let ent = cur.childs.entry(*inp);
                    match ent {
                        Entry::Occupied(_) => {}
                        Entry::Vacant(v) => {
                            v.insert(Box::new(InputTree {
                                actions: Vec::new(),
                                childs: Default::default(),
                            }));
                        }
                    }
                    cur = &mut **cur.childs.get_mut(inp).unwrap();
                }
                cur.actions.push(*act);
            }
        }

        root
    }

    pub(crate) fn query(
        &self,
        kb: &FastSet<KeyCode>,
        kb_scans: &FastSet<ScanCode>,
        mouse: &FastSet<MouseButton>,
        wheel: f32,
    ) -> impl Iterator<Item = InputAction> + '_ {
        let mut units: HashSet<UnitInput> = HashSet::with_capacity(kb.len() + mouse.len() + 1);

        units.extend(kb.iter().map(|x| UnitInput::Key(*x)));
        units.extend(mouse.iter().map(|x| UnitInput::Mouse(*x)));
        units.extend(kb_scans.iter().map(|x| UnitInput::KeyScan(*x)));
        if wheel > 0.0 {
            units.insert(UnitInput::WheelUp);
        }
        if wheel < 0.0 {
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
                        if !inp.actions.is_empty() {
                            matches.push((newstack.clone(), &inp.actions));
                        }
                        queue.push((newstack, &**inp));
                    }
                }
            }
        }

        matches
            .into_iter()
            .rev()
            .filter_map(move |(inp, act)| {
                if !inp.iter().all(|x| units.contains(x)) {
                    return None;
                }
                for v in inp {
                    units.remove(&v);
                }
                Some(act)
            })
            .flatten()
            .copied()
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
                InputAction::CameraRotate => "Camera Rotate",
                InputAction::Zoom => "Zoom",
                InputAction::Dezoom => "Dezoom",
                InputAction::Rotate => "Rotate",
                InputAction::Close => "Close",
                InputAction::Select => "Select",
                InputAction::HideInterface => "Hide interface",
                InputAction::NoSnapping => "No Snapping",
                InputAction::UpElevation => "Up Elevation",
                InputAction::DownElevation => "Down Elevation",
                InputAction::OpenEconomyMenu => "Economy Menu",
                InputAction::PausePlay => "Pause/Play",
            }
        )
    }
}
