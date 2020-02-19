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

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct TrafficLightSchedule {
    period: usize,
    green: usize,
    orange: usize,
    red: usize,
    offset: usize,
}

impl TrafficLightSchedule {
    pub fn from_basic(green: usize, orange: usize, red: usize, offset: usize) -> Self {
        Self {
            period: green + orange + red,
            green,
            orange,
            red,
            offset,
        }
    }
}

#[derive(Clone, Copy, Serialize, Deserialize)]
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
                let remainder = (time_seconds as usize + schedule.offset) % schedule.period;
                if remainder < schedule.green {
                    TrafficLightColor::GREEN
                } else if remainder < schedule.green + schedule.orange {
                    TrafficLightColor::ORANGE((schedule.green + schedule.orange - remainder) as f32)
                } else {
                    TrafficLightColor::RED
                }
            }
        }
    }
}
