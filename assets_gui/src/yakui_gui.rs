use yakui::widgets::{CountGrid, List, Pad, StateResponse};
use yakui::{
    colored_box_container, column, constrained, divider, row, use_state, Constraints,
    CrossAxisAlignment, MainAxisAlignment, MainAxisSize, Response, Vec2,
};

use engine::meshload::CPUMesh;
use engine::wgpu::RenderPass;
use engine::{set_cursor_icon, CursorIcon, Drawable, GfxContext, InstancedMesh, Mesh, SpriteBatch};
use geom::Matrix4;
use goryak::{
    background, button_primary, checkbox_value, constrained_viewport, dragvalue, icon,
    interact_box_radius, is_hovered, on_secondary_container, on_surface, outline_variant,
    round_rect, secondary_container, set_theme, surface, surface_variant, textc, use_changed,
    RoundRect, Theme, VertScrollSize,
};
use prototypes::{prototypes_iter, GoodsCompanyID, GoodsCompanyPrototype};

use crate::lod::LodGenerateParams;
use crate::{GUIAction, State};

#[derive(PartialEq, Eq, Copy, Clone)]
pub enum Inspected {
    None,
    Company(GoodsCompanyID),
}

#[derive(Clone)]
pub enum Shown {
    None,
    Error(String),
    Model((Mesh, Vec<InstancedMesh>, CPUMesh)),
    Sprite(SpriteBatch),
}

pub struct Gui {
    pub inspected: Inspected,
    pub shown: Shown,
}

impl Gui {
    pub fn new() -> Self {
        Self {
            inspected: Inspected::None,
            shown: Shown::None,
        }
    }
}

impl State {
    pub fn gui_yakui(&mut self) {
        constrained_viewport(|| {
            row(|| {
                self.explorer();
                self.model_properties();
                //self.properties();
            });
        });
    }

    fn explorer(&mut self) {
        let mut off = use_state(|| 300.0);
        constrained(
            Constraints::loose(Vec2::new(off.get(), f32::INFINITY)),
            || {
                colored_box_container(background(), || {
                    let mut l = List::column();
                    l.cross_axis_alignment = CrossAxisAlignment::Stretch;
                    l.show(|| {
                        let mut l = List::row();
                        l.item_spacing = 5.0;
                        l.main_axis_alignment = MainAxisAlignment::Center;
                        Pad::all(5.0).show(|| {
                            l.show(|| {
                                if button_primary("Dark theme").show().clicked {
                                    set_theme(Theme::Dark);
                                }
                                if button_primary("Light theme").show().clicked {
                                    set_theme(Theme::Light);
                                }
                            });
                        });
                        VertScrollSize::Percent(1.0).show(|| {
                            let mut l = List::column();
                            l.cross_axis_alignment = CrossAxisAlignment::Stretch;
                            l.main_axis_size = MainAxisSize::Min;
                            l.show(|| {
                                let companies_open = use_state(|| false);
                                Self::explore_item(
                                    0,
                                    false,
                                    "Companies".to_string(),
                                    Some(companies_open.get()),
                                    || {
                                        companies_open.modify(|x| !x);
                                    },
                                );
                                if companies_open.get() {
                                    for comp in prototypes_iter::<GoodsCompanyPrototype>() {
                                        Self::explore_item(
                                            4,
                                            Inspected::Company(comp.id) == self.gui.inspected,
                                            comp.name.to_string(),
                                            None,
                                            || {
                                                self.gui.inspected = Inspected::Company(comp.id);
                                            },
                                        );
                                    }
                                }
                            });
                        });
                    });
                });
            },
        );
        resizebar_vert(&mut off, false);
    }

    fn explore_item(
        indent: usize,
        selected: bool,
        name: String,
        folder: Option<bool>,
        on_click: impl FnOnce(),
    ) {
        divider(outline_variant(), 3.0, 1.0);
        let r = interact_box_radius(
            if selected {
                surface_variant()
            } else {
                surface()
            },
            surface_variant(),
            surface_variant(),
            0.0,
            || {
                let mut p = Pad::ZERO;
                p.left = 4.0;
                p.show(|| {
                    let mut l = List::row();
                    l.cross_axis_alignment = CrossAxisAlignment::Center;
                    l.show(|| {
                        RoundRect::new(0.0)
                            .color(surface_variant())
                            .min_size(Vec2::new(
                                indent as f32 * 4.0 + if folder.is_none() { 12.0 } else { 0.0 },
                                1.0,
                            ))
                            .show();
                        if let Some(v) = folder {
                            let triangle = if v { "caret-down" } else { "caret-right" };
                            icon(on_surface(), triangle);
                        }
                        textc(on_surface(), name);
                    });
                });
            },
        );
        if r.clicked {
            on_click();
        }
    }

