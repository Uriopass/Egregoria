use common::descriptions::{BuildingGen, CompanyKind};
use egui::{Color32, Ui};
use egui_dock::{
    DockArea, DockState, NodeIndex, Style, TabBodyStyle, TabInteractionStyle, TabStyle,
};
use egui_inspect::{Inspect, InspectArgs};

use engine::meshload::MeshProperties;
use engine::wgpu::RenderPass;
use engine::{Drawable, GfxContext, Mesh, SpriteBatch};
use geom::{Matrix4, Vec2};

use crate::companies::Companies;
use crate::State;

#[derive(Copy, Clone, Debug)]
pub enum Tab {
    View,
    Explorer,
    Properties,
    ModelProperties,
}

#[derive(PartialEq, Eq, Copy, Clone)]
pub enum Inspected {
    None,
    Company(usize),
}

#[derive(Clone)]
pub enum Shown {
    None,
    Error(String),
    Model((Mesh, MeshProperties)),
    Sprite(SpriteBatch),
}

pub struct Gui {
    pub tree: Option<DockState<Tab>>,
    pub companies: Companies,
    pub inspected: Inspected,
    pub shown: Shown,
}

impl Gui {
    pub fn new() -> Self {
        let mut tree = DockState::new(vec![Tab::View]);

        let view = NodeIndex::root();
        let [view, _] = tree
            .main_surface_mut()
            .split_left(view, 0.2, vec![Tab::Explorer]);
        let [view, _] = tree
            .main_surface_mut()
            .split_right(view, 0.8, vec![Tab::Properties]);
        tree.main_surface_mut()
            .split_below(view, 0.8, vec![Tab::ModelProperties]);

        Self {
            tree: Some(tree),
            companies: Companies::new().expect("could not load companies.json"),
            inspected: Inspected::None,
            shown: Shown::None,
        }
    }
}

impl State {
    pub fn gui(&mut self, ui: &egui::Context) {
        let mut tree = self.gui.tree.take().unwrap();
        DockArea::new(&mut tree)
            .show_close_buttons(false)
            .draggable_tabs(false)
            .style(Style::from_egui(ui.style().as_ref()))
            .show(ui, &mut TabViewer { state: self });
        self.gui.tree = Some(tree);
    }
}

fn explorer(state: &mut State, ui: &mut Ui) {
    if state.gui.companies.changed {
        if ui.button("Save").clicked() {
            state.gui.companies.save();
        }
    }
    for (i, comp) in state.gui.companies.companies.iter().enumerate() {
        let r = ui.add_sized([ui.available_width(), 40.0], egui::Button::new(&comp.name));
        if r.clicked() {
            state.gui.inspected = Inspected::Company(i);
        }
    }
}

fn properties(state: &mut State, ui: &mut Ui) {
    match state.gui.inspected {
        Inspected::None => {}
        Inspected::Company(i) => {
            let comp = &mut state.gui.companies.companies[i];
            text_inp(ui, "name", &mut comp.name);

            let mut selected = match comp.kind {
                CompanyKind::Store => 0,
                CompanyKind::Factory { .. } => 1,
                CompanyKind::Network => 2,
            };

            if egui::ComboBox::new("company_kind", "company kind")
                .show_index(ui, &mut selected, 3, |idx| match idx {
                    0 => "Store",
                    1 => "Factory",
                    2 => "Network",
                    _ => unreachable!(),
                })
                .changed()
            {
                match selected {
                    0 => comp.kind = CompanyKind::Store,
                    1 => comp.kind = CompanyKind::Factory { n_trucks: 1 },
                    2 => comp.kind = CompanyKind::Network,
                    _ => unreachable!(),
                }
            }

            match &mut comp.kind {
                CompanyKind::Store | CompanyKind::Network => {}
                CompanyKind::Factory { n_trucks } => {
                    inspect(ui, "n_trucks", n_trucks);
                }
            }

            let mut selected = match comp.bgen {
                BuildingGen::House => unreachable!(),
                BuildingGen::Farm => 0,
                BuildingGen::CenteredDoor { .. } => 1,
                BuildingGen::NoWalkway { .. } => 2,
            };
            if egui::ComboBox::new("bgen_kind", "bgen kind")
                .show_index(ui, &mut selected, 3, |idx| match idx {
                    0 => "Farm",
                    1 => "Centered door",
                    2 => "No walkway",
                    _ => unreachable!(),
                })
                .changed()
            {
                match selected {
                    0 => comp.bgen = BuildingGen::Farm,
                    1 => {
                        comp.bgen = BuildingGen::CenteredDoor {
                            vertical_factor: 1.0,
                        }
                    }
                    2 => {
                        comp.bgen = BuildingGen::NoWalkway {
                            door_pos: Vec2::ZERO,
                        }
                    }
                    _ => unreachable!(),
                }
            }

            match &mut comp.bgen {
                BuildingGen::House | BuildingGen::Farm => {}
                BuildingGen::CenteredDoor { vertical_factor } => {
                    inspect(ui, "vertical factor", vertical_factor);
                }
                BuildingGen::NoWalkway { door_pos } => {
                    inspect(ui, "door_pos", door_pos);
                }
            }

            ui.add_space(5.0);
            ui.label("Recipe");
            inspect(ui, "complexity", &mut comp.recipe.complexity);
            inspect(
                ui,
                "storage_multiplier",
                &mut comp.recipe.storage_multiplier,
            );
            ui.label("consumption");
            ui.indent("consumption", |ui| {
                for (name, amount) in comp.recipe.consumption.iter_mut() {
                    inspect_item(ui, name, amount);
                }
            });

            ui.label("production");
            ui.indent("production", |ui| {
                for (name, amount) in comp.recipe.production.iter_mut() {
                    inspect_item(ui, name, amount);
                }
            });

            inspect(ui, "n_workers", &mut comp.n_workers);
            inspect(ui, "size", &mut comp.size);
            text_inp(ui, "asset_location", &mut comp.asset_location);
            inspect(ui, "price", &mut comp.price);

            ui_opt(ui, "zone", &mut comp.zone, |ui, zone| {
                ui.indent("zone", |ui| {
                    inspect(ui, "floor", &mut zone.floor);
                    inspect(ui, "filler", &mut zone.filler);
                    inspect(ui, "price_per_area", &mut zone.price_per_area);
                });
            });
        }
    }
}

