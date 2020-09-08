use crate::{IntersectionID, LaneID, Lanes, Map, TurnID};
use geom::PolyLine;
use imgui_inspect::imgui;
use imgui_inspect_derive::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub enum TraverseDirection {
    Forward,
    Backward,
}

#[derive(Clone, Copy, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub enum TraverseKind {
    Lane(LaneID),
    Turn(TurnID),
}

impl TraverseKind {
    pub fn is_lane(&self) -> bool {
        matches!(self, TraverseKind::Lane(_))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize, Inspect)]
pub struct Traversable {
    pub kind: TraverseKind,
    pub dir: TraverseDirection,
}

impl Traversable {
    pub fn new(kind: TraverseKind, dir: TraverseDirection) -> Self {
        Self { kind, dir }
    }

    pub fn points(&self, m: &Map) -> Option<PolyLine> {
        let p = self.raw_points(m)?;

        match self.dir {
            TraverseDirection::Forward => Some(p.clone()),
            TraverseDirection::Backward => Some(PolyLine::new(p.iter().copied().rev().collect())),
        }
    }

    pub fn raw_points<'a>(&self, m: &'a Map) -> Option<&'a PolyLine> {
        match self.kind {
            TraverseKind::Lane(id) => Some(&m.lanes.get(id)?.points),
            TraverseKind::Turn(id) => Some(&m.intersections.get(id.parent)?.find_turn(id)?.points),
        }
    }

    pub fn can_pass(&self, time: u64, lanes: &Lanes) -> bool {
        match self.kind {
            TraverseKind::Lane(id) => !lanes[id].control.get_behavior(time).is_red(),
            TraverseKind::Turn(_) => true,
        }
    }

    pub fn destination_intersection(&self, lanes: &Lanes) -> IntersectionID {
        match self.kind {
            TraverseKind::Lane(p) => match self.dir {
                TraverseDirection::Forward => lanes[p].dst,
                TraverseDirection::Backward => lanes[p].src,
            },
            TraverseKind::Turn(id) => id.parent,
        }
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
            fn render(data: &[&$t], label: &'static str, ui: &imgui::Ui, _: &imgui_inspect::InspectArgsDefault,
            ) {
                if data.len() != 1 {
                    unimplemented!()
                }
                let d = &data[0];
                let mut aha = "No match";
                $(
                    if let $x = d {
                        aha = stringify!($x);
                    }
                )+

                ui.text(imgui::im_str!("{} {}", &aha, label));
            }

            fn render_mut(
                data: &mut [&mut $t],
                label: &'static str,
                ui: &imgui::Ui,
                _: &imgui_inspect::InspectArgsDefault,
            ) -> bool {
                if data.len() != 1 {
                    unimplemented!()
                }
                let d = &mut data[0];
                let mut aha = "No match";
                $(
                    if let $x = d {
                        aha = stringify!($x);
                    }
                )+

                ui.text(imgui::im_str!("{} {}", &aha, label));
                false
            }
        }
    };
}

enum_inspect_impl!(TraverseKind; TraverseKind::Lane(_), TraverseKind::Turn(_));
enum_inspect_impl!(TraverseDirection; TraverseDirection::Forward, TraverseDirection::Backward);
