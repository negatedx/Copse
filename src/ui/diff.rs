use crate::{
    git::{DiffHunk, DiffLine, DiffLineKind},
    state::AppState,
};
use egui::{Color32, FontId, RichText, ScrollArea, Ui};

const MIN_WIDTH_FOR_SIDE_BY_SIDE: f32 = 600.0;

struct Colors {
    add_bg: Color32,
    add_fg: Color32,
    del_bg: Color32,
    del_fg: Color32,
    ctx_fg: Color32,
    ln_fg:  Color32,
    hunk_fg: Color32,
    hunk_header_bg: Color32,
}

pub fn show(ui: &mut Ui, state: &mut AppState) {
    let dark = ui.visuals().dark_mode;
    let c = Colors {
        add_bg: if dark { Color32::from_rgb(30, 55, 30)    } else { Color32::from_rgb(215, 242, 215) },
        add_fg: if dark { Color32::from_rgb(130, 190, 100) } else { Color32::from_rgb(25, 105, 25)   },
        del_bg: if dark { Color32::from_rgb(60, 25, 25)    } else { Color32::from_rgb(250, 220, 220) },
        del_fg: if dark { Color32::from_rgb(200, 100, 100) } else { Color32::from_rgb(155, 30, 30)   },
        ctx_fg: ui.visuals().text_color(),
        ln_fg:  if dark { Color32::from_rgb(80, 80, 80)    } else { Color32::from_rgb(160, 160, 160) },
        hunk_fg: Color32::GRAY,
        hunk_header_bg: if dark { Color32::from_rgb(35, 35, 50) } else { Color32::from_rgb(220, 220, 235) },
    };

    let panel_width = ui.available_width();
    let wide_enough = panel_width >= MIN_WIDTH_FOR_SIDE_BY_SIDE;

    // ── Header ────────────────────────────────────────────────────────────────
    if let Some(file) = state.selected_file() {
        let wt_name   = state.selected_worktree().map(|w| w.name.as_str()).unwrap_or("").to_string();
        let repo_name = state.selected_repo().map(|r| r.name.as_str()).unwrap_or("").to_string();
        let file_name = file.path.to_string_lossy().to_string();

        ui.horizontal(|ui| {
            ui.label(RichText::new(format!("{repo_name} / {wt_name}")).size(11.0).color(Color32::GRAY));
            ui.label(RichText::new("—").size(11.0).color(Color32::GRAY));
            ui.label(RichText::new(&file_name).size(11.0).monospace());
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let wrap_label = if state.settings.diff_word_wrap { "No Wrap" } else { "Wrap" };
                if ui.small_button(wrap_label).clicked() {
                    state.settings.diff_word_wrap = !state.settings.diff_word_wrap;
                    state.settings.save();
                }
                if wide_enough {
                    let split_label = if state.settings.diff_side_by_side { "Unified" } else { "Split" };
                    if ui.small_button(split_label).clicked() {
                        state.settings.diff_side_by_side = !state.settings.diff_side_by_side;
                        state.settings.save();
                    }
                }
            });
        });
        ui.separator();
    } else {
        ui.add_space(8.0);
        ui.vertical_centered(|ui| {
            ui.label(RichText::new("select a file to view diff").size(12.0).color(Color32::GRAY));
        });
        return;
    }

    if state.ui.diff_hunks.is_empty() {
        ui.add_space(8.0);
        ui.label(
            RichText::new("  no diff available (untracked or binary file)")
                .size(11.0)
                .color(Color32::GRAY),
        );
        return;
    }

    let side_by_side = state.settings.diff_side_by_side && wide_enough;
    let word_wrap    = state.settings.diff_word_wrap;
    let hunks: Vec<DiffHunk> = state.ui.diff_hunks.clone();

    if side_by_side {
        if word_wrap {
            let half_width = (panel_width / 2.0 - 1.0).max(0.0);
            show_split_wrap(ui, &hunks, half_width, &c);
        } else {
            show_split_scroll(ui, &hunks, &c);
        }
    } else {
        show_unified(ui, &hunks, word_wrap, &c);
    }
}

