use crate::geometry::polyline::PolyLine;
use crate::gui::{InspectDragf, InspectVec2};
use crate::map_model::{Map, Traversable};
use crate::rendering::meshrender_component::{MeshRender, RectRender};
use crate::rendering::Color;
use cgmath::{vec2, Vector2};
use imgui::{im_str, Ui};
use imgui_inspect::{InspectArgsDefault, InspectRenderDefault};
use imgui_inspect_derive::*;
use serde::{Deserialize, Serialize};
use specs::{Component, DenseVecStorage, World};

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum VehicleKind {
    Car,
    Bus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VehicleObjective {
    None,
    Temporary(Traversable),
}

#[derive(Component, Debug, Inspect, Clone, Serialize, Deserialize)]
pub struct VehicleComponent {
    pub objective: VehicleObjective,
    pub pos_objective: PolyLine,
    #[inspect(proxy_type = "InspectDragf")]
    pub desired_speed: f32,
    #[inspect(proxy_type = "InspectVec2")]
    pub desired_dir: Vector2<f32>,
    #[inspect(proxy_type = "InspectDragf")]
    pub ang_velocity: f32,
    #[inspect(proxy_type = "InspectDragf")]
    pub wait_time: f32,

    pub kind: VehicleKind,
}

impl VehicleKind {
    pub fn width(self) -> f32 {
        match self {
            VehicleKind::Car => 4.5,
            VehicleKind::Bus => 9.0,
        }
    }

    pub fn height(self) -> f32 {
        match self {
            VehicleKind::Car => 2.0,
            VehicleKind::Bus => 2.0,
        }
    }

    pub fn acceleration(self) -> f32 {
        match self {
            VehicleKind::Car => 3.0,
            VehicleKind::Bus => 2.0,
        }
    }

    pub fn deceleration(self) -> f32 {
        match self {
            VehicleKind::Car => 9.0,
            VehicleKind::Bus => 9.0,
        }
    }

    pub fn min_turning_radius(self) -> f32 {
        match self {
            VehicleKind::Car => 3.0,
            VehicleKind::Bus => 5.0,
        }
    }

    pub fn cruising_speed(self) -> f32 {
        match self {
            VehicleKind::Car => 15.0,
            VehicleKind::Bus => 10.0,
        }
    }

    pub fn ang_acc(self) -> f32 {
        match self {
            VehicleKind::Car => 1.0,
            VehicleKind::Bus => 0.8,
        }
    }

    pub fn build_mr(self, mr: &mut MeshRender) {
        match self {
            VehicleKind::Car => {
                mr.add(RectRender {
                    width: self.width(),
                    height: self.height(),
                    color: get_random_car_color(),
                    ..Default::default()
                })
                .add(RectRender {
                    width: 0.4,
                    height: 1.8,
                    offset: [-1.7, 0.0].into(),
                    color: Color::BLACK,
                    ..Default::default()
                })
                .add(RectRender {
                    width: 1.0,
                    height: 1.6,
                    offset: [0.8, 0.0].into(),
                    color: Color::BLACK,
                    ..Default::default()
                })
                .add(RectRender {
                    width: 2.7,
                    height: 0.15,
                    offset: [-0.4, 0.85].into(),
                    color: Color::BLACK,
                    ..Default::default()
                })
                .add(RectRender {
                    width: 2.7,
                    height: 0.15,
                    offset: [-0.4, -0.85].into(),
                    color: Color::BLACK,
                    ..Default::default()
                })
                .add(RectRender {
                    width: 0.4,
                    height: 0.15,
                    offset: [2.1, -0.7].into(),
                    color: Color::BLACK,
                    ..Default::default()
                })
                .add(RectRender {
                    width: 0.4,
                    height: 0.15,
                    offset: [2.1, 0.7].into(),
                    color: Color::BLACK,
                    ..Default::default()
                });
            }
            VehicleKind::Bus => {
                mr.add(RectRender {
                    width: self.width(),
                    height: self.height(),
                    color: Color::ORANGE,
                    ..Default::default()
                });
            }
        }
    }
}

pub fn get_random_car_color() -> Color {
    let car_colors: [(Color, f32); 9] = [
        (Color::from_hex(0x22_22_22), 0.22),  // Black
        (Color::from_hex(0xff_ff_ff), 0.19),  // White
        (Color::from_hex(0x66_66_66), 0.17),  // Gray
        (Color::from_hex(0xb8_b8_b8), 0.14),  // Silver
        (Color::from_hex(0x1a_3c_70), 0.1),   // Blue
        (Color::from_hex(0xd8_22_00), 0.1),   // Red
        (Color::from_hex(0x7c_4b_24), 0.02),  // Brown
        (Color::from_hex(0xd4_c6_78), 0.015), // Gold
        (Color::from_hex(0x72_cb_19), 0.015), // Green
    ];

    let total: f32 = car_colors.iter().map(|x| x.1).sum();

    let r = rand::random::<f32>() * total;
    let mut partial = 0.0;
    for (col, freq) in &car_colors {
        partial += freq;
        if partial >= r {
            return *col;
        }
    }
    unreachable!();
}

impl Default for VehicleComponent {
    fn default() -> Self {
        Self {
            objective: VehicleObjective::None,
            desired_speed: 0.0,
            desired_dir: vec2(0.0, 0.0),
            wait_time: 0.0,
            ang_velocity: 0.0,
            pos_objective: PolyLine::with_capacity(7),
            kind: VehicleKind::Car,
        }
    }
}

impl VehicleComponent {
    pub fn new(objective: VehicleObjective, kind: VehicleKind) -> VehicleComponent {
        Self {
            objective,
            kind,
            ..Default::default()
        }
    }

    pub fn set_travers_objective(&mut self, travers: Traversable, map: &Map) {
        self.objective = VehicleObjective::Temporary(travers);
        let p = travers.points(map);
        self.pos_objective.extend(p.iter().rev());
    }
}

impl<'a> InspectRenderDefault<VehicleObjective> for VehicleObjective {
    fn render(
        _: &[&VehicleObjective],
        _: &'static str,
        _: &mut World,
        _: &Ui,
        _: &InspectArgsDefault,
    ) {
        unimplemented!();
    }

    fn render_mut(
        data: &mut [&mut VehicleObjective],
        label: &'static str,
        _: &mut World,
        ui: &Ui,
        _: &InspectArgsDefault,
    ) -> bool {
        if data.len() != 1 {
            return false;
        }

        let obj = &data[0];
        match obj {
            VehicleObjective::None => ui.text(im_str!("None {}", label)),
            VehicleObjective::Temporary(x) => ui.text(im_str!("{:?} {}", x, label)),
        }

        false
    }
}

impl InspectRenderDefault<VehicleKind> for VehicleKind {
    fn render(_: &[&VehicleKind], _: &'static str, _: &mut World, _: &Ui, _: &InspectArgsDefault) {
        unimplemented!()
    }

    fn render_mut(
        data: &mut [&mut VehicleKind],
        label: &'static str,
        _: &mut World,
        ui: &Ui,
        _: &InspectArgsDefault,
    ) -> bool {
        if data.len() != 1 {
            unimplemented!()
        }
        let d = &mut data[0];
        let mut id = match d {
            VehicleKind::Car => 0,
            VehicleKind::Bus => 1,
        };

        let changed = imgui::ComboBox::new(&im_str!("{}", label)).build_simple_string(
            ui,
            &mut id,
            &[im_str!("Car"), im_str!("Bus")],
        );

        match id {
            0 => **d = VehicleKind::Car,
            1 => **d = VehicleKind::Bus,
            _ => {}
        }
        changed
    }
}
