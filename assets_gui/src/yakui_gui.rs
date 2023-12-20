use common::descriptions::{BuildingGen, CompanyKind};
use std::rc::Rc;
use yakui::context::dom;
use yakui::event::{EventInterest, EventResponse, WidgetEvent};
use yakui::paint::PaintRect;
use yakui::widget::{EventContext, Widget};
use yakui::{
    Alignment, Color, Constraints, CrossAxisAlignment, Dim2, MainAxisAlignment, Response, Vec2,
    WidgetId,
};
use yakui_widgets::util::widget;
use yakui_widgets::widgets::{Button, Layer, List, Pad, Slider, StateResponse};
use yakui_widgets::*;

use engine::meshload::MeshProperties;
use engine::wgpu::RenderPass;
use engine::{set_cursor_icon, CursorIcon, Drawable, GfxContext, Mesh, SpriteBatch};
use geom::Matrix4;

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
        Self::scrollbar(&mut off, false);
    }

    fn scrollbar(off: &mut Response<StateResponse<f32>>, scrollbar_on_left_side: bool) {
        colored_box_container(Color::GRAY.adjust(0.5), || {
            constrained(Constraints::tight(Vec2::new(5.0, f32::INFINITY)), || {
                let last_val = use_state(|| None);
                let mut hovered = false;
                let d = draggable(|| {
                    hovered = *widget::<IsHovered>(());
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
                on_changed(should_show_mouse_icon, || {
                    set_colresize_icon(should_show_mouse_icon);
                });
            });
        });
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
                Self::scrollbar(&mut off, true);
                constrained(
                    Constraints::loose(Vec2::new(off.get(), f32::INFINITY)),
                    || {
                        colored_box_container(Color::GRAY, || {
                            let comp = &mut self.gui.companies.companies[i];
                            let mut props = PropertiesBuilder::new();

                            props.add("Name", || text_inp(&mut comp.name));

                            props.add("Kind", || {
                                let mut selected = match comp.kind {
                                    CompanyKind::Store => 0,
                                    CompanyKind::Factory { .. } => 1,
                                    CompanyKind::Network => 2,
                                };

                                if combo_box(&mut selected, &["Store", "Factory", "Network"]) {
                                    match selected {
                                        0 => comp.kind = CompanyKind::Store,
                                        1 => comp.kind = CompanyKind::Factory { n_trucks: 1 },
                                        2 => comp.kind = CompanyKind::Network,
                                        _ => unreachable!(),
                                    }
                                }
                            });
                            props.add("Building generator", || {
                                let mut selected = match comp.bgen {
                                    BuildingGen::House => unreachable!(),
                                    BuildingGen::Farm => 0,
                                    BuildingGen::CenteredDoor { .. } => 1,
                                    BuildingGen::NoWalkway { .. } => 2,
                                };

                                if combo_box(
                                    &mut selected,
                                    &["Farm", "Centered door", "No walkway"],
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
                            });
                            props.add("Recipe", || {
                                label(" ");
                            });

                            let recipe = &mut comp.recipe;
                            props.add("complexity", || inspect_v(&mut recipe.complexity));
                            props.add("storage_multiplier", || {
                                inspect_v(&mut recipe.storage_multiplier)
                            });
                            props.add("consumption", || {
                                label(" ");
                            });
                            let consumption: Vec<_> = recipe
                                .consumption
                                .iter()
                                .map(|(_, v)| Rc::new(std::cell::RefCell::new(*v)))
                                .collect();
                            for ((name, _), amount) in
                                recipe.consumption.iter().zip(consumption.iter())
                            {
                                let amount = amount.clone();
                                props.add(name, move || inspect_v(&mut *amount.borrow_mut()));
                            }

                            props.add("production", || {
                                label(" ");
                            });
                            //for (name, amount) in recipe.production.iter_mut() {
                            //    props.add(name, || inspect_v(amount));
                            //}

                            props.add("n_workers", || inspect_v(&mut comp.n_workers));
                            props.add("size", || inspect_v(&mut comp.size));
                            props.add("asset_location", || text_inp(&mut comp.asset_location));
                            props.add("price", || inspect_v(&mut comp.price));

                            let mut v = comp.zone.is_some();
                            props.add("zone", || {
                                center(|| {
                                    v = checkbox(v).checked;
                                });
                            });

                            if let Some(ref mut z) = comp.zone {
                                props.add("floor", || text_inp(&mut z.floor));
                                props.add("filler", || text_inp(&mut z.filler));
                                props.add("price_per_area", || inspect_v(&mut z.price_per_area));
                            }

                            props.show();

                            if v != comp.zone.is_some() {
                                if v {
                                    comp.zone = Some(Default::default());
                                } else {
                                    comp.zone = None;
                                }
                            }

                            for ((_, amt), amount_ref) in
                                recipe.consumption.iter_mut().zip(consumption.iter())
                            {
                                *amt = *amount_ref.borrow();
                            }

                            /*
                            column(|| {
                                inspect_v("complexity", &mut comp.recipe.complexity);
                                inspect_v("storage_multiplier", &mut comp.recipe.storage_multiplier);
                                label("consumption");
                                for (name, amount) in comp.recipe.consumption.iter_mut() {
                                    inspect_v(name, amount);
                                }

                                label("production");
                                for (name, amount) in comp.recipe.production.iter_mut() {
                                    inspect_v(name, amount);
                                }

                                inspect_v("n_workers", &mut comp.n_workers);
                                inspect_v("size", &mut comp.size);
                                text_inp("asset_location", &mut comp.asset_location);
                                inspect_v("price", &mut comp.price);

                                ui_opt("zone", &mut comp.zone, |zone| {
                                    text_inp("floor", &mut zone.floor);
                                    text_inp("filler", &mut zone.filler);
                                    inspect_v("price_per_area", &mut zone.price_per_area);
                                });
                            });*/
                        });
                    },
                );
            }
        }
    }
}

