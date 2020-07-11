use geom::Vec2;

impl imgui_inspect::InspectRenderDefault<Vec2> for Vec2 {
    fn render(
        data: &[&Vec2],
        label: &'static str,
        _: &mut imgui_inspect::specs::World,
        ui: &imgui_inspect::imgui::Ui,
        _: &imgui_inspect::InspectArgsDefault,
    ) {
        if data.len() != 1 {
            unimplemented!();
        }
        let x = data[0];
        imgui_inspect::imgui::InputFloat2::new(ui, &im_str!("{}", label), &mut [x.x, x.y])
            .always_insert_mode(false)
            .build();
    }

    fn render_mut(
        data: &mut [&mut Vec2],
        label: &'static str,
        _: &mut imgui_inspect::specs::World,
        ui: &imgui_inspect::imgui::Ui,
        args: &imgui_inspect::InspectArgsDefault,
    ) -> bool {
        if data.len() != 1 {
            unimplemented!();
        }
        let x = &mut data[0];
        let mut conv = [x.x, x.y];
        let changed = ui
            .drag_float2(&im_str!("{}", label), &mut conv)
            .speed(args.step.unwrap_or(0.1))
            .build();
        x.x = conv[0];
        x.y = conv[1];
        changed
    }
}
