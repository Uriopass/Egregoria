use crate::rendering::meshrender_component::{MeshRender, RectRender};
use crate::rendering::{Color, BLACK, ORANGE};
use imgui::{im_str, Ui};
use imgui_inspect::{InspectArgsDefault, InspectRenderDefault};
use serde::{Deserialize, Serialize};
use specs::World;

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum TransportKind {
    Car,
    Bus,
}
impl TransportKind {
    pub fn width(self) -> f32 {
        match self {
            TransportKind::Car => 4.5,
            TransportKind::Bus => 9.0,
        }
    }

    pub fn height(self) -> f32 {
        match self {
            TransportKind::Car => 2.0,
            TransportKind::Bus => 2.0,
        }
    }

    pub fn acceleration(self) -> f32 {
        match self {
            TransportKind::Car => 3.0,
            TransportKind::Bus => 2.0,
        }
    }

    pub fn deceleration(self) -> f32 {
        match self {
            TransportKind::Car => 9.0,
            TransportKind::Bus => 9.0,
        }
    }

    pub fn min_turning_radius(self) -> f32 {
        match self {
            TransportKind::Car => 3.0,
            TransportKind::Bus => 5.0,
        }
    }

    pub fn cruising_speed(self) -> f32 {
        match self {
            TransportKind::Car => 15.0,
            TransportKind::Bus => 10.0,
        }
    }

    pub fn ang_acc(self) -> f32 {
        match self {
            TransportKind::Car => 1.0,
            TransportKind::Bus => 0.8,
        }
    }

    pub fn build_mr(self, mr: &mut MeshRender) {
        match self {
            TransportKind::Car => {
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
                    color: BLACK,
                    ..Default::default()
                })
                .add(RectRender {
                    width: 1.0,
                    height: 1.6,
                    offset: [0.8, 0.0].into(),
                    color: BLACK,
                    ..Default::default()
                })
                .add(RectRender {
                    width: 2.7,
                    height: 0.15,
                    offset: [-0.4, 0.85].into(),
                    color: BLACK,
                    ..Default::default()
                })
                .add(RectRender {
                    width: 2.7,
                    height: 0.15,
                    offset: [-0.4, -0.85].into(),
                    color: BLACK,
                    ..Default::default()
                })
                .add(RectRender {
                    width: 0.4,
                    height: 0.15,
                    offset: [2.1, -0.7].into(),
                    color: BLACK,
                    ..Default::default()
                })
                .add(RectRender {
                    width: 0.4,
                    height: 0.15,
                    offset: [2.1, 0.7].into(),
                    color: BLACK,
                    ..Default::default()
                });
            }
            TransportKind::Bus => {
                mr.add(RectRender {
                    width: self.width(),
                    height: self.height(),
                    color: ORANGE,
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

impl InspectRenderDefault<TransportKind> for TransportKind {
    fn render(
        _: &[&TransportKind],
        _: &'static str,
        _: &mut World,
        _: &Ui,
        _: &InspectArgsDefault,
    ) {
        unimplemented!()
    }

    fn render_mut(
        data: &mut [&mut TransportKind],
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
            TransportKind::Car => 0,
            TransportKind::Bus => 1,
        };

        let changed = imgui::ComboBox::new(&im_str!("{}", label)).build_simple_string(
            ui,
            &mut id,
            &[im_str!("Car"), im_str!("Bike")],
        );

        match id {
            0 => **d = TransportKind::Car,
            1 => **d = TransportKind::Bus,
            _ => {}
        }
        changed
    }
}
