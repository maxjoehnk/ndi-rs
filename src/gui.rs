use crate::Model;
use nannou::prelude::*;
use nannou::App;
use nannou_egui::egui;
use nannou_egui::egui::ComboBox;

pub fn update(_app: &App, model: &mut Model, _update: Update) {
    if model.ui_open {
        let ctx = model.egui.begin_frame();
        egui::Window::new("Settings").show(&ctx, |ui| {
            ui.label("Sources");
            ComboBox::from_label("Source")
                .selected_text(if let Some(ref source) = model.selected_source {
                    source.get_name()
                } else {
                    "Select a source".to_string()
                })
                .show_ui(ui, |ui| {
                    let selected = model
                        .selected_source
                        .as_ref()
                        .map(|source| source.get_name());
                    for source in &model.sources {
                        let name = source.get_name();
                        if ui
                            .add(egui::SelectableLabel::new(
                                selected.as_ref() == Some(&name),
                                &name,
                            ))
                            .clicked()
                        {
                            model.selected_source = Some(source.clone());
                            model.selected_source_send.send(source.clone()).unwrap();
                        }
                    }
                });
        });
    }
}
