use egui::{Color32, Ui};
use egui_dock::{DockArea, NodeIndex, Style, TabStyle, Tree};

use engine::meshload::MeshProperties;
use engine::wgpu::{BindGroup, RenderPass};
use engine::{Drawable, GfxContext, Mesh, SpriteBatch};
use geom::Matrix4;

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
    pub tree: Option<Tree<Tab>>,
    pub companies: Companies,
    pub inspected: Inspected,
    pub shown: Shown,
}

impl Gui {
    pub fn new() -> Self {
        let mut tree = Tree::new(vec![Tab::View]);

        let view = NodeIndex::root();
        let [view, _] = tree.split_left(view, 0.2, vec![Tab::Explorer]);
        let [view, _] = tree.split_right(view, 0.8, vec![Tab::Properties]);
        tree.split_below(view, 0.8, vec![Tab::ModelProperties]);

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
            ui.label(&comp.name);
        }
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
                bg_fill: Color32::TRANSPARENT,
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
        proj: &'a BindGroup,
    ) {
        match self {
            Shown::None | Shown::Error(_) => {}
            Shown::Model((mesh, _)) => mesh.draw_depth(gfx, rp, shadow_cascade, proj),
            Shown::Sprite(sprite) => sprite.draw_depth(gfx, rp, shadow_cascade, proj),
        }
    }
}
