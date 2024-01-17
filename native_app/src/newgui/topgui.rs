use yakui::{Alignment, Color, Dim2, Vec2};

use simulation::map_dynamic::ElectricityFlow;
use simulation::Simulation;

use crate::gui::{Gui, UiTextures};
use crate::uiworld::UiWorld;

impl Gui {
    /// Root GUI entrypoint
    pub fn render_newgui(&mut self, uiworld: &mut UiWorld, sim: &Simulation) {
        profiling::scope!("topgui::render");
        self.auto_save(uiworld);

        if self.hidden {
            return;
        }

        yakui::column(|| {
            self.power_errors(uiworld, sim);
        });
    }

    fn power_errors(&mut self, uiworld: &UiWorld, sim: &Simulation) {
        profiling::scope!("topgui::power_errors");
        let map = sim.map();
        let flow = sim.read::<ElectricityFlow>();

        let no_power_img = uiworld.read::<UiTextures>().get_yakui("no_power");

        for network in map.electricity.networks() {
            let prod = flow.productivity(network.id);

            if prod >= 1.0 {
                continue;
            }
            for &building in &network.buildings {
                let Some(b) = map.get(building) else {
                    continue;
                };

                let center = b.obb.center();

                let pos = center.z(b.height
                    + 20.0
                    + 1.0 * f32::cos(uiworld.time_always() + center.mag() * 0.05));
                let (screenpos, depth) = uiworld.camera().project(pos);

                let size = 10000.0 / depth;

                yakui::reflow(
                    Alignment::TOP_LEFT,
                    Dim2::pixels(screenpos.x - size * 0.5, screenpos.y - size * 0.5),
                    || {
                        let mut image =
                            yakui::widgets::Image::new(no_power_img, Vec2::new(size, size));
                        image.color = Color::WHITE.with_alpha(0.5);
                        image.show();
                    },
                );
            }
        }
    }
}
