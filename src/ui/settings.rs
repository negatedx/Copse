use crate::state::{AppState, Theme};
use egui::{Color32, Context, RichText, Slider};

pub fn show(ctx: &Context, state: &mut AppState, system_ppp: f32) {
    if !state.ui.show_settings {
        return;
    }

    let mut open = true;
    egui::Window::new("Settings")
        .open(&mut open)
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .min_width(320.0)
        .show(ctx, |ui| {
            ui.add_space(4.0);

            // ── Theme ──────────────────────────────────────────────────────────
            ui.label(RichText::new("Theme").size(11.0).color(Color32::GRAY));
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                for (label, variant) in [
                    ("Dark", Theme::Dark),
                    ("Light", Theme::Light),
                    ("System", Theme::System),
                ] {
                    let selected = state.settings.theme == variant;
                    if ui.add(egui::SelectableLabel::new(selected, label)).clicked() {
                        state.settings.theme = variant;
                        state.settings.save();
                    }
                }
            });

            ui.add_space(12.0);
            ui.separator();
            ui.add_space(8.0);

            // ── UI Scale ───────────────────────────────────────────────────────
            ui.label(RichText::new("UI Scale").size(11.0).color(Color32::GRAY));
            ui.add_space(4.0);
            let mut scale = state.settings.ui_scale;
            if ui
                .add(Slider::new(&mut scale, 0.75f32..=2.0).step_by(0.05).suffix("×"))
                .changed()
            {
                state.settings.ui_scale = scale;
                state.settings.save();
                ctx.set_pixels_per_point(system_ppp * scale);
            }
            ui.add_space(4.0);
            if ui.small_button("Reset to default").clicked() {
                state.settings.ui_scale = 1.0;
                state.settings.save();
                ctx.set_pixels_per_point(system_ppp);
            }

            ui.add_space(12.0);
            ui.separator();
            ui.add_space(8.0);

            // ── History limit ──────────────────────────────────────────────────
            ui.label(RichText::new("History Limit").size(11.0).color(Color32::GRAY));
            ui.add_space(4.0);
            let mut limit = state.settings.history_limit;
            if ui
                .add(Slider::new(&mut limit, 10usize..=500).step_by(10.0).suffix(" commits"))
                .changed()
            {
                state.settings.history_limit = limit;
                state.settings.save();
            }

            ui.add_space(8.0);
        });

    if !open {
        state.ui.show_settings = false;
    }
}
