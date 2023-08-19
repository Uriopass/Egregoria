use common::{FastMap, FastSet};
use geom::{Vec2, Vec3};
use std::collections::hash_map::Entry;
use std::collections::{BTreeMap, HashSet};
use std::fmt::{Debug, Display, Formatter};
use wgpu_engine::{InputContext, KeyCode, MouseButton};
use winit::event::ScanCode;

// Either combinations can work
#[derive(Serialize, Deserialize)]
pub struct InputCombinations(pub Vec<InputCombination>);

#[derive(Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
enum UnitInput {
    Key(KeyCode),
    KeyScan(ScanCode),
    Mouse(MouseButton),
    WheelUp,
    WheelDown,
}

#[derive(Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
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
    NoSnapping,
    HideInterface,
    UpElevation,
    DownElevation,
    OpenEconomyMenu,
    PausePlay,
    OpenChat,
}

// All unit inputs need to match
#[derive(Serialize, Deserialize, Clone)]
pub struct InputCombination(Vec<UnitInput>);

#[derive(Default)]
struct InputTree {
    actions: Vec<InputAction>,
    childs: FastMap<UnitInput, Box<InputTree>>,
}

#[derive(Default)]
pub struct InputMap {
    pub just_act: FastSet<InputAction>,
    pub act: FastSet<InputAction>,
    pub wheel: f32,
    pub unprojected: Option<Vec3>,
    pub screen: Vec2,
    input_tree: InputTree,
}

#[derive(Serialize, Deserialize)]
pub struct Bindings(pub BTreeMap<InputAction, InputCombinations>);

use InputAction::*;
use KeyCode as K;
use MouseButton::*;
use UnitInput::*;

// https://stackoverflow.com/a/38068969/5000800 for key scans
#[rustfmt::skip]
const DEFAULT_BINDINGS: &[(InputAction, &[&[UnitInput]])] = &[
    (GoForward,       &[&[KeyScan(17)], &[Key(K::Up)]]),
    (GoBackward,      &[&[KeyScan(31)], &[Key(K::Down)]]),
    (GoLeft,          &[&[KeyScan(30)], &[Key(K::Left)]]),
    (GoRight,         &[&[KeyScan(32)], &[Key(K::Right)]]),
    (CameraRotate,    &[&[Mouse(Right)]]),
    (CameraMove,      &[&[Key(K::LShift), Mouse(Right)], &[Mouse(Middle)]]),
    (Zoom,            &[&[Key(K::Plus)], &[WheelUp]]),
    (Dezoom,          &[&[Key(K::Minus)], &[WheelDown]]),
    (Rotate,          &[&[Key(K::LControl), WheelUp], &[Key(K::LControl), WheelDown]]),
    (Close,           &[&[Key(K::Escape)]]),
    (Select,          &[&[Mouse(Left)]]),
    (NoSnapping,      &[&[Key(K::LControl)]]),
    (HideInterface,   &[&[Key(K::H)]]),
    (UpElevation,     &[&[Key(K::LControl), WheelUp]]),
    (DownElevation,   &[&[Key(K::LControl), WheelDown]]),
    (OpenEconomyMenu, &[&[Key(K::E)]]),
    (PausePlay,       &[&[Key(K::Space)]]),
    (OpenChat, &[&[Key(K::T)]]),
];

impl Default for Bindings {
    fn default() -> Self {
        let mut m = BTreeMap::default();

        for &(k, v) in DEFAULT_BINDINGS {
            if m.insert(
                k,
                InputCombinations(v.iter().map(|&x| InputCombination(x.to_vec())).collect()),
            )
            .is_some()
            {
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

    pub fn prepare_frame(&mut self, input: &InputContext, kb: bool, mouse: bool) {
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
    pub fn new(bindings: &Bindings) -> Self {
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

    pub fn query(
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
                GoLeft => "Go Left",
                GoRight => "Go Right",
                GoForward => "Go Forward",
                GoBackward => "Go Backward",
                CameraMove => "Camera Move",
                CameraRotate => "Camera Rotate",
                Zoom => "Zoom",
                Dezoom => "Dezoom",
                Rotate => "Rotate",
                Close => "Close",
                Select => "Select",
                HideInterface => "Hide interface",
                NoSnapping => "No Snapping",
                UpElevation => "Up Elevation",
                DownElevation => "Down Elevation",
                OpenEconomyMenu => "Economy Menu",
                PausePlay => "Pause/Play",
                OpenChat => "Interact with Chat",
            }
        )
    }
}