// ── Unified ────────────────────────────────────────────────────────────────────

fn show_unified(ui: &mut Ui, hunks: &[DiffHunk], word_wrap: bool, c: &Colors) {
    ScrollArea::new([!word_wrap, true])
        .id_source("diff_scroll_unified")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing = egui::Vec2::ZERO;
            for (hunk_i, hunk) in hunks.iter().enumerate() {
                ui.push_id(hunk_i, |ui| {
                    hunk_header(ui, &hunk.header, word_wrap, c);
                    for (line_i, line) in hunk.lines.iter().enumerate() {
                        let (bg, fg, prefix) = match line.kind {
                            DiffLineKind::Added   => (c.add_bg, c.add_fg, "+"),
                            DiffLineKind::Deleted => (c.del_bg, c.del_fg, "-"),
                            DiffLineKind::Context => (Color32::TRANSPARENT, c.ctx_fg, " "),
                        };
                        ui.push_id(line_i, |ui| {
                            let row_top = ui.cursor().min.y;
                            let row_x   = ui.cursor().min.x;
                            let bg_id   = ui.painter().add(egui::Shape::Noop);
                            ui.horizontal(|ui| {
                                let ln = line.new_lineno.or(line.old_lineno)
                                    .map(|n| format!("{n:>4}"))
                                    .unwrap_or_else(|| "    ".to_string());
                                ui.label(RichText::new(&ln).size(10.0).monospace().color(c.ln_fg));
                                ui.add_space(4.0);
                                let text = format!("{prefix}{}", line.content);
                                let lbl  = egui::Label::new(RichText::new(&text).size(11.0).monospace().color(fg));
                                ui.add(if word_wrap { lbl.wrap() } else { lbl });
                            });
                            if bg != Color32::TRANSPARENT {
                                let row_bottom = ui.cursor().min.y;
                                ui.painter().set(bg_id, egui::Shape::rect_filled(
                                    egui::Rect::from_min_max(
                                        egui::pos2(row_x, row_top),
                                        egui::pos2(row_x + 32_768.0, row_bottom),
                                    ),
                                    0.0, bg,
                                ));
                            }
                        });
                    }
                });
            }
        });
}

// ── Split — no-wrap (two synced scroll areas) ──────────────────────────────────

fn show_split_scroll(ui: &mut Ui, hunks: &[DiffHunk], c: &Colors) {
    let pairs_per_hunk: Vec<(String, Vec<SidePair>)> = hunks.iter()
        .map(|h| (h.header.clone(), build_side_pairs(&h.lines)))
        .collect();

    // Determine driver side once per frame from pointer position.
    let ptr_x = ui.ctx().pointer_hover_pos().map_or(f32::NEG_INFINITY, |p| p.x);
    let mid_x = ui.cursor().min.x + ui.available_width() / 2.0;
    let right_drives = ptr_x >= mid_x;

    // Outer vertical scroll area — lets hunk headers render full-width outside the columns.
    ScrollArea::vertical()
        .id_source("diff_split_v")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing = egui::Vec2::ZERO;

            for (hunk_i, (header, pairs)) in pairs_per_hunk.iter().enumerate() {
                hunk_header(ui, header, false, c);

                // Each hunk owns its horizontal scroll position, stored in egui temp data.
                let hunk_h_id = egui::Id::new(("diff_hunk_h", hunk_i));
                let stored_h: f32 = ui.ctx().data(|d| d.get_temp(hunk_h_id).unwrap_or(0.0));

                let mut new_h = stored_h;
                ui.columns(2, |cols| {
                    let h_vec = egui::Vec2::new(stored_h, 0.0);

                    let left_sa = ScrollArea::horizontal()
                        .id_source(("diff_sl", hunk_i))
                        .auto_shrink([false, true]);
                    let left_sa = if right_drives { left_sa.scroll_offset(h_vec) } else { left_sa };
                    let lo = left_sa.show(&mut cols[0], |ui| {
                        ui.spacing_mut().item_spacing = egui::Vec2::ZERO;
                        for (row_i, pair) in pairs.iter().enumerate() {
                            ui.push_id(row_i, |ui| split_cell(ui, pair.left.as_ref(), false, c));
                        }
                    }).state.offset.x;

                    let right_sa = ScrollArea::horizontal()
                        .id_source(("diff_sr", hunk_i))
                        .auto_shrink([false, true]);
                    let right_sa = if !right_drives {
                        right_sa.scroll_offset(egui::Vec2::new(lo, 0.0))
                    } else {
                        right_sa
                    };
                    let ro = right_sa.show(&mut cols[1], |ui| {
                        ui.spacing_mut().item_spacing = egui::Vec2::ZERO;
                        for (row_i, pair) in pairs.iter().enumerate() {
                            ui.push_id(row_i, |ui| split_cell(ui, pair.right.as_ref(), true, c));
                        }
                    }).state.offset.x;

                    new_h = if right_drives { ro } else { lo };
                });

                ui.ctx().data_mut(|d| d.insert_temp(hunk_h_id, new_h));
            }
        });
}

