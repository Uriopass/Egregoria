use std::path::PathBuf;
use std::time::Instant;

use yakui::widgets::List;
use yakui::{
    reflow, use_state, Alignment, Color, CrossAxisAlignment, Dim2, MainAxisAlignment, MainAxisSize,
    Pivot, TextureId, Vec2,
};

use common::FastMap;
use engine::wgpu::TextureFormat;
use engine::{Context, TextureBuilder};
use geom::{Camera, Degrees, Polygon, Vec3};
use goryak::{
    blur_bg, fixed_spacer, image_button, is_hovered, mincolumn, minrow, on_secondary_container,
    padxy, primary, secondary_container, textc, titlec, HorizScrollSize,
};
use prototypes::{
    prototypes_iter, BuildingPrototypeID, GoodsCompanyID, GoodsCompanyPrototype, Prototype,
    RenderAsset,
};
use simulation::map::{BuildingKind, Zone};
use simulation::world_command::WorldCommand;

use crate::gui::item_icon_yakui;
use crate::gui::specialbuilding::{SpecialBuildKind, SpecialBuildingResource};
use crate::uiworld::UiWorld;

pub fn special_building_properties(uiw: &UiWorld) {
    let mut state = uiw.write::<SpecialBuildingResource>();
    let icons = uiw.read::<BuildingIcons>();

    HorizScrollSize::Max.show(|| {
        padxy(50.0, 10.0, || {
            let mut l = List::row();
            l.main_axis_alignment = MainAxisAlignment::Center;
            l.cross_axis_alignment = CrossAxisAlignment::Center;
            l.main_axis_size = MainAxisSize::Min;
            l.item_spacing = 10.0;
            l.show(|| {
                let tooltip_active = use_state(|| Option::<(GoodsCompanyID, Instant)>::None);
                for descr in prototypes_iter::<GoodsCompanyPrototype>() {
                    let Some(tex_id) = icons.ids.get(&descr.parent().id) else {
                        continue;
                    };

                    minrow(0.0, || {
                        let default_col = Color::WHITE;
                        let hover_col = primary();
                        let active_col = default_col.with_alpha(0.5);

                        let resp = image_button(
                            *tex_id,
                            Vec2::splat(64.0),
                            default_col,
                            hover_col,
                            active_col,
                            "",
                        );

                        if resp.hovering {
                            tooltip_active.set(Some((descr.id, Instant::now())));
                        }

                        if tooltip_active
                            .borrow()
                            .map(|(id, last)| id == descr.id && last.elapsed().as_secs_f32() < 0.2)
                            .unwrap_or(false)
                        {
                            reflow(
                                Alignment::TOP_CENTER,
                                Pivot::BOTTOM_CENTER,
                                Dim2::pixels(0.0, -20.0),
                                || {
                                    let hov_resp = is_hovered(|| {
                                        blur_bg(
                                            secondary_container().with_alpha(0.5),
                                            10.0,
                                            || {
                                                padxy(10.0, 10.0, || {
                                                    mincolumn(3.0, || {
                                                        titlec(
                                                            on_secondary_container(),
                                                            &descr.label,
                                                        );
                                                        textc(
                                                            on_secondary_container(),
                                                            format!("workers: {}", descr.n_workers),
                                                        );

                                                        if let Some(ref recipe) = descr.recipe {
                                                            fixed_spacer((0.0, 10.0));
                                                            if !recipe.consumption.is_empty() {
                                                                textc(
                                                                    on_secondary_container(),
                                                                    "consumption:",
                                                                );
                                                                for item in &recipe.consumption {
                                                                    item_icon_yakui(
                                                                        uiw,
                                                                        item.id,
                                                                        item.amount,
                                                                    );
                                                                }
                                                                fixed_spacer((0.0, 10.0));
                                                            }
                                                            if !recipe.production.is_empty() {
                                                                textc(
                                                                    on_secondary_container(),
                                                                    "production:",
                                                                );
                                                                for item in &recipe.production {
                                                                    item_icon_yakui(
                                                                        uiw,
                                                                        item.id,
                                                                        item.amount,
                                                                    );
                                                                }
                                                                fixed_spacer((0.0, 10.0));
                                                            }
                                                            textc(
                                                                on_secondary_container(),
                                                                format!(
                                                                    "time: {}",
                                                                    recipe.duration
                                                                ),
                                                            );
                                                            textc(
                                                                on_secondary_container(),
                                                                format!(
                                                                    "storage multiplier: {}",
                                                                    recipe.storage_multiplier
                                                                ),
                                                            );
                                                        }

                                                        if let Some(p) = descr.power_consumption {
                                                            fixed_spacer((0.0, 10.0));
                                                            textc(
                                                                on_secondary_container(),
                                                                format!("Power: {}", p),
                                                            );
                                                        }
                                                        if let Some(p) = descr.power_production {
                                                            fixed_spacer((0.0, 10.0));
                                                            textc(
                                                                on_secondary_container(),
                                                                format!("Power production: {}", p),
                                                            );
                                                        }
                                                    });
                                                });
                                            },
                                        );
                                    });
                                    if hov_resp.hovered {
                                        tooltip_active.set(Some((descr.id, Instant::now())));
                                    }
                                },
                            );
                        }

                        if resp.clicked || state.opt.is_none() {
                            let bkind = BuildingKind::GoodsCompany(descr.id);
                            let bgen = descr.bgen;
                            let has_zone = descr.zone.is_some();
                            state.opt = Some(SpecialBuildKind {
                                road_snap: true,
                                make: Box::new(move |args| {
                                    vec![WorldCommand::MapBuildSpecialBuilding {
                                        pos: args.obb,
                                        kind: bkind,
                                        gen: bgen,
                                        zone: has_zone.then(|| {
                                            Zone::new(
                                                Polygon::from(args.obb.corners.as_slice()),
                                                geom::Vec2::X,
                                            )
                                        }),
                                        connected_road: args.connected_road,
                                    }]
                                }),
                                size: descr.size,
                                asset: descr.asset.clone(),
                            });
                        }
                    });
                }
            });
        });
    });

    /*
    let bdescrpt_w = 180.0;

    if let Some(descr) = picked_descr {
        Window::new("Building description")
            .default_width(bdescrpt_w)
            .auto_sized()
            .fixed_pos([
                w - toolbox_w - building_select_w - bdescrpt_w,
                h * 0.5 - 30.0,
            ])
            .hscroll(false)
            .title_bar(true)
            .collapsible(false)
            .resizable(false)
            .show(ui.ctx(), |ui| {
                ui.label(format!("workers: {}", descr.n_workers));

                if let Some(ref recipe) = descr.recipe {
                    ui.add_space(10.0);
                    if !recipe.consumption.is_empty() {
                        ui.label("consumption:");
                        for item in &recipe.consumption {
                            item_icon(ui, uiworld, item.id, item.amount);
                        }
                        ui.add_space(10.0);
                    }
                    if !recipe.production.is_empty() {
                        ui.label("production:");
                        for item in &recipe.production {
                            item_icon(ui, uiworld, item.id, item.amount);
                        }
                        ui.add_space(10.0);
                    }
                    ui.label(format!("time: {}", recipe.duration));
                    ui.label(format!("storage multiplier: {}", recipe.storage_multiplier));
                }

                if let Some(p) = descr.power_consumption {
                    ui.add_space(10.0);
                    ui.label(format!("Power: {}", p));
                }
                if let Some(p) = descr.power_production {
                    ui.add_space(10.0);
                    ui.label(format!("Power production: {}", p));
                }
            });
     */
}

