use crate::{InspectArgsDefault, InspectRenderDefault};
use std::collections::{BTreeMap, BTreeSet};

impl<K: InspectRenderDefault<K>, V: InspectRenderDefault<V>> InspectRenderDefault<BTreeMap<K, V>>
    for BTreeMap<K, V>
{
    fn render(data: &Self, label: &'static str, ui: &mut egui::Ui, args: &InspectArgsDefault) {
        let v = data;

        ui.collapsing(label, |ui| {
            for (i, (k, v)) in v.iter().enumerate() {
                ui.push_id(i, |ui| {
                    ui.horizontal(|ui| {
                        <K as InspectRenderDefault<K>>::render(k, "", ui, args);
                        <V as InspectRenderDefault<V>>::render(v, "", ui, args);
                    });
                });
            }
        });
    }

    fn render_mut(
        data: &mut Self,
        label: &str,

        ui: &mut egui::Ui,
        args: &InspectArgsDefault,
    ) -> bool {
        let v = data;

        let mut changed = false;
        egui::CollapsingHeader::new(label).show(ui, |ui| {
            for (i, (k, v)) in v.iter_mut().enumerate() {
                ui.push_id(i, |ui| {
                    ui.horizontal(|ui| {
                        <K as InspectRenderDefault<K>>::render(k, "", ui, args);
                        changed |= <V as InspectRenderDefault<V>>::render_mut(v, "", ui, args);
                    });
                });
            }
        });

        changed
    }
}

impl<T: InspectRenderDefault<T>> InspectRenderDefault<BTreeSet<T>> for BTreeSet<T> {
    fn render(data: &Self, label: &'static str, ui: &mut egui::Ui, args: &InspectArgsDefault) {
        egui::CollapsingHeader::new(label).show(ui, |ui| {
            for (i, x) in data.iter().enumerate() {
                ui.push_id(i, |ui| {
                    <T as InspectRenderDefault<T>>::render(x, "", ui, args);
                });
            }
        });
    }

    fn render_mut(
        data: &mut Self,
        label: &'static str,
        ui: &mut egui::Ui,
        args: &InspectArgsDefault,
    ) -> bool {
        Self::render(data, label, ui, args);
        false
    }
}