/// Render one cell in the split view using labels (mirrors the unified approach).
fn split_cell(ui: &mut Ui, line: Option<&DiffLine>, is_right: bool, c: &Colors) {
    let row_top = ui.cursor().min.y;
    let row_x   = ui.cursor().min.x;
    let bg_id   = ui.painter().add(egui::Shape::Noop);
    let bg      = line_bg(&line.cloned(), c.add_bg, c.del_bg);

    ui.horizontal(|ui| {
        match line {
            None => {
                // Invisible placeholder keeps row height consistent with opposite cell.
                ui.label(RichText::new("    ").size(10.0).monospace().color(Color32::TRANSPARENT));
                ui.add_space(4.0);
                ui.label(RichText::new(" ").size(11.0).monospace().color(Color32::TRANSPARENT));
            }
            Some(l) => {
                let (fg, prefix) = line_style(l, c);
                let lineno = if is_right { l.new_lineno } else { l.old_lineno };
                let ln = lineno.map(|n| format!("{n:>4}")).unwrap_or_else(|| "    ".to_string());
                ui.label(RichText::new(&ln).size(10.0).monospace().color(c.ln_fg));
                ui.add_space(4.0);
                let text = format!("{prefix}{}", l.content.trim_end_matches(['\n', '\r']));
                ui.label(RichText::new(&text).size(11.0).monospace().color(fg));
            }
        }
    });

    if bg != Color32::TRANSPARENT {
        let row_bottom = ui.cursor().min.y;
        ui.painter().set(bg_id, egui::Shape::rect_filled(
            egui::Rect::from_min_max(
                egui::pos2(row_x, row_top),
                egui::pos2(row_x + 32_768.0, row_bottom),
            ),
            0.0, bg,
        ));
    }
}

// ── Split — wrap (painter + galley, variable row height) ───────────────────────

