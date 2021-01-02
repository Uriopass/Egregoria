mod buyfood;
mod home;
mod work;

pub use buyfood::*;
pub use home::*;
pub use work::*;

pub struct Desire<T> {
    pub score: f32,
    pub was_max: bool,
    v: T,
}

impl<T> Desire<T> {
    pub fn new(v: T) -> Self {
        Self {
            score: 0.0,
            was_max: false,
            v,
        }
    }

    pub fn score_and_apply(&mut self, score: impl FnOnce(&T) -> f32, apply: impl FnOnce(&mut T)) {
        if self.was_max {
            apply(&mut self.v);
        }
        self.score = score(&self.v);
    }
}

macro_rules! desires_system {
    ( $system_name: ident, $marker: ty, $($t:tt;$idx: literal)+) => (
    use crate::souls::desire::Desire;
    use legion::system;
    #[system(par_for_each)]
    #[allow(non_snake_case)]
    #[allow(unused_assignments)]
    pub fn $system_name($(_: &$marker, mut $t: Option<&mut Desire<$t>>),+) {
        let mut max_score = f32::NEG_INFINITY;
        let mut max_idx = -1;
        $(
        if let Some(ref mut v) = $t {
          let score = v.score;
          v.was_max = false;
          if score > max_score {
            max_score = score;
            max_idx = $idx;
          }
        }
        )+

        if max_idx == -1 {
            return;
        }

        println!("{} {}", max_idx, max_score);

        match max_idx {
        $(
            $idx => $t.unwrap().was_max = true,
        )+
        _ => unreachable!(),
        }
    }
    );
}
