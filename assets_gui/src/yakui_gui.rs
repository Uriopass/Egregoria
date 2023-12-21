use common::descriptions::{BuildingGen, CompanyKind};
use yakui::widgets::*;
use yakui::*;

use engine::meshload::MeshProperties;
use engine::wgpu::RenderPass;
use engine::{set_cursor_icon, CursorIcon, Drawable, GfxContext, Mesh, SpriteBatch};
use geom::Matrix4;
use goryak::{
    center_width, checkbox_value, combo_box, drag_value, is_hovered, use_changed, CountGrid,
    MainAxisAlignItems,
};

use crate::companies::Companies;
use crate::State;

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
    pub companies: Companies,
    pub inspected: Inspected,
    pub shown: Shown,
}

impl Gui {
    pub fn new() -> Self {
        Self {
            companies: Companies::new().expect("could not load companies.json"),
            inspected: Inspected::None,
            shown: Shown::None,
        }
    }
}

impl State {
    pub fn gui_yakui(&mut self) {
        row(|| {
            self.explorer();
            self.model_properties();
            self.properties();
        });
    }

    fn explorer(&mut self) {
        let mut off = use_state(|| 300.0);
        constrained(
            Constraints::loose(Vec2::new(off.get(), f32::INFINITY)),
            || {
                colored_box_container(Color::GRAY.with_alpha(0.8), || {
                    scroll_vertical(|| {
                        let mut l = List::column();
                        l.cross_axis_alignment = CrossAxisAlignment::Stretch;
                        l.show(|| {
                            label("Companies");
                            if self.gui.companies.changed && button("Save").clicked {
                                self.gui.companies.save();
                            }
                            for (i, comp) in self.gui.companies.companies.iter().enumerate() {
                                let b = Button::styled(comp.name.to_string());

                                Pad::all(3.0).show(|| {
                                    if b.show().clicked {
                                        self.gui.inspected = Inspected::Company(i);
                                    }
                                });
                            }
                        });
                    });
                });
            },
        );
        resizebar_vert(&mut off, false);
    }

    fn model_properties(&mut self) {
        let mut l = List::column();
        l.main_axis_alignment = MainAxisAlignment::End;
        l.cross_axis_alignment = CrossAxisAlignment::Stretch;
        l.show(|| {
            colored_box_container(Color::GRAY, || {
                column(|| {
                    label("Model properties");
                    match &self.gui.shown {
                        Shown::None => {
                            label("No model selected");
                        }
                        Shown::Error(e) => {
                            label(e.clone());
                        }
                        Shown::Model((_, props)) => {
                            row(|| {
                                column(|| {
                                    label("Vertices");
                                    label("Triangles");
                                    label("Materials");
                                    label("Textures");
                                    label("Draw calls");
                                });
                                column(|| {
                                    label(format!("{}", props.n_vertices));
                                    label(format!("{}", props.n_triangles));
                                    label(format!("{}", props.n_materials));
                                    label(format!("{}", props.n_textures));
                                    label(format!("{}", props.n_draw_calls));
                                });
                            });
                        }
                        Shown::Sprite(_sprite) => {
                            label("Sprite");
                        }
                    }
                });
            });
        });
    }

