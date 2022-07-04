use crate::map::{IntersectionID, Intersections, LaneID, Lanes, Map, TurnID};
use geom::PolyLine3;
use imgui_inspect::imgui::Ui;
use imgui_inspect::{imgui, InspectArgsDefault, InspectRenderDefault};
use imgui_inspect_derive::Inspect;
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum TraverseDirection {
    Forward,
    Backward,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Hash)]
pub enum TraverseKind {
    Lane(LaneID),
    Turn(TurnID),
}

impl TraverseKind {
    pub fn is_lane(&self) -> bool {
        matches!(self, TraverseKind::Lane(_))
    }
    pub fn length(&self, lanes: &Lanes, inters: &Intersections) -> Option<f32> {
        Some(match *self {
            TraverseKind::Lane(i) => lanes.get(i)?.points.length(),
            TraverseKind::Turn(t) => inters.get(t.parent)?.find_turn(t)?.points.length(),
        })
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Hash, Inspect)]
pub struct Traversable {
    pub kind: TraverseKind,
    pub dir: TraverseDirection,
}

impl Traversable {
    pub fn new(kind: TraverseKind, dir: TraverseDirection) -> Self {
        Self { kind, dir }
    }

    pub fn points(&self, m: &Map) -> Option<PolyLine3> {
        let mut p = self.raw_points(m)?.clone();

        if let TraverseDirection::Backward = self.dir {
            p.reverse();
        }
        Some(p)
    }

    pub fn raw_points<'a>(&self, m: &'a Map) -> Option<&'a PolyLine3> {
        match self.kind {
            TraverseKind::Lane(id) => Some(&m.lanes.get(id)?.points),
            TraverseKind::Turn(id) => Some(&m.intersections.get(id.parent)?.find_turn(id)?.points),
        }
    }

    pub fn can_pass(&self, time: u32, lanes: &Lanes) -> bool {
        match self.kind {
            TraverseKind::Lane(id) => {
                let l = unwrap_or!(lanes.get(id), return true);
                !l.control.get_behavior(time).is_red()
            }
            TraverseKind::Turn(_) => true,
        }
    }

    pub fn destination_intersection(&self, lanes: &Lanes) -> Option<IntersectionID> {
        Some(match self.kind {
            TraverseKind::Lane(p) => match self.dir {
                TraverseDirection::Forward => lanes.get(p)?.dst,
                TraverseDirection::Backward => lanes.get(p)?.src,
            },
            TraverseKind::Turn(id) => id.parent,
        })
    }

    pub fn destination_lane(&self) -> LaneID {
        match self.kind {
            TraverseKind::Lane(p) => p,
            TraverseKind::Turn(t) => match self.dir {
                TraverseDirection::Forward => t.dst,
                TraverseDirection::Backward => t.src,
            },
        }
    }
}

macro_rules! enum_inspect_impl {
    ($t: ty; $($x: pat),+) => {
        impl imgui_inspect::InspectRenderDefault<$t> for $t {
            fn render(data: &[&$t], label: &'static str, ui: &imgui::Ui<'_>, _: &imgui_inspect::InspectArgsDefault,
            ) {
                if data.len() != 1 {
                    unimplemented!()
                }
                let d = unwrap_ret!(data.get(0));
                let mut aha = "No match";
                $(
                    if let $x = d {
                        aha = stringify!($x);
                    }
                )+

                ui.text(format!("{} {}", &aha, label));
            }

            fn render_mut(
                data: &mut [&mut $t],
                label: &'static str,
                ui: &imgui::Ui<'_>,
                _: &imgui_inspect::InspectArgsDefault,
            ) -> bool {
                if data.len() != 1 {
                    unimplemented!()
                }
                let d = unwrap_ret!(data.get_mut(0), false);
                let mut aha = "No match";
                $(
                    if let $x = d {
                        aha = stringify!($x);
                    }
                )+

                ui.text(format!("{} {}", &aha, label));
                false
            }
        }
    };
}

impl InspectRenderDefault<TraverseKind> for TraverseKind {
    fn render(
        data: &[&TraverseKind],
        label: &'static str,
        ui: &Ui<'_>,
        _args: &InspectArgsDefault,
    ) {
        if data.len() != 1 {
            panic!("not implemented")
        }
        let d = match data.get(0) {
            Some(x) => x,
            None => return,
        };
        match d {
            TraverseKind::Lane(l) => {
                ui.text(format!("TraverseKind::Lane({:?}): {}", l, label));
            }
            TraverseKind::Turn(v) => {
                ui.text(format!("TraverseKind::Turn({:?}): {}", v, label));
            }
        }
    }

    fn render_mut(
        data: &mut [&mut TraverseKind],
        label: &'static str,
        ui: &Ui<'_>,
        _args: &InspectArgsDefault,
    ) -> bool {
        if data.len() != 1 {
            panic!("not implemented")
        }
        let d = match data.get(0) {
            Some(x) => x,
            None => return false,
        };
        match d {
            TraverseKind::Lane(l) => {
                ui.text(format!("TraverseKind::Lane({:?}): {}", l, label));
            }
            TraverseKind::Turn(v) => {
                ui.text(format!("TraverseKind::Turn({:?}): {}", v, label));
            }
        }
        false
    }
}

enum_inspect_impl!(TraverseDirection; TraverseDirection::Forward, TraverseDirection::Backward);
