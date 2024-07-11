use yakui::widgets::Layer;
use yakui::{center, reflow, Alignment, Dim2, Pivot};

use engine::InputContext;
use goryak::{blur_bg, constrained_viewport, mincolumn, on_secondary, primary, textc, titlec};
use simulation::Simulation;

use crate::inputmap::{Bindings, InputAction, InputCombination, InputMap, UnitInput};
use crate::uiworld::UiWorld;

#[derive(Default)]
pub struct KeybindState {
    pub enabled: Option<KeybindStateInner>,
}

pub struct KeybindStateInner {
    pub to_bind_to: InputAction,
    pub bind_index: usize,
    pub cur: InputCombination,
}

pub fn keybind_modal(uiw: &UiWorld, _: &Simulation) {
    profiling::scope!("hud::keybind_modal");

    let mut state = uiw.write::<KeybindState>();
    let Some(state) = &mut state.enabled else {
        return;
    };

    Layer::new().show(|| {
        reflow(
            Alignment::TOP_LEFT,
            Pivot::TOP_LEFT,
            Dim2::pixels(0.0, 0.0),
            || {
                blur_bg(primary().with_alpha(0.5), 0.0, || {
                    constrained_viewport(|| {
                        center(|| {
                            mincolumn(10.0, || {
                                titlec(on_secondary(), format!("{}", state.to_bind_to));
                                textc(on_secondary(), "Press key/mouse to bind to action");
                            });
                        });
                    });
                })
            },
        );
    });
}

impl KeybindState {
    pub fn update(
        &mut self,
        bindings: &mut Bindings,
        input_map: &mut InputMap,
        inp: &InputContext,
    ) {
        let Some(state) = &mut self.enabled else {
            return;
        };

        for key in &inp.keyboard.pressed {
            state.cur.push_unique(UnitInput::Key(key.clone()));
        }
        for &mouse in &inp.mouse.pressed {
            state.cur.push_unique(UnitInput::Mouse(mouse));
        }
        if inp.mouse.wheel_delta > 0.0 {
            state.cur.push_unique(UnitInput::WheelUp);
        }
        if inp.mouse.wheel_delta < 0.0 {
            state.cur.push_unique(UnitInput::WheelDown);
        }

        if state.cur.is_modifiers_only() {
            return;
        }

        if !state.cur.is_valid() {
            state.cur.clear();
            return;
        }

        let comb = &mut bindings.0.get_mut(&state.to_bind_to).unwrap().0;

        let mut cur = std::mem::take(&mut state.cur);
        cur.sort();
        if state.bind_index < comb.len() {
            comb[state.bind_index] = cur;
        } else {
            comb.push(cur);
        }
        comb.dedup_by(|a, b| a == b);

        input_map.build_input_tree(bindings);

        self.enabled = None;
    }
}
