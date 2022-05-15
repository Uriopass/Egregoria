use crate::{InspectArgsDefault, InspectRenderDefault};
use imgui::Ui;
use std::collections::{BTreeMap, BTreeSet};

impl<K: InspectRenderDefault<K>, V: InspectRenderDefault<V>> InspectRenderDefault<BTreeMap<K, V>>
    for BTreeMap<K, V>
{
    fn render(data: &[&Self], label: &'static str, ui: &Ui<'_>, args: &InspectArgsDefault) {
        if data.len() != 1 {
            unimplemented!();
        }

        let v = data[0];

        if imgui::CollapsingHeader::new(&label).build(ui) {
            ui.indent();
            for (i, (k, v)) in v.iter().enumerate() {
                let id = ui.push_id(i as i32);
                <K as InspectRenderDefault<K>>::render(&[k], "", ui, args);
                ui.same_line();
                <V as InspectRenderDefault<V>>::render(&[v], "", ui, args);
                id.pop();
            }
            ui.unindent();
        }
    }

    fn render_mut(
        data: &mut [&mut Self],
        label: &str,

        ui: &Ui<'_>,
        args: &InspectArgsDefault,
    ) -> bool {
        if data.len() != 1 {
            unimplemented!();
        }

        let v = &mut data[0];

        let mut changed = false;
        if imgui::CollapsingHeader::new(&label).build(ui) {
            ui.indent();
            for (i, (k, v)) in v.iter_mut().enumerate() {
                let id = ui.push_id(i as i32);
                <K as InspectRenderDefault<K>>::render(&[k], "", ui, args);
                ui.same_line();
                changed |= <V as InspectRenderDefault<V>>::render_mut(&mut [v], "", ui, args);
                id.pop();
            }
            ui.unindent();
        }

        changed
    }
}

impl<T: InspectRenderDefault<T>> InspectRenderDefault<BTreeSet<T>> for BTreeSet<T> {
    fn render(data: &[&Self], label: &'static str, ui: &Ui<'_>, args: &InspectArgsDefault) {
        if data.len() != 1 {
            unimplemented!();
        }

        let v = data[0];

        if imgui::CollapsingHeader::new(&label).build(ui) {
            ui.indent();
            for (i, x) in v.iter().enumerate() {
                let id = ui.push_id(i as i32);
                <T as InspectRenderDefault<T>>::render(&[x], "", ui, args);
                id.pop();
            }
            ui.unindent();
        }
    }

    fn render_mut(
        data: &mut [&mut Self],
        label: &str,

        ui: &Ui<'_>,
        args: &InspectArgsDefault,
    ) -> bool {
        if data.len() != 1 {
            unimplemented!();
        }

        let v = &data[0];

        if imgui::CollapsingHeader::new(&label).build(ui) {
            ui.indent();
            for (i, x) in v.iter().enumerate() {
                let id = ui.push_id(i as i32);
                <T as InspectRenderDefault<T>>::render(&[x], "", ui, args);
                id.pop();
            }
            ui.unindent();
        }

        false
    }
}
