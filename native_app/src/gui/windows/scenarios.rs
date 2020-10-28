use crate::gui::windows::ImguiWindow;
use egregoria::Egregoria;
use imgui::{im_str, Ui};

fn available_scenarios() -> Vec<String> {
    let mut available_scenarios = vec![];
    for file in std::fs::read_dir("lua/scenarios")
        .into_iter()
        .flatten()
        .filter_map(|x| x.ok())
    {
        available_scenarios.push(file.file_name().to_string_lossy().into_owned());
    }
    available_scenarios
}

pub struct Scenarios {
    available_scenarios: Vec<String>,
}

impl Default for Scenarios {
    fn default() -> Self {
        Self {
            available_scenarios: available_scenarios(),
        }
    }
}

impl ImguiWindow for Scenarios {
    fn render(&mut self, ui: &Ui, goria: &mut Egregoria) {
        for scenario in self.available_scenarios.iter() {
            if ui.small_button(&im_str!("{}", scenario)) {
                egregoria::scenarios::scenario_runner::set_scenario(
                    goria,
                    &format!("lua/scenarios/{}", scenario),
                );
            }
        }
        if ui.small_button(im_str!("reload scenario list")) {
            self.available_scenarios = available_scenarios();
        }
    }
}
