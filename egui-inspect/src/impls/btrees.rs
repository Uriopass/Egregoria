use crate::{Inspect, InspectArgs};
use std::collections::{BTreeMap, BTreeSet};

impl<K: Inspect<K>, V: Inspect<V>> Inspect<BTreeMap<K, V>> for BTreeMap<K, V> {
    fn render(data: &Self, label: &'static str, ui: &mut egui::Ui, args: &InspectArgs) {
        let v = data;

        ui.collapsing(label, |ui| {
            for (i, (k, v)) in v.iter().enumerate() {
                ui.push_id(i, |ui| {
                    ui.horizontal(|ui| {
                        <K as Inspect<K>>::render(k, "", ui, args);
                        <V as Inspect<V>>::render(v, "", ui, args);
                    });
                });
            }
        });
    }

    fn render_mut(data: &mut Self, label: &str, ui: &mut egui::Ui, args: &InspectArgs) -> bool {
        let v = data;

        let mut changed = false;
        egui::CollapsingHeader::new(label).show(ui, |ui| {
            for (i, (k, v)) in v.iter_mut().enumerate() {
                ui.push_id(i, |ui| {
                    ui.horizontal(|ui| {
                        <K as Inspect<K>>::render(k, "", ui, args);
                        changed |= <V as Inspect<V>>::render_mut(v, "", ui, args);
                    });
                });
            }
        });

        changed
    }
}

impl<T: Inspect<T>> Inspect<BTreeSet<T>> for BTreeSet<T> {
    fn render(data: &Self, label: &'static str, ui: &mut egui::Ui, args: &InspectArgs) {
        egui::CollapsingHeader::new(label).show(ui, |ui| {
            for (i, x) in data.iter().enumerate() {
                ui.push_id(i, |ui| {
                    <T as Inspect<T>>::render(x, "", ui, args);
                });
            }
        });
    }

    fn render_mut(
        data: &mut Self,
        label: &'static str,
        ui: &mut egui::Ui,
        args: &InspectArgs,
    ) -> bool {
        Self::render(data, label, ui, args);
        false
    }
}
