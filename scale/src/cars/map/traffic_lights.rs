use crate::rendering::{Color, GREEN, ORANGE, RED};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Serialize, Deserialize)]
pub enum TrafficLightColor {
    RED,
    ORANGE(f32),
    GREEN,
}

impl TrafficLightColor {
    pub fn as_render_color(self) -> Color {
        match self {
            TrafficLightColor::RED => RED,
            TrafficLightColor::ORANGE(_) => ORANGE,
            TrafficLightColor::GREEN => GREEN,
        }
    }

    pub fn is_red(self) -> bool {
        match self {
            TrafficLightColor::RED => true,
            _ => false,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct TrafficLightSchedule {
    period: u64,
    lights: Vec<TrafficLightColor>,
}

impl TrafficLightSchedule {
    pub fn from_basic(green: usize, orange: usize, red: usize, offset: usize) -> Self {
        let period = (green + orange + red) as u64;
        let mut i = orange;
        let mut lights = std::iter::repeat(TrafficLightColor::GREEN)
            .take(green)
            .chain(
                std::iter::repeat_with(|| {
                    i -= 1;
                    TrafficLightColor::ORANGE(i as f32)
                })
                .take(orange),
            )
            .chain(std::iter::repeat(TrafficLightColor::RED).take(red))
            .collect::<Vec<TrafficLightColor>>();
        lights.rotate_right(offset);
        assert_eq!(lights.len(), period as usize);

        Self { lights, period }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub enum TrafficLight {
    Always,
    Periodic(TrafficLightSchedule),
}

impl TrafficLight {
    pub fn is_always(&self) -> bool {
        match self {
            TrafficLight::Always => true,
            _ => false,
        }
    }

    pub fn get_color(&self, time_seconds: u64) -> TrafficLightColor {
        match self {
            TrafficLight::Always => TrafficLightColor::GREEN,
            TrafficLight::Periodic(schedule) => {
                let remainder = (time_seconds % schedule.period) as usize;
                schedule.lights[remainder]
            }
        }
    }
}