    fn properties(&mut self) {
        match self.gui.inspected {
            Inspected::None => {}
            Inspected::Company(i) => {
                let mut off = use_state(|| 350.0);
                resizebar_vert(&mut off, true);
                constrained(
                    Constraints::loose(Vec2::new(off.get(), f32::INFINITY)),
                    || {
                        colored_box_container(Color::GRAY, || {
                            CountGrid::col(2)
                                .main_axis_align_items(MainAxisAlignItems::Center)
                                .show(|| {
                                    let comp = &mut self.gui.companies.companies[i];

                                    let label = |name: &str| {
                                        pad(Pad::all(3.0), || {
                                            label(name.to_string());
                                        });
                                    };

                                    label("Name");
                                    text_inp(&mut comp.name);

                                    label("Kind");
                                    let mut selected = match comp.kind {
                                        CompanyKind::Store => 0,
                                        CompanyKind::Factory { .. } => 1,
                                        CompanyKind::Network => 2,
                                    };

                                    if combo_box(
                                        &mut selected,
                                        &["Store", "Factory", "Network"],
                                        150.0,
                                    ) {
                                        match selected {
                                            0 => comp.kind = CompanyKind::Store,
                                            1 => comp.kind = CompanyKind::Factory { n_trucks: 1 },
                                            2 => comp.kind = CompanyKind::Network,
                                            _ => unreachable!(),
                                        }
                                    }

                                    label("Building generator");
                                    let mut selected = match comp.bgen {
                                        BuildingGen::House => unreachable!(),
                                        BuildingGen::Farm => 0,
                                        BuildingGen::CenteredDoor { .. } => 1,
                                        BuildingGen::NoWalkway { .. } => 2,
                                    };

                                    if combo_box(
                                        &mut selected,
                                        &["Farm", "Centered door", "No walkway"],
                                        150.0,
                                    ) {
                                        match selected {
                                            0 => comp.bgen = BuildingGen::Farm,
                                            1 => {
                                                comp.bgen = BuildingGen::CenteredDoor {
                                                    vertical_factor: 1.0,
                                                }
                                            }
                                            2 => {
                                                comp.bgen = BuildingGen::NoWalkway {
                                                    door_pos: geom::Vec2::ZERO,
                                                }
                                            }
                                            _ => unreachable!(),
                                        }
                                    }

                                    label("Recipe");
                                    label(" ");

                                    let recipe = &mut comp.recipe;
                                    label("complexity");
                                    drag_value(&mut recipe.complexity);
                                    label("storage_multiplier");
                                    drag_value(&mut recipe.storage_multiplier);
                                    label("consumption");
                                    label(" ");

                                    for (name, amount) in recipe.consumption.iter_mut() {
                                        label(name);
                                        drag_value(amount);
                                    }

                                    label("production");
                                    label(" ");
                                    for (name, amount) in recipe.production.iter_mut() {
                                        label(name);
                                        drag_value(amount);
                                    }

                                    label("n_workers");
                                    drag_value(&mut comp.n_workers);

                                    label("size");
                                    drag_value(&mut comp.size);

                                    label("asset_location");
                                    text_inp(&mut comp.asset_location);

                                    label("price");
                                    drag_value(&mut comp.price);

                                    label("zone");
                                    let mut v = comp.zone.is_some();
                                    center_width(|| checkbox_value(&mut v));

                                    if v != comp.zone.is_some() {
                                        if v {
                                            comp.zone = Some(Default::default());
                                        } else {
                                            comp.zone = None;
                                        }
                                    }

                                    if let Some(ref mut z) = comp.zone {
                                        label("floor");
                                        text_inp(&mut z.floor);

                                        label("filler");
                                        text_inp(&mut z.filler);

                                        label("price_per_area");
                                        drag_value(&mut z.price_per_area);
                                    }
                                });
                        });
                    },
                );
            }
        }
    }
}

/// A horizontal resize bar.
pub fn resizebar_vert(off: &mut Response<StateResponse<f32>>, scrollbar_on_left_side: bool) {
    colored_box_container(Color::GRAY.adjust(0.5), || {
        constrained(Constraints::tight(Vec2::new(5.0, f32::INFINITY)), || {
            let last_val = use_state(|| None);
            let mut hovered = false;
            let d = draggable(|| {
                hovered = is_hovered();
            })
            .dragging;
            let delta = d
                .map(|v| {
                    let delta = v.current.x - last_val.get().unwrap_or(v.current.x);
                    last_val.set(Some(v.current.x));
                    delta
                })
                .unwrap_or_else(|| {
                    last_val.set(None);
                    0.0
                });
            off.modify(|v| {
                if scrollbar_on_left_side {
                    v - delta
                } else {
                    v + delta
                }
                .clamp(100.0, 600.0)
            });

            let should_show_mouse_icon = d.is_some() || hovered;
            use_changed(should_show_mouse_icon, || {
                set_colresize_icon(should_show_mouse_icon);
            });
        });
    });
}

fn text_inp(v: &mut String) {
    center(|| {
        if let Some(x) = textbox(v.clone()).into_inner().text {
            *v = x;
        }
    });
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

fn set_colresize_icon(enabled: bool) {
    if enabled {
        set_cursor_icon(CursorIcon::ColResize);
    } else {
        set_cursor_icon(CursorIcon::Default);
    }
}