fn inspect_item(ui: &mut Ui, name: &mut String, amount: &mut i32) {
    ui.horizontal(|ui| {
        ui.label(&*name);
        ui.add(egui::DragValue::new(amount));
    });
}

fn inspect<T: Inspect<T>>(ui: &mut Ui, label: &'static str, x: &mut T) -> bool {
    <T as Inspect<T>>::render_mut(x, label, ui, &InspectArgs::default())
}

fn text_inp(ui: &mut Ui, label: &'static str, v: &mut String) {
    ui.horizontal(|ui| {
        ui.label(label);
        ui.text_edit_singleline(v);
    });
}

fn ui_opt<T: Default>(
    ui: &mut Ui,
    label: &'static str,
    v: &mut Option<T>,
    f: impl FnOnce(&mut Ui, &mut T),
) {
    ui.horizontal(|ui| {
        let mut is_some = v.is_some();
        ui.checkbox(&mut is_some, label);
        if is_some != v.is_some() {
            if is_some {
                *v = Some(Default::default());
            } else {
                *v = None;
            }
        }
    });
    if let Some(v) = v {
        f(ui, v);
    }
}

fn model_properties(state: &mut State, ui: &mut Ui) {
    match &state.gui.shown {
        Shown::None => {}
        Shown::Error(e) => {
            ui.label(e);
        }
        Shown::Model((_, props)) => {
            ui.columns(2, |ui| {
                ui[0].label("Vertices");
                ui[1].label(format!("{}", props.n_vertices));

                ui[0].label("Triangles");
                ui[1].label(format!("{}", props.n_triangles));

                ui[0].label("Materials");
                ui[1].label(format!("{}", props.n_materials));

                ui[0].label("Textures");
                ui[1].label(format!("{}", props.n_textures));

                ui[0].label("Draw calls");
                ui[1].label(format!("{}", props.n_draw_calls));
            });
        }
        Shown::Sprite(_sprite) => {
            ui.label("Sprite");
        }
    }
}

struct TabViewer<'a> {
    state: &'a mut State,
}

impl<'a> egui_dock::TabViewer for TabViewer<'a> {
    type Tab = Tab;

    fn ui(&mut self, ui: &mut Ui, tab: &mut Self::Tab) {
        match tab {
            Tab::View => return,
            Tab::Explorer => explorer(self.state, ui),
            Tab::Properties => properties(self.state, ui),
            Tab::ModelProperties => model_properties(self.state, ui),
        }
    }

    fn title(&mut self, tab: &mut Tab) -> egui::WidgetText {
        match tab {
            Tab::Explorer => "Explorer".into(),
            Tab::Properties => "Properties".into(),
            Tab::ModelProperties => "Model Properties".into(),
            Tab::View => "View".into(),
        }
    }

    #[inline]
    fn tab_style_override(&self, tab: &Self::Tab, global_style: &TabStyle) -> Option<TabStyle> {
        if matches!(tab, Tab::View) {
            return Some(TabStyle {
                active: TabInteractionStyle {
                    bg_fill: Color32::TRANSPARENT,
                    ..global_style.active
                },
                focused: TabInteractionStyle {
                    bg_fill: Color32::TRANSPARENT,
                    ..global_style.focused
                },
                hovered: TabInteractionStyle {
                    bg_fill: Color32::TRANSPARENT,
                    ..global_style.hovered
                },
                inactive: TabInteractionStyle {
                    bg_fill: Color32::TRANSPARENT,
                    ..global_style.inactive
                },
                tab_body: TabBodyStyle {
                    bg_fill: Color32::TRANSPARENT,
                    ..global_style.tab_body
                },
                hline_below_active_tab_name: false,
                ..global_style.clone()
            });
        }
        None
    }
}

impl Drawable for Shown {
    fn draw<'a>(&'a self, gfx: &'a GfxContext, rp: &mut RenderPass<'a>) {
        match self {
            Shown::None | Shown::Error(_) => {}
            Shown::Model((mesh, _)) => mesh.draw(gfx, rp),
            Shown::Sprite(sprite) => sprite.draw(gfx, rp),
        }
    }

    fn draw_depth<'a>(
        &'a self,
        gfx: &'a GfxContext,
        rp: &mut RenderPass<'a>,
        shadow_cascade: Option<&Matrix4>,
    ) {
        match self {
            Shown::None | Shown::Error(_) => {}
            Shown::Model((mesh, _)) => mesh.draw_depth(gfx, rp, shadow_cascade),
            Shown::Sprite(sprite) => sprite.draw_depth(gfx, rp, shadow_cascade),
        }
    }
}
