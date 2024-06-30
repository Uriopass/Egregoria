use common::{FastMap, FastSet};
use engine::{InputContext, Key, MouseButton};
use geom::{Ray3, Vec2, Vec3};
use serde::{Deserialize, Serialize};
use std::collections::hash_map::Entry;
use std::collections::{BTreeMap, HashSet};
use std::fmt::{Debug, Display, Formatter};

// Either combinations can work
#[derive(Serialize, Deserialize)]
pub struct InputCombinations(pub Vec<InputCombination>);

#[derive(Clone, Hash, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub enum UnitInput {
    Key(Key),
    KeyScan(u32),
    Mouse(MouseButton),
    WheelUp,
    WheelDown,
}

#[derive(Clone, Hash, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
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
    SizeUp,
    SizeDown,
    Close,
    Select,
    SecondarySelect,
    NoSnapping,
    HideInterface,
    UpElevation,
    DownElevation,
    OpenEconomyMenu,
    OpenDebugMenu,
    PausePlay,
    OpenChat,
}

// All unit inputs need to match
#[derive(Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
pub struct InputCombination(Vec<UnitInput>);

#[derive(Default)]
struct InputTree {
    actions: Vec<InputAction>,
    childs: FastMap<UnitInput, Box<InputTree>>,
}

#[derive(Default)]
pub struct InputMap {
    /// Actions that were just pressed this frame
    pub just_act: FastSet<InputAction>,
    /// Actions that are currently pressed
    pub act: FastSet<InputAction>,
    /// Mouse wheel delta
    pub wheel: f32,
    /// Mouse position in world space on the terrain (max height 0)
    pub unprojected: Option<Vec3>,

    pub unprojected_normal: Option<Vec3>,
    /// Ray from camera to mouse
    pub ray: Option<Ray3>,
    /// Mouse position in screen space
    pub screen: Vec2,
    input_tree: InputTree,
}

#[derive(Serialize, Deserialize)]
pub struct Bindings(pub BTreeMap<InputAction, InputCombinations>);

use InputAction::*;
use Key as K;
use MouseButton::*;
use UnitInput::*;

// https://stackoverflow.com/a/38068969/5000800 for key scans
#[rustfmt::skip]
const DEFAULT_BINDINGS: &[(InputAction, &[&[UnitInput]])] = &[
    (GoForward,       &[&[KeyScan(17)], &[Key(K::ArrowUp)]]),
    (GoBackward,      &[&[KeyScan(31)], &[Key(K::ArrowDown)]]),
    (GoLeft,          &[&[KeyScan(30)], &[Key(K::ArrowLeft)]]),
    (GoRight,         &[&[KeyScan(32)], &[Key(K::ArrowRight)]]),
    (CameraRotate,    &[&[Mouse(Right)]]),
    (CameraMove,      &[&[Key(K::Shift), Mouse(Right)], &[Mouse(Middle)]]),
    (Zoom,            &[&[Key(K::c("+"))], &[WheelUp]]),
    (Dezoom,          &[&[Key(K::c("-"))], &[WheelDown]]),
    (Rotate,          &[&[Key(K::Control), WheelUp], &[Key(K::Control), WheelDown]]),
    (SizeUp,          &[&[Key(K::Control), WheelUp]]),
    (SizeDown,        &[&[Key(K::Control), WheelDown]]),
    (Close,           &[&[Key(K::Escape)]]),
    (Select,          &[&[Mouse(Left)]]),
    (SecondarySelect, &[&[Key(K::Control), Mouse(Left)]]),
    (NoSnapping,      &[&[Key(K::Control)]]),
    (HideInterface,   &[&[Key(K::c("H"))]]),
    (UpElevation,     &[&[Key(K::Control), WheelUp]]),
    (DownElevation,   &[&[Key(K::Control), WheelDown]]),
    (OpenEconomyMenu, &[&[Key(K::c("E"))]]),
    (OpenDebugMenu,   &[&[Key(K::F3)]]),
    (PausePlay,       &[&[Key(K::Space)]]),
    (OpenChat,        &[&[Key(K::c("T"))]]),
];