fn show_split_wrap(ui: &mut Ui, hunks: &[DiffHunk], half_width: f32, c: &Colors) {
    let ln_font   = FontId::monospace(10.0);
    let text_font = FontId::monospace(11.0);
    let (fixed_row_h, ln_col_w) = ui.fonts(|f| {
        (f.row_height(&text_font) + 2.0, f.glyph_width(&ln_font, '0') * 4.0 + 10.0)
    });
    let wrap_w = (half_width - ln_col_w).max(40.0);
    let sep_color = if ui.visuals().dark_mode {
        Color32::from_rgb(60, 60, 60)
    } else {
        Color32::from_rgb(200, 200, 200)
    };

    ScrollArea::vertical()
        .id_source("diff_scroll_split_wrap")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing = egui::Vec2::ZERO;
            for (hunk_i, hunk) in hunks.iter().enumerate() {
                ui.push_id(hunk_i, |ui| {
                    hunk_header(ui, &hunk.header, true, c);
                    let pairs = build_side_pairs(&hunk.lines);
                    for (row_i, pair) in pairs.iter().enumerate() {
                        ui.push_id(row_i, |ui| {
                            let ry = ui.cursor().min.y;
                            let rx = ui.cursor().min.x;

                            let (lg, rg, row_h) = ui.fonts(|f| {
                                let lg = make_galley(f, &pair.left,  wrap_w, &text_font, c);
                                let rg = make_galley(f, &pair.right, wrap_w, &text_font, c);
                                let rh = lg.size().y.max(rg.size().y).max(fixed_row_h);
                                (lg, rg, rh)
                            });

                            ui.allocate_space(egui::vec2(half_width * 2.0 + 2.0, row_h));
                            draw_split_row_bg(ui, &pair, rx, ry, half_width, row_h, c);
                            ui.painter().vline(rx + half_width + 1.0, ry..=(ry + row_h), egui::Stroke::new(1.0, sep_color));

                            let lx  = rx;
                            let rx2 = rx + half_width + 2.0;
                            let lclip = col_clip(ui, lx,  ry, half_width, row_h);
                            let rclip = col_clip(ui, rx2, ry, half_width, row_h);

                            paint_lineno(ui, &pair.left,  false, lx,  ry, row_h, &ln_font, ln_col_w, c.ln_fg, lclip);
                            paint_lineno(ui, &pair.right, true,  rx2, ry, row_h, &ln_font, ln_col_w, c.ln_fg, rclip);

                            ui.painter_at(lclip).galley(egui::pos2(lx  + ln_col_w, ry + 1.0), lg, c.ctx_fg);
                            ui.painter_at(rclip).galley(egui::pos2(rx2 + ln_col_w, ry + 1.0), rg, c.ctx_fg);
                        });
                    }
                });
            }
        });
}

// ── Shared helpers ─────────────────────────────────────────────────────────────

fn hunk_header(ui: &mut Ui, header: &str, word_wrap: bool, c: &Colors) {
    let row_top = ui.cursor().min.y;
    let row_x   = ui.cursor().min.x;
    let bg_id   = ui.painter().add(egui::Shape::Noop);
    ui.add_space(3.0);
    ui.horizontal(|ui| {
        ui.add_space(6.0);
        if !word_wrap {
            ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
        }
        let lbl = egui::Label::new(RichText::new(header).size(10.0).monospace().color(c.hunk_fg));
        ui.add(if word_wrap { lbl.wrap() } else { lbl });
    });
    ui.add_space(3.0);
    let row_bottom = ui.cursor().min.y;
    ui.painter().set(bg_id, egui::Shape::rect_filled(
        egui::Rect::from_min_max(
            egui::pos2(row_x, row_top),
            egui::pos2(row_x + 32_768.0, row_bottom),
        ),
        0.0, c.hunk_header_bg,
    ));
}

fn draw_split_row_bg(ui: &Ui, pair: &SidePair, rx: f32, ry: f32, half_width: f32, row_h: f32, c: &Colors) {
    let left_bg  = line_bg(&pair.left,  c.add_bg, c.del_bg);
    let right_bg = line_bg(&pair.right, c.add_bg, c.del_bg);
    if left_bg != Color32::TRANSPARENT {
        ui.painter().rect_filled(
            egui::Rect::from_min_size(egui::pos2(rx, ry), egui::vec2(half_width, row_h)),
            0.0, left_bg,
        );
    }
    if right_bg != Color32::TRANSPARENT {
        ui.painter().rect_filled(
            egui::Rect::from_min_size(egui::pos2(rx + half_width + 2.0, ry), egui::vec2(half_width, row_h)),
            0.0, right_bg,
        );
    }
}