fn on_changed<T: Copy + PartialEq + 'static>(v: T, f: impl FnOnce()) {
    let old_v = use_state(|| None);
    if old_v.get() != Some(v) {
        old_v.set(Some(v));
        f();
    }
}

#[allow(clippy::type_complexity)]
struct PropertiesBuilder<'a> {
    props: Vec<(&'a str, Box<dyn FnOnce() + 'a>)>,
}

impl<'a> PropertiesBuilder<'a> {
    fn new() -> Self {
        Self { props: Vec::new() }
    }

    fn add(&mut self, name: &'a str, f: impl FnOnce() + 'a) {
        self.props.push((name, Box::new(f)));
    }

    fn show(self) {
        row(|| {
            column(|| {
                let parent = dom().current();
                label("Properties");
                let c = Constraints {
                    min: Vec2::new(50.0, 50.0),
                    max: Vec2::new(f32::INFINITY, f32::INFINITY),
                };
                Self::horiz_line(parent);
                for (name, _) in &self.props {
                    constrained(c, || {
                        pad(Pad::all(3.0), || {
                            center(|| {
                                label(name.to_string());
                            });
                        });
                    });
                    Self::horiz_line(parent);
                }
            });
            let mut c = List::column();
            c.cross_axis_alignment = CrossAxisAlignment::Stretch;
            c.show(|| {
                let parent = dom().current();
                label(" ");
                let c = Constraints {
                    min: Vec2::new(50.0, 50.0),
                    max: Vec2::new(f32::INFINITY, 50.0),
                };
                for (_, f) in self.props {
                    constrained(c, || {
                        pad(Pad::all(3.0), || {
                            f();
                        });
                    });
                    Self::horiz_line(parent);
                }
            });
        });
    }

    fn horiz_line(width_id: WidgetId) {
        canvas(move |ctx| {
            let w = ctx.layout.get(width_id).unwrap().rect.size().x;
            let layout = ctx.layout.get(ctx.dom.current()).unwrap();

            let mut r = layout.rect;
            r.set_size(Vec2::new(w, 3.0));
            r.set_pos(Vec2::new(r.pos().x, r.pos().y - 3.0 / 2.0));
            let mut pr = PaintRect::new(r);
            pr.color = Color::GRAY.adjust(0.8);
            pr.add(ctx.paint);
        });
    }
}

trait Slidable: Copy {
    fn to_f64(self) -> f64;
    fn from_f64(v: f64) -> Self;
}

macro_rules! impl_slidable {
    ($($t:ty),*) => {
        $(
            impl Slidable for $t {
                fn to_f64(self) -> f64 {
                    self as f64
                }

                fn from_f64(v: f64) -> Self {
                    v as Self
                }
            }
        )*
    };
}

impl_slidable!(i32, u32, i64, u64, f32, f64);

fn inspect_v<T: Slidable>(amount: &mut T) {
    let mut l = List::column();
    l.main_axis_alignment = MainAxisAlignment::Center;
    l.cross_axis_alignment = CrossAxisAlignment::Stretch;
    l.show(|| {
        let mut slider = Slider::new((*amount).to_f64(), 1.0, 10.0);
        slider.step = Some(1.0);
        if let Some(v) = slider.show().value {
            *amount = Slidable::from_f64(v);
        }
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

#[derive(Debug)]
struct IsHovered {
    hovered: bool,
}

impl Widget for IsHovered {
    type Props<'a> = ();
    type Response = bool;

    fn new() -> Self {
        Self { hovered: false }
    }

    fn update(&mut self, _: Self::Props<'_>) -> Self::Response {
        self.hovered
    }

    fn event_interest(&self) -> EventInterest {
        EventInterest::MOUSE_INSIDE
    }

    fn event(&mut self, _: EventContext<'_>, event: &WidgetEvent) -> EventResponse {
        match *event {
            WidgetEvent::MouseEnter => self.hovered = true,
            WidgetEvent::MouseLeave => self.hovered = false,
            _ => {}
        };
        EventResponse::Bubble
    }
}

pub fn combo_box(selected: &mut usize, items: &[&str]) -> bool {
    let open = use_state(|| false);
    let mut changed = false;

    center(|| {
        let mut l = List::column();
        l.main_axis_alignment = MainAxisAlignment::Center;
        l.show(|| {
            if button(items[*selected].to_string()).clicked {
                open.modify(|x| !x);
            }

            if open.get() {
                Layer::new().show(|| {
                    reflow(Alignment::BOTTOM_LEFT, Dim2::ZERO, || {
                        column(|| {
                            for (i, item) in items.iter().enumerate() {
                                if i == *selected {
                                    continue;
                                }
                                colored_box_container(Color::BLUE, || {
                                    if button(item.to_string()).clicked {
                                        *selected = i;
                                        open.set(false);
                                        changed = true;
                                    }
                                });
                            }
                        });
                    });
                });
            }
        });
    });

    changed
}
