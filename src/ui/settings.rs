use crate::state::{AppState, Theme};
use egui::{Color32, Context, RichText, ScrollArea, Slider, TextEdit};
use egui_phosphor::regular as ph;
use std::path::PathBuf;

pub fn show(ctx: &Context, state: &mut AppState, system_ppp: f32) {
    if !state.ui.show_settings {
        return;
    }

    let mut open = true;
    egui::Window::new("Settings")
        .open(&mut open)
        .collapsible(false)
        .resizable(true)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .min_width(340.0)
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
            let mut scale = state.ui.pending_ui_scale.unwrap_or(state.settings.ui_scale);
            let scale_resp = ui.add(Slider::new(&mut scale, 0.75f32..=2.0).step_by(0.05).suffix("×"));
            if scale_resp.dragged() {
                state.ui.pending_ui_scale = Some(scale);
            } else if scale_resp.drag_stopped() || (!scale_resp.dragged() && scale_resp.changed()) {
                state.settings.ui_scale = scale;
                state.settings.save();
                ctx.set_pixels_per_point(system_ppp * scale);
                state.ui.pending_ui_scale = None;
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

            // ── Font ───────────────────────────────────────────────────────────
            ui.label(RichText::new("Font").size(11.0).color(Color32::GRAY));
            ui.add_space(4.0);

            if ui.checkbox(&mut state.ui.font_monospace_only, "Monospace only").changed() {
                state.ui.font_search.clear();
            }
            ui.add_space(4.0);

            // Snapshot values needed inside the popup closure before any &mut borrows.
            let popup_id = ui.make_persistent_id("font_picker");
            let display_text = if state.settings.font_name.is_empty() {
                "Default (built-in)".to_owned()
            } else {
                state.settings.font_name.clone()
            };
            let fonts = state.ui.available_fonts.clone();
            let mono_only = state.ui.font_monospace_only;

            // Dropdown trigger button.
            // Custom dropdown button: font name left-aligned, ▾ pinned to right edge.
            let btn_width = ui.available_width().max(300.0);
            let (rect, trigger) =
                ui.allocate_exact_size(egui::vec2(btn_width, 24.0), egui::Sense::click());

            if ui.is_rect_visible(rect) {
                let visuals = if trigger.hovered() {
                    ui.style().visuals.widgets.hovered
                } else {
                    ui.style().visuals.widgets.inactive
                };
                ui.painter().rect(rect, visuals.rounding, visuals.bg_fill, visuals.bg_stroke);

                let color = visuals.fg_stroke.color;
                let font_id = egui::TextStyle::Button.resolve(ui.style());
                let arrow_galley = ui.fonts(|f| {
                    f.layout_no_wrap(ph::CARET_DOWN.to_owned(), font_id.clone(), color)
                });
                let arrow_x = rect.right() - arrow_galley.size().x - 6.0;
                let text_galley = ui.fonts(|f| {
                    f.layout(
                        display_text.clone(),
                        font_id,
                        color,
                        arrow_x - rect.left() - 10.0,
                    )
                });
                let y_center = rect.center().y;
                ui.painter().galley(
                    egui::pos2(rect.left() + 6.0, y_center - text_galley.size().y / 2.0),
                    text_galley,
                    color,
                );
                ui.painter().galley(
                    egui::pos2(arrow_x, y_center - arrow_galley.size().y / 2.0),
                    arrow_galley,
                    color,
                );
            }
            if trigger.clicked() {
                ui.memory_mut(|m| {
                    if m.is_popup_open(popup_id) {
                        m.close_popup();
                    } else {
                        m.open_popup(popup_id);
                    }
                });
                state.ui.font_search.clear();
            }

            egui::popup_below_widget(
                ui,
                popup_id,
                &trigger,
                egui::PopupCloseBehavior::CloseOnClickOutside,
                |ui| {
                    ui.set_min_width(300.0);

                    let search_id = ui.make_persistent_id("font_search_input");
                    let search_resp = ui.add(
                        TextEdit::singleline(&mut state.ui.font_search)
                            .id(search_id)
                            .hint_text("Search fonts…")
                            .desired_width(f32::INFINITY),
                    );
                    if !search_resp.has_focus() {
                        search_resp.request_focus();
                    }

                    ui.add_space(2.0);

                    let search_lower = state.ui.font_search.to_lowercase();
                    ScrollArea::vertical().max_height(220.0).show(ui, |ui| {
                        ui.set_min_width(280.0);

                        if search_lower.is_empty() || "default".contains(&search_lower) {
                            if ui
                                .selectable_label(
                                    state.settings.font_name.is_empty(),
                                    "Default (built-in)",
                                )
                                .clicked()
                            {
                                state.settings.font_name = String::new();
                                state.settings.save();
                                ui.memory_mut(|m| m.close_popup());
                            }
                        }

                        for (name, _) in fonts
                            .iter()
                            .filter(|(n, _)| {
                                (search_lower.is_empty()
                                    || n.to_lowercase().contains(&search_lower))
                                    && (!mono_only || is_likely_monospace(n))
                            })
                        {
                            let selected = state.settings.font_name == *name;
                            if ui.selectable_label(selected, name.as_str()).clicked() {
                                state.settings.font_name = name.clone();
                                state.settings.save();
                                ui.memory_mut(|m| m.close_popup());
                            }
                        }
                    });
                },
            );

            ui.add_space(8.0);
            ui.label(RichText::new("Font Size").size(11.0).color(Color32::GRAY));
            ui.add_space(4.0);
            let mut font_size = state.settings.font_size;
            if ui
                .add(Slider::new(&mut font_size, 10.0f32..=24.0).step_by(1.0).suffix(" px"))
                .changed()
            {
                state.settings.font_size = font_size;
                state.settings.save();
            }
            ui.add_space(4.0);
            if ui.small_button("Reset font to default").clicked() {
                state.settings.font_size = 14.0;
                state.settings.font_name = String::new();
                state.settings.save();
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

fn is_likely_monospace(name: &str) -> bool {
    let lower = name.to_lowercase();
    ["mono", "code", "console", "courier", "fixed", "term", "typewriter",
     "hack", "inconsolata", "cascadia", "caskaydia", "fira code", "source code",
     "anonymous", "nerd font"]
        .iter()
        .any(|h| lower.contains(h))
}

/// Returns all TTF/OTF fonts sorted alphabetically by name.
pub(super) fn enumerate_fonts() -> Vec<(String, PathBuf)> {
    let font_dirs = [
        PathBuf::from(r"C:\Windows\Fonts"),
        dirs::data_local_dir()
            .unwrap_or_default()
            .join("Microsoft")
            .join("Windows")
            .join("Fonts"),
    ];

    let mut fonts: Vec<(String, PathBuf)> = Vec::new();
    let mut seen = std::collections::HashSet::new();

    for dir in &font_dirs {
        let Ok(entries) = std::fs::read_dir(dir) else { continue };
        for entry in entries.flatten() {
            let path = entry.path();
            let ext = path.extension().unwrap_or_default().to_string_lossy().to_lowercase();
            if ext != "ttf" && ext != "otf" {
                continue;
            }
            let name = path.file_stem().unwrap_or_default().to_string_lossy().to_string();
            if seen.insert(name.clone()) {
                fonts.push((name, path));
            }
        }
    }

    fonts.sort_by(|a, b| a.0.to_lowercase().cmp(&b.0.to_lowercase()));
    fonts
}
