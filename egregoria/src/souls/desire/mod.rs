use imgui::Ui;
use imgui_inspect::{InspectArgsDefault, InspectRenderDefault};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct Desire<T> {
    pub score: f32,
    pub was_max: bool,
    pub v: T,
}

mod buyfood;
mod home;
mod work;

pub use buyfood::*;
pub use home::*;
pub use work::*;

impl<T: InspectRenderDefault<T>> Desire<T> {
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

    register_system!($system_name);
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

        match max_idx {
        $(
            $idx => $t.unwrap().was_max = true,
        )+
        _ => unreachable!(),
        }
    }
    );
}

impl<T: InspectRenderDefault<T>> InspectRenderDefault<Desire<T>> for Desire<T> {
    fn render(data: &[&Desire<T>], label: &'static str, ui: &Ui, args: &InspectArgsDefault) {
        if imgui::CollapsingHeader::new(&*imgui::im_str!("{}", label)).build(ui) {
            ui.indent();
            let v = *unwrap_ret!(data.get(0));
            let mut wasmax = v.was_max;
            #[allow(clippy::indexing_slicing)]
            ui.checkbox(imgui::im_str!("was_max"), &mut wasmax);
            <f32 as InspectRenderDefault<f32>>::render(&[&v.score], "score", ui, args);
            ui.unindent();
            <T as InspectRenderDefault<T>>::render(
                &[&v.v],
                label,
                ui,
                &InspectArgsDefault {
                    header: Some(false),
                    ..*args
                },
            );
            ui.unindent();
        }
    }

    fn render_mut(
        data: &mut [&mut Desire<T>],
        label: &'static str,
        ui: &Ui,
        args: &InspectArgsDefault,
    ) -> bool {
        let v = &mut *unwrap_ret!(data.get_mut(0), false);
        ui.text(format!("{} {}", v.was_max, label));
        ui.text(format!("{} {}", v.score, label));
        <T as InspectRenderDefault<T>>::render_mut(&mut [&mut v.v], label, ui, args)
    }
}