#[derive(Default)]
pub struct BuildingIcons {
    ids: FastMap<BuildingPrototypeID, TextureId>,
}

pub fn do_icons(ctx: &mut Context, uiw: &UiWorld) {
    let mut state = uiw.write::<BuildingIcons>();

    let mut cam = Camera::new(Vec3::new(0.0, 0.0, 0.0), 256.0, 256.0);

    cam.fovy = 30.0;
    cam.pitch = Degrees(35.0).into();
    cam.yaw = Degrees(-130.0).into();

    state.ids.clear();

    let gfx = &mut ctx.gfx;

    for building in prototypes::BuildingPrototype::iter() {
        if state.ids.contains_key(&building.id) {
            continue;
        }
        if let RenderAsset::Sprite { ref path } = building.asset {
            let t = gfx.texture(path, "building icon");
            let tex_id = ctx.yakui.add_texture(&t);
            state.ids.insert(building.id, tex_id);
            continue;
        }

        let RenderAsset::Mesh { ref path } = building.asset else {
            continue;
        };
        let cache_path = PathBuf::from(format!(
            "assets/generated/building_icons/{}.png",
            building.id.hash()
        ));
        //if std::fs::metadata(&cache_path).is_ok() {
        //    let t = TextureBuilder::from_path(&cache_path).build(&gfx.device, &gfx.queue);
        //    let tex_id = yakui.add_texture(&t);
        //    state.ids.insert(building.id, tex_id);
        //    continue;
        //}

        let Ok(mesh) = gfx.mesh(path.as_ref()) else {
            continue;
        };

        let t = TextureBuilder::empty(128, 128, 1, TextureFormat::Rgba8UnormSrgb)
            .with_label("building icon")
            .with_usage(
                engine::wgpu::TextureUsages::COPY_DST
                    | engine::wgpu::TextureUsages::COPY_SRC
                    | engine::wgpu::TextureUsages::RENDER_ATTACHMENT
                    | engine::wgpu::TextureUsages::TEXTURE_BINDING,
            )
            .build_no_queue(&gfx.device);

        let t_msaa = TextureBuilder::empty(128, 128, 1, TextureFormat::Rgba8UnormSrgb)
            .with_label("building icon msaa")
            .with_usage(engine::wgpu::TextureUsages::RENDER_ATTACHMENT)
            .with_sample_count(4)
            .build_no_queue(&gfx.device);

        let aabb3 = mesh.lods[0].aabb3;
        cam.pos = aabb3.center();
        cam.dist = aabb3.ll.distance(aabb3.ur);
        cam.update();

        mesh.render_to_texture(&cam, gfx, &t, &t_msaa);

        t.save_to_file(&gfx.device, &gfx.queue, cache_path, 0);

        let tex_id = ctx.yakui.add_texture(&t);

        state.ids.insert(building.id, tex_id);
    }
}