impl Default for Bindings {
    fn default() -> Self {
        let mut m = BTreeMap::default();

        for (k, v) in DEFAULT_BINDINGS {
            if m.insert(
                k.clone(),
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
                self.just_act.insert(v.clone());
            }
        }
        self.screen = input.mouse.screen;
        self.wheel = input.mouse.wheel_delta;
    }
}

impl InputCombination {
    pub fn clear(&mut self) {
        self.0.clear();
    }

    /// A valid combination is a list of modifiers and a single key/mouse button
    pub fn is_valid(&self) -> bool {
        let mut has_primary = false;
        let mut has_more_than_one_primary = false;

        for x in &self.0 {
            match x {
                // ignore modifiers
                Key(k) if k.is_modifier() => {}
                Key(_) | Mouse(_) | KeyScan(_) | WheelDown | WheelUp => {
                    if has_primary {
                        has_more_than_one_primary = true;
                    }
                    has_primary = true;
                }
            }
        }

        has_primary && !has_more_than_one_primary
    }

    /// Returns true if the combination is only modifiers
    pub fn is_modifiers_only(&self) -> bool {
        self.0.iter().all(|x| match x {
            Key(k) => k.is_modifier(),
            _ => false,
        })
    }

    /// Pushes a unit if not already present
    pub fn push_unique(&mut self, unit: UnitInput) {
        if self.0.contains(&unit) {
            return;
        }
        self.0.push(unit);
    }

    pub fn sort(&mut self) {
        // put modifiers in front
        self.0.sort();
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
            UnitInput::Key(code) => match code {
                Key::Char(s) => Display::fmt(s, f),
                _ => Debug::fmt(code, f),
            },
            Mouse(mb) => Debug::fmt(mb, f),
            WheelUp => {
                write!(f, "Scroll Up")
            }
            WheelDown => {
                write!(f, "Scroll Down")
            }
            KeyScan(scan) => write!(f, "ScanCode({scan})"),
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
            for comb in &combs.0 {
                let mut cur: &mut InputTree = &mut root;
                for inp in &comb.0 {
                    let ent = cur.childs.entry(inp.clone());
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
                cur.actions.push(act.clone());
            }
        }

        root
    }

    pub fn query(
        &self,
        kb: &FastSet<Key>,
        kb_scans: &FastSet<u32>,
        mouse: &FastSet<MouseButton>,
        wheel: f32,
    ) -> impl Iterator<Item = InputAction> + '_ {
        let mut units: HashSet<UnitInput> = HashSet::with_capacity(kb.len() + mouse.len() + 1);

        units.extend(kb.iter().map(|x| UnitInput::Key(x.clone())));
        units.extend(mouse.iter().map(|x| Mouse(*x)));
        units.extend(kb_scans.iter().map(|x| KeyScan(*x)));
        if wheel > 0.0 {
            units.insert(WheelUp);
        }
        if wheel < 0.0 {
            units.insert(WheelDown);
        }

        let mut matches = vec![];
        let mut queue = vec![(vec![], self)];

        while !queue.is_empty() {
            for (input_stack, q) in std::mem::take(&mut queue) {
                for key in units.iter().cloned() {
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
            .cloned()
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
                SecondarySelect => "Secondary Select",
                HideInterface => "Hide interface",
                NoSnapping => "No Snapping",
                UpElevation => "Up Elevation",
                DownElevation => "Down Elevation",
                OpenEconomyMenu => "Economy Menu",
                PausePlay => "Pause/Play",
                OpenChat => "Interact with Chat",
                SizeUp => "Size Up",
                SizeDown => "Size Down",
                OpenDebugMenu => "Debug Menu",
            }
        )
    }
}