    fn model_properties(&mut self) {
        Self::model_properties_container(|| {
            let tc = on_secondary_container(); // text color
            textc(tc, "Model properties");
            match self.gui.shown {
                Shown::None => {
                    textc(tc, "No model selected");
                }
                Shown::Error(ref e) => {
                    textc(tc, e.clone());
                }
                Shown::Model((ref mesh, _, ref mut props)) => {
                    let params = use_state(|| LodGenerateParams {
                        n_lods: 3,
                        quality: 0.9,
                        sloppy: false,
                    });

                    row(|| {
                        let mut c = List::column();
                        c.main_axis_size = MainAxisSize::Min;
                        c.show(|| {
                            CountGrid::col(2)
                                .main_axis_size(MainAxisSize::Min)
                                .show(|| {
                                    params.modify(|mut params| {
                                        textc(tc, "n_lods");
                                        dragvalue().min(1.0).max(4.0).show(&mut params.n_lods);

                                        textc(tc, "quality");
                                        dragvalue().min(0.0).max(1.0).show(&mut params.quality);

                                        checkbox_value(&mut params.sloppy, tc, "sloppy");

                                        params
                                    });
                                });
                            if button_primary("Generate LODs").show().clicked {
                                let asset_path = &props.asset_path;

                                self.actions
                                    .push(GUIAction::GenerateLOD(asset_path.clone(), params.get()));
                            }
                        });

                        let mut lod_details = CountGrid::col(1 + mesh.lods.len());
                        lod_details.main_axis_size = MainAxisSize::Min;
                        lod_details.show(|| {
                            textc(tc, "");
                            for (i, _) in mesh.lods.iter().enumerate() {
                                textc(tc, format!("LOD{}", i));
                            }

                            textc(tc, "Vertices");
                            for lod in &*mesh.lods {
                                textc(tc, format!("{}", lod.n_vertices));
                            }

                            textc(tc, "Triangles");
                            for lod in &*mesh.lods {
                                textc(tc, format!("{}", lod.n_indices / 3));
                            }

                            textc(tc, "Draw calls");
                            for lod in &*mesh.lods {
                                textc(tc, format!("{}", lod.primitives.len()));
                            }

                            textc(tc, "Coverage");
                            for lod in &*mesh.lods {
                                textc(tc, format!("{:.3}", lod.screen_coverage));
                            }
                        });
                    });
                }
                Shown::Sprite(ref _sprite) => {
                    textc(tc, "Sprite");
                }
            }
        });
    }

    fn model_properties_container(children: impl FnOnce()) {
        let mut l = List::column();
        l.main_axis_alignment = MainAxisAlignment::End;
        l.cross_axis_alignment = CrossAxisAlignment::Stretch;
        l.show(|| {
            colored_box_container(background(), || {
                Pad::all(8.0).show(|| {
                    round_rect(10.0, secondary_container(), || {
                        Pad::all(5.0).show(|| {
                            column(children);
                        });
                    });
                });
            });
        });
    }

    /*
    fn properties(&mut self) {
        match self.gui.inspected {
            Inspected::None => {}
            Inspected::Company(i) => {
                properties_container(|| {
                    let comp = prototype(i).unwrap();

                    let label = |name: &str| {
                        pad(Pad::all(3.0), || {
                            labelc(on_background(), name.to_string());
                        });
                    };

                    fn dragv(v: &mut impl Draggable) {
                        Pad::all(5.0).show(|| {
                            stretch_width(|| {
                                dragvalue().show(v);
                            });
                        });
                    }

                    label("Name");
                    text_inp(&mut comp.name);

                    label("Kind");
                    let mut selected = match comp.kind {
                        CompanyKind::Store => 0,
                        CompanyKind::Factory { .. } => 1,
                        CompanyKind::Network => 2,
                    };

                    if combo_box(&mut selected, &["Store", "Factory", "Network"], 150.0) {
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
                    dragv(&mut recipe.complexity);

                    label("storage_multiplier");
                    dragv(&mut recipe.storage_multiplier);

                    label("consumption");
                    label(" ");

                    for (name, amount) in recipe.consumption.iter_mut() {
                        label(name);
                        dragv(amount);
                    }

                    label("production");
                    label(" ");
                    for (name, amount) in recipe.production.iter_mut() {
                        label(name);
                        dragv(amount);
                    }

                    label("n_workers");
                    dragv(&mut comp.n_workers);

                    label("size");
                    dragv(&mut comp.size);

                    label("asset_location");
                    text_inp(&mut comp.asset_location);

                    label("price");
                    dragv(&mut comp.price);

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
                        dragv(&mut z.price_per_area);
                    }
                });
            }
        }
    }*/
}

/*fn properties_container(children: impl FnOnce()) {
    let mut off = use_state(|| 350.0);
    resizebar_vert(&mut off, true);
    constrained(
        Constraints::loose(Vec2::new(off.get(), f32::INFINITY)),
        || {
            colored_box_container(background(), || {
                align(Alignment::TOP_CENTER, || {
                    Pad::balanced(5.0, 20.0).show(|| {
                        RoundRect::new(10.0)
                            .color(secondary_container())
                            .show_children(|| {
                                Pad::all(8.0).show(|| {
                                    CountGrid::col(2)
                                        .main_axis_size(MainAxisSize::Min)
                                        .main_axis_align_items(MainAxisAlignItems::Center)
                                        .show(children);
                                });
                            });
                    });
                });
            });
        },
    );
}*/

/// A horizontal resize bar.
pub fn resizebar_vert(off: &mut Response<StateResponse<f32>>, scrollbar_on_left_side: bool) {
    colored_box_container(outline_variant(), || {
        let last_val = use_state(|| None);
        let mut hovered = false;
        let d = yakui::draggable(|| {
            hovered = is_hovered(|| {
                constrained(Constraints::tight(Vec2::new(5.0, f32::INFINITY)), || {});
            })
            .hovered;
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
}

/*fn text_inp(v: &mut String) {
    center(|| {
        let mut t = TextBox::new(v.clone());
        t.fill = Some(secondary());
        t.style.color = on_secondary();
        if let Some(x) = t.show().into_inner().text {
            *v = x;
        }
    });
}*/

impl Drawable for Shown {
    fn draw<'a>(&'a self, gfx: &'a GfxContext, rp: &mut RenderPass<'a>) {
        match self {
            Shown::None | Shown::Error(_) => {}
            Shown::Model((_, mesh, _)) => mesh.draw(gfx, rp),
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
            Shown::Model((_, mesh, _)) => mesh.draw_depth(gfx, rp, shadow_cascade),
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