fn col_clip(ui: &Ui, col_x: f32, row_y: f32, col_w: f32, row_h: f32) -> egui::Rect {
    egui::Rect::from_min_size(egui::pos2(col_x, row_y), egui::vec2(col_w, row_h))
        .intersect(ui.clip_rect())
}

fn paint_lineno(
    ui: &Ui, line: &Option<DiffLine>, is_right: bool,
    col_x: f32, row_y: f32, row_h: f32,
    ln_font: &FontId, ln_col_w: f32, ln_fg: Color32,
    clip: egui::Rect,
) {
    let Some(l) = line else { return };
    let lineno  = if is_right { l.new_lineno } else { l.old_lineno };
    let ln_str  = lineno.map(|n| format!("{n:>4}")).unwrap_or_else(|| "    ".to_string());
    let ln_h    = ui.fonts(|f| f.row_height(ln_font));
    let _ = ln_col_w; // used by caller for content x offset
    ui.painter_at(clip).text(
        egui::pos2(col_x + 2.0, row_y + (row_h - ln_h) / 2.0),
        egui::Align2::LEFT_TOP, &ln_str, ln_font.clone(), ln_fg,
    );
}

fn make_galley(
    f: &egui::text::Fonts,
    line: &Option<DiffLine>,
    wrap_w: f32,
    font: &FontId,
    c: &Colors,
) -> std::sync::Arc<egui::Galley> {
    match line {
        None    => f.layout(" ".to_string(), font.clone(), Color32::TRANSPARENT, f32::INFINITY),
        Some(l) => {
            let (fg, prefix) = line_style(l, c);
            let text = format!("{prefix}{}", l.content.trim_end_matches(['\n', '\r']));
            f.layout(text, font.clone(), fg, wrap_w)
        }
    }
}

// ── Side-pair logic ────────────────────────────────────────────────────────────

struct SidePair {
    left:  Option<DiffLine>,
    right: Option<DiffLine>,
}

fn build_side_pairs(lines: &[DiffLine]) -> Vec<SidePair> {
    let mut pairs: Vec<SidePair> = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        match lines[i].kind {
            DiffLineKind::Context => {
                pairs.push(SidePair { left: Some(lines[i].clone()), right: Some(lines[i].clone()) });
                i += 1;
            }
            DiffLineKind::Deleted => {
                let mut deleted: Vec<DiffLine> = Vec::new();
                while i < lines.len() && lines[i].kind == DiffLineKind::Deleted {
                    deleted.push(lines[i].clone());
                    i += 1;
                }
                let mut added: Vec<DiffLine> = Vec::new();
                while i < lines.len() && lines[i].kind == DiffLineKind::Added {
                    added.push(lines[i].clone());
                    i += 1;
                }
                for j in 0..deleted.len().max(added.len()) {
                    pairs.push(SidePair { left: deleted.get(j).cloned(), right: added.get(j).cloned() });
                }
            }
            DiffLineKind::Added => {
                pairs.push(SidePair { left: None, right: Some(lines[i].clone()) });
                i += 1;
            }
        }
    }
    pairs
}

fn line_bg(line: &Option<DiffLine>, add_bg: Color32, del_bg: Color32) -> Color32 {
    match line {
        None    => Color32::TRANSPARENT,
        Some(l) => match l.kind {
            DiffLineKind::Added   => add_bg,
            DiffLineKind::Deleted => del_bg,
            DiffLineKind::Context => Color32::TRANSPARENT,
        },
    }
}

fn line_style(l: &DiffLine, c: &Colors) -> (Color32, &'static str) {
    match l.kind {
        DiffLineKind::Added   => (c.add_fg, "+"),
        DiffLineKind::Deleted => (c.del_fg, "-"),
        DiffLineKind::Context => (c.ctx_fg, " "),
    }
}
