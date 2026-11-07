//! Faithful native rendering of the Compact Plus composition.
use super::*;
use chrono::{Datelike, Local};
use egui::{Align2, FontId, Pos2, Rect, Response, Sense, Ui, UiBuilder};

const TEXT: Color32 = Color32::from_rgb(244, 241, 248);
const MODAL: Color32 = Color32::from_rgb(16, 13, 24);
fn ink(ui: &Ui) -> Color32 {
    ui.visuals().text_color()
}
fn label_at(ui: &Ui, pos: Pos2, text: impl Into<String>, size: f32, c: Color32) {
    let text = text.into();
    let family = if size >= 50. {
        "InterBold"
    } else if size == 30. {
        "InterMedium"
    } else if (14.0..=18.0).contains(&size)
        || size == 28.
        || (size >= 20. && text.chars().next().is_some_and(|c| c.is_alphabetic()))
    {
        "InterSemiBold"
    } else if size >= 20. {
        "InterMedium"
    } else {
        ""
    };
    ui.painter().text(
        pos,
        Align2::LEFT_TOP,
        text,
        if family.is_empty() {
            FontId::proportional(size)
        } else {
            FontId::new(size, egui::FontFamily::Name(family.into()))
        },
        c,
    );
}
fn caps(ui: &Ui, pos: Pos2, text: &str, size: f32, c: Color32, spacing: f32) {
    let mut job = egui::text::LayoutJob::default();
    job.append(
        text,
        0.,
        egui::TextFormat {
            font_id: FontId::proportional(size),
            color: c,
            extra_letter_spacing: spacing,
            ..Default::default()
        },
    );
    let galley = ui.painter().layout_job(job);
    ui.painter().galley(pos, galley, c);
}
fn line(ui: &Ui, y: f32) {
    ui.painter()
        .hline(ui.max_rect().x_range(), y, Stroke::new(1.0_f32, BORDER));
}
fn hit(ui: &mut Ui, r: Rect, id: impl std::hash::Hash, label: &str, enabled: bool) -> Response {
    let response = ui.interact(
        r,
        ui.id().with(id),
        if enabled {
            Sense::click()
        } else {
            Sense::hover()
        },
    );
    response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, enabled, label));
    response
}
fn chevron(ui: &Ui, c: Pos2, cvalue: Color32, down: bool) {
    let offsets = if down {
        [(-3., -1.5), (0., 1.5), (3., -1.5)]
    } else {
        [(-1.5, -3.), (1.5, 0.), (-1.5, 3.)]
    };
    ui.painter().add(egui::Shape::line(
        offsets
            .into_iter()
            .map(|(x, y)| c + Vec2::new(x, y))
            .collect(),
        Stroke::new(1.2_f32, cvalue),
    ));
}
fn slider(ui: &mut Ui, value: u32, min: u32, max: u32, accent: Color32) -> u32 {
    let (r, mut response) = ui.allocate_exact_size(
        Vec2::new(ui.available_width(), 28.),
        Sense::click_and_drag(),
    );
    let mut new_value = value.clamp(min, max);
    let left = r.left() + 7.;
    let right = r.right() - 7.;
    if ui.is_enabled() {
        if response.clicked() || response.dragged() {
            response.request_focus();
            if let Some(p) = response.interact_pointer_pos() {
                new_value = (min as f32
                    + ((p.x - left) / (right - left)).clamp(0., 1.) * (max - min) as f32)
                    .round() as u32;
            }
        }
        if response.has_focus() {
            if ui.input(|i| i.key_pressed(egui::Key::ArrowLeft)) {
                new_value = new_value.saturating_sub(1).max(min);
            }
            if ui.input(|i| i.key_pressed(egui::Key::ArrowRight)) {
                new_value = (new_value + 1).min(max);
            }
        }
    }
    if new_value != value {
        response.mark_changed();
    }
    response.widget_info(|| {
        egui::WidgetInfo::labeled(
            egui::WidgetType::Slider,
            ui.is_enabled(),
            new_value.to_string(),
        )
    });
    let center_y = r.center().y;
    let x = left + (right - left) * (new_value - min) as f32 / (max - min) as f32;
    ui.painter().rect_filled(
        Rect::from_min_max(
            Pos2::new(left, center_y - 3.),
            Pos2::new(right, center_y + 3.),
        ),
        3.,
        BORDER,
    );
    ui.painter().rect_filled(
        Rect::from_min_max(Pos2::new(left, center_y - 3.), Pos2::new(x, center_y + 3.)),
        3.,
        accent,
    );
    ui.painter()
        .circle_filled(Pos2::new(x, center_y), 7., accent);
    new_value
}
fn outline(
    ui: &mut Ui,
    r: Rect,
    id: impl std::hash::Hash,
    name: &str,
    selected: bool,
    enabled: bool,
    accent: Color32,
) -> Response {
    let resp = hit(ui, r, id, name, enabled);
    let hovered = resp.hovered() && enabled;
    ui.painter().rect(
        r,
        8.,
        if selected || hovered {
            accent.gamma_multiply(0.1)
        } else {
            Color32::TRANSPARENT
        },
        Stroke::new(
            1.0_f32,
            if selected || hovered {
                accent.gamma_multiply(0.65)
            } else {
                BORDER
            },
        ),
        egui::StrokeKind::Inside,
    );
    ui.painter().text(
        r.center(),
        Align2::CENTER_CENTER,
        name,
        FontId::proportional(13.),
        if !enabled {
            MUTED.gamma_multiply(0.45)
        } else if selected {
            accent
        } else {
            ink(ui)
        },
    );
    resp
}
fn tab(ui: &mut Ui, id: impl std::hash::Hash, name: &str, selected: bool, accent: Color32) -> bool {
    let width = ui
        .painter()
        .layout_no_wrap(name.into(), FontId::proportional(13.), TEXT)
        .size()
        .x;
    let (r, _) = ui.allocate_exact_size(Vec2::new(width, 44.), Sense::hover());
    let response = hit(ui, r, id, name, true);
    ui.painter().text(
        r.left_center(),
        Align2::LEFT_CENTER,
        name,
        FontId::proportional(13.),
        if selected { ink(ui) } else { MUTED },
    );
    if selected {
        ui.painter()
            .hline(r.x_range(), r.bottom() - 1., Stroke::new(2.0_f32, accent));
    }
    response.clicked()
}
fn section(ui: &mut Ui, name: &str, help: &str, body: impl FnOnce(&mut Ui)) {
    Frame::new()
        .inner_margin(Margin::symmetric(0, 25))
        .show(ui, |ui| {
            ui.horizontal_top(|ui| {
                ui.allocate_ui_with_layout(
                    Vec2::new(170., 0.),
                    Layout::top_down(Align::Min),
                    |ui| {
                        ui.set_min_width(170.);
                        ui.label(
                            RichText::new(name)
                                .size(12.)
                                .family(egui::FontFamily::Name("InterSemiBold".into())),
                        );
                        ui.add_space(9.);
                        secondary(ui, help);
                    },
                );
                ui.add_space(10.);
                ui.vertical(|ui| {
                    ui.set_width(ui.available_width());
                    body(ui);
                });
            });
        });
    ui.separator();
}
fn modal(ctx: &egui::Context, id: &str, padding: i8) -> egui::Modal {
    egui::Modal::new(egui::Id::new(id))
        .backdrop_color(Color32::from_rgba_unmultiplied(3, 2, 7, 190))
        .frame(
            Frame::new()
                .fill(MODAL)
                .stroke(Stroke::new(1.0_f32, Color32::from_rgb(56, 43, 75)))
                .corner_radius(18)
                .inner_margin(padding),
        )
        .area(
            egui::Modal::default_area(egui::Id::new(id))
                .default_width((ctx.content_rect().width() - 80.).min(580.)),
        )
}
fn modal_heading(ui: &mut Ui, eyebrow: &str, title: &str, accent: Color32) -> bool {
    let (r, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 56.), Sense::hover());
    caps(ui, r.min, eyebrow, 10., accent, 2.4);
    label_at(ui, r.min + Vec2::new(0., 19.), title, 25., TEXT);
    let close = hit(
        ui,
        Rect::from_center_size(r.right_top() + Vec2::new(-12., 23.), Vec2::splat(32.)),
        "close-modal",
        "Закрыть",
        true,
    );
    let c = close.rect.center();
    let stroke = Stroke::new(1.5_f32, if close.hovered() { TEXT } else { MUTED });
    ui.painter()
        .line_segment([c + Vec2::splat(-4.), c + Vec2::splat(4.)], stroke);
    ui.painter()
        .line_segment([c + Vec2::new(-4., 4.), c + Vec2::new(4., -4.)], stroke);
    close.clicked()
}
fn ellipse(ui: &Ui, c: Pos2, radii: Vec2, angle: f32, cvalue: Color32) {
    let points = (0..=180)
        .map(|i| {
            let a = i as f32 / 180. * std::f32::consts::TAU;
            let x = a.cos() * radii.x;
            let y = a.sin() * radii.y;
            c + Vec2::new(
                x * angle.cos() - y * angle.sin(),
                x * angle.sin() + y * angle.cos(),
            )
        })
        .collect();
    ui.painter()
        .add(egui::Shape::line(points, Stroke::new(1.0_f32, cvalue)));
}
fn downloads(n: u64) -> String {
    if n >= 1_000_000 {
        format!("{:.1} млн", n as f64 / 1_000_000.).replace('.', ",")
    } else if n >= 1000 {
        format!("{:.0} тыс.", n as f64 / 1000.)
    } else {
        n.to_string()
    }
}

impl Launcher {
    pub(super) fn theme(&self, ctx: &egui::Context) {
        let accent = color(&self.settings.accent);
        let mut style = egui::Style::default();
        let light = self.settings.theme == "light";
        style.visuals = if light {
            egui::Visuals::light()
        } else {
            egui::Visuals::dark()
        };
        style.visuals.panel_fill = match self.settings.theme.as_str() {
            "dark" => Color32::from_rgb(19, 17, 25),
            "midnight" => Color32::from_rgb(8, 15, 28),
            "light" => Color32::from_rgb(246, 244, 249),
            _ => BG,
        };
        style.visuals.override_text_color = Some(if light {
            Color32::from_rgb(28, 23, 37)
        } else {
            TEXT
        });
        style.visuals.window_fill = MODAL;
        style.visuals.extreme_bg_color = Color32::TRANSPARENT;
        style.visuals.selection.bg_fill = accent.gamma_multiply(0.16);
        style.visuals.selection.stroke = Stroke::new(1.0_f32, accent);
        for (ty, size) in [
            (egui::TextStyle::Body, 14.),
            (egui::TextStyle::Button, 13.),
            (egui::TextStyle::Small, 12.),
        ] {
            style.text_styles.insert(ty, FontId::proportional(size));
        }
        style.spacing.item_spacing = Vec2::new(10., 8.);
        style.spacing.button_padding = Vec2::new(16., 10.);
        style.spacing.interact_size = Vec2::new(40., 36.);
        style.spacing.slider_width = 240.;
        style.visuals.widgets.inactive.bg_fill = Color32::TRANSPARENT;
        style.visuals.widgets.inactive.weak_bg_fill = Color32::TRANSPARENT;
        style.visuals.widgets.inactive.bg_stroke = Stroke::new(1.0_f32, BORDER);
        style.visuals.widgets.hovered.bg_fill = accent.gamma_multiply(0.12);
        style.visuals.widgets.hovered.weak_bg_fill = accent.gamma_multiply(0.12);
        style.visuals.widgets.hovered.bg_stroke = Stroke::new(1.0_f32, accent);
        style.visuals.widgets.active = style.visuals.widgets.hovered;
        style.visuals.widgets.noninteractive.bg_fill = Color32::TRANSPARENT;
        style.visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0_f32, BORDER);
        for w in [
            &mut style.visuals.widgets.inactive,
            &mut style.visuals.widgets.hovered,
            &mut style.visuals.widgets.active,
        ] {
            w.corner_radius = CornerRadius::same(8);
        }
        style.visuals.window_corner_radius = CornerRadius::same(18);
        style.visuals.window_highlight_topmost = false;
        ctx.set_style(style);
    }
    pub(super) fn sidebar(&mut self, ui: &mut Ui) {
        let r = ui.max_rect();
        let accent = color(&self.settings.accent);
        let fg = ink(ui);
        ui.painter().rect(
            r,
            16.,
            Color32::TRANSPARENT,
            Stroke::new(1.0_f32, BORDER),
            egui::StrokeKind::Inside,
        );
        label_at(ui, r.min + Vec2::new(22., 14.), "K", 28., fg);
        label_at(ui, r.min + Vec2::new(53., 16.), "/", 25., accent);
        label_at(ui, r.min + Vec2::new(74., 21.), "KYRO", 15., fg);
        for (i, name) in MENU.iter().enumerate() {
            let row = Rect::from_min_size(
                r.min + Vec2::new(11., 59. + i as f32 * 50.),
                Vec2::new(r.width() - 22., 46.),
            );
            let selected = self.page == i;
            let resp = hit(ui, row, ("nav", i), name, true);
            if selected || resp.hovered() {
                ui.painter().rect(
                    row,
                    9.,
                    accent.gamma_multiply(if selected { 0.06 } else { 0.1 }),
                    Stroke::new(
                        1.0_f32,
                        accent.gamma_multiply(if selected { 0.4 } else { 0.65 }),
                    ),
                    egui::StrokeKind::Inside,
                );
            }
            draw_icon(
                ui.painter(),
                row.left_center() + Vec2::new(21., 0.),
                i,
                if selected { fg } else { MUTED },
            );
            label_at(
                ui,
                row.left_top() + Vec2::new(44., 14.),
                *name,
                14.,
                if selected { fg } else { MUTED },
            );
            if resp.clicked() {
                self.navigate(i);
            }
        }
        let profile = Rect::from_min_size(
            Pos2::new(r.left() + 11., r.bottom() - 65.),
            Vec2::new(r.width() - 22., 53.),
        );
        let pr = hit(ui, profile, "profile", "Аккаунты", true);
        ui.painter().rect(
            profile,
            12.,
            accent.gamma_multiply(0.05),
            Stroke::new(1.0_f32, if pr.hovered() { accent } else { BORDER }),
            egui::StrokeKind::Inside,
        );
        let avatar = Rect::from_min_size(profile.min + Vec2::new(9., 10.), Vec2::splat(34.));
        ui.painter().rect(
            avatar,
            9.,
            accent.gamma_multiply(0.11),
            Stroke::new(1.0_f32, accent.gamma_multiply(0.4)),
            egui::StrokeKind::Inside,
        );
        let name = text(&self.account, "name");
        ui.painter().text(
            avatar.center(),
            Align2::CENTER_CENTER,
            name.chars()
                .next()
                .map(|c| c.to_string())
                .unwrap_or("+".into()),
            FontId::proportional(18.),
            accent,
        );
        label_at(
            ui,
            profile.min + Vec2::new(54., 8.),
            if name.is_empty() {
                "Аккаунты"
            } else {
                &name
            },
            14.,
            fg,
        );
        label_at(
            ui,
            profile.min + Vec2::new(54., 28.),
            if self.account["kind"] == "offline" {
                "Офлайн-аккаунт"
            } else if self.account["kind"] == "microsoft" {
                "Microsoft"
            } else {
                "Не выбран"
            },
            11.,
            MUTED,
        );
        chevron(ui, profile.right_center() - Vec2::new(12., 0.), MUTED, true);
        if pr.clicked() {
            if self.dirty.is_some() {
                self.save();
            }
            self.account_open = true;
        }
        let update = Rect::from_min_size(
            profile.min - Vec2::new(0., 60.),
            Vec2::new(profile.width(), 52.),
        );
        let count = array(&self.mods, "files")
            .iter()
            .filter(|m| !m["update"].is_null())
            .count();
        let resp = hit(ui, update, "mod-updates", "Обновления модов", true);
        ui.painter().rect(
            update,
            12.,
            accent.gamma_multiply(if count > 0 { 0.1 } else { 0.04 }),
            Stroke::new(
                1.0_f32,
                if resp.hovered() || count > 0 {
                    accent.gamma_multiply(0.6)
                } else {
                    BORDER
                },
            ),
            egui::StrokeKind::Inside,
        );
        label_at(
            ui,
            update.min + Vec2::new(10., 8.),
            "Обновления модов",
            11.,
            MUTED,
        );
        let status = if self.pending.contains("scan") {
            "Проверяем…".into()
        } else if !text(&self.mods, "warning").is_empty() {
            "Не удалось проверить".into()
        } else if count > 0 {
            format!("Доступно {count} {}", plural(count))
        } else if array(&self.mods, "files").is_empty() {
            "Нет модов в профиле".into()
        } else {
            "Все моды актуальны".into()
        };
        label_at(
            ui,
            update.min + Vec2::new(10., 28.),
            status,
            12.,
            if count > 0 { accent } else { fg },
        );
        label_at(
            ui,
            update.right_top() + Vec2::new(-18., 9.),
            "↗",
            14.,
            accent,
        );
        if resp.clicked() {
            self.navigate(1);
            self.catalog_tab = 1;
            self.scan();
        }
    }
    fn core_picker(&mut self, ui: &mut Ui, r: Rect, large: bool) {
        let accent = color(&self.settings.accent);
        let enabled = !self.busy() && self.settings.instance_id.is_empty();
        let resp = hit(
            ui,
            r,
            if large { "home-core" } else { "catalog-core" },
            "Ядро Minecraft",
            enabled,
        );
        if large {
            ui.painter().hline(
                r.x_range(),
                r.bottom(),
                Stroke::new(1.0_f32, if resp.hovered() { accent } else { BORDER }),
            );
            label_at(
                ui,
                r.min + Vec2::new(0., 3.),
                &self.settings.loader,
                30.,
                accent,
            );
        } else {
            ui.painter().rect_stroke(
                r,
                8.,
                Stroke::new(1.0_f32, BORDER),
                egui::StrokeKind::Inside,
            );
            label_at(
                ui,
                r.min + Vec2::new(13., 11.),
                &self.settings.loader,
                13.,
                ink(ui),
            );
        }
        chevron(ui, r.right_center() - Vec2::new(9., 0.), accent, true);
        if enabled {
            if large && self.smoke.as_ref().is_some_and(|s| s.step == 12) {
                egui::Popup::open_id(ui.ctx(), egui::Popup::default_response_id(&resp));
            }
            egui::Popup::from_toggle_button_response(&resp)
                .width(r.width())
                .gap(10.)
                .frame(
                    Frame::new()
                        .fill(MODAL)
                        .stroke(Stroke::new(1.0_f32, BORDER))
                        .corner_radius(12)
                        .inner_margin(8),
                )
                .show(|ui| {
                    ui.set_width((r.width() - 16.).max(120.));
                    for core in crate::cores::LOADERS {
                        let (row, _) = ui.allocate_exact_size(
                            Vec2::new(ui.available_width(), 44.),
                            Sense::hover(),
                        );
                        let selected = self.settings.loader == core;
                        let response = hit(ui, row, core, core, true);
                        if selected || response.hovered() {
                            ui.painter()
                                .rect_filled(row, 9., accent.gamma_multiply(0.11));
                        }
                        let tag = Rect::from_min_size(
                            row.min + Vec2::new(12., 9.),
                            Vec2::new(core.len() as f32 * 7.2 + 20., 26.),
                        );
                        ui.painter()
                            .rect_filled(tag, 7., accent.gamma_multiply(0.1));
                        label_at(ui, tag.min + Vec2::new(10., 5.), core, 13., accent);
                        if selected {
                            label_at(
                                ui,
                                row.right_center() + Vec2::new(-22., -8.),
                                "✓",
                                15.,
                                accent,
                            );
                        }
                        if response.clicked() {
                            self.settings.loader = core.into();
                            if self.save() {
                                self.profile_changed();
                            }
                            ui.close();
                        }
                    }
                });
        }
    }
    pub(super) fn home(&mut self, ui: &mut Ui) {
        let r = ui.max_rect();
        let accent = color(&self.settings.accent);
        let fg = ink(ui);
        let card_layout = self.settings.home_layout == "card";
        let bg = self
            .data
            .modpacks
            .iter()
            .find(|p| p.id == self.settings.instance_id)
            .map(|p| p.cover_image.clone())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| self.settings.home_background.clone());
        if let Some(uri) = include_data(ui.ctx(), &bg, "home-background") {
            ui.place(
                r,
                egui::Image::new(uri)
                    .fit_to_exact_size(r.size())
                    .tint(Color32::from_white_alpha(60)),
            );
        }
        if !card_layout {
            let c = Pos2::new(r.left() + r.width() * 0.85, r.top() + r.height() * 0.45);
            ellipse(
                ui,
                c,
                Vec2::new(350., 180.),
                -34_f32.to_radians(),
                accent.gamma_multiply(0.48),
            );
            ellipse(
                ui,
                c,
                Vec2::new(310., 145.),
                -17_f32.to_radians(),
                accent.gamma_multiply(0.08),
            );
            ui.painter()
                .circle_filled(c + Vec2::new(-190., -80.), 2.5, accent);
            caps(
                ui,
                Pos2::new(r.left() + r.width() * 0.69, r.top() + r.height() * 0.51),
                "JAVA EDITION / 01",
                10.,
                MUTED,
                2.,
            );
        }
        let packs = !self.data.modpacks.is_empty();
        let height = if packs { 404. } else { 360. };
        let x = if card_layout {
            r.center().x - 190.
        } else {
            r.left() + 16.
        };
        let top = r.top() + ((r.height() - height) / 2. - 10.).max(5.);
        let extra = if packs { 70. } else { 0. };
        if card_layout {
            ui.painter().rect(
                Rect::from_min_size(Pos2::new(x - 22., top - 22.), Vec2::new(424., height + 44.)),
                20.,
                ui.visuals().panel_fill,
                Stroke::new(1.0_f32, BORDER),
                egui::StrokeKind::Inside,
            );
        }
        caps(
            ui,
            Pos2::new(x, top),
            "MINECRAFT · JAVA EDITION",
            11.,
            MUTED,
            2.5,
        );
        if packs {
            label_at(ui, Pos2::new(x, top + 30.), "Игровой профиль", 11., MUTED);
            let p = Rect::from_min_size(Pos2::new(x, top + 50.), Vec2::new(340., 40.));
            let name = self
                .data
                .modpacks
                .iter()
                .find(|p| p.id == self.settings.instance_id)
                .map(|p| p.name.clone())
                .unwrap_or("Обычный профиль".into());
            let response = outline(ui, p, "pack-selector", &name, false, !self.busy(), accent);
            egui::Popup::from_toggle_button_response(&response)
                .width(340.)
                .frame(
                    Frame::new()
                        .fill(MODAL)
                        .stroke(Stroke::new(1.0_f32, BORDER))
                        .corner_radius(12)
                        .inner_margin(8),
                )
                .show(|ui| {
                    if ui.button("Обычный профиль").clicked() {
                        self.settings.instance_id.clear();
                        self.settings.loader_version.clear();
                        if self.save() {
                            self.profile_changed();
                        }
                        ui.close();
                    }
                    for pack in self.data.modpacks.clone() {
                        if ui.button(&pack.name).clicked() {
                            let result = crate::modpacks::select_modpack(self.app.state(), pack.id);
                            self.result(result, "Сборка выбрана");
                            self.profile_changed();
                            ui.close();
                        }
                    }
                });
        }
        let version = Rect::from_min_size(
            Pos2::new(x, top + if packs { 41. } else { 32. } + extra),
            Vec2::new(360., 88.),
        );
        label_at(
            ui,
            version.min,
            &self.settings.version,
            if packs { 60. } else { 68. },
            fg,
        );
        if hit(
            ui,
            version,
            "home-version",
            "Версия Minecraft",
            !self.busy(),
        )
        .clicked()
        {
            self.open_versions();
        }
        self.core_picker(
            ui,
            Rect::from_min_size(Pos2::new(x, top + 118. + extra), Vec2::new(240., 56.)),
            true,
        );
        let mini = Rect::from_min_size(Pos2::new(x, top + 188. + extra), Vec2::new(96., 32.));
        label_at(
            ui,
            mini.min + Vec2::new(0., 7.),
            &self.settings.version,
            13.,
            fg,
        );
        chevron(ui, mini.right_center() - Vec2::new(8., 0.), MUTED, true);
        if hit(ui, mini, "version-mini", "Версии Minecraft", !self.busy()).clicked() {
            self.open_versions();
        }
        ui.painter().vline(
            x + 104.,
            (mini.top() + 6.0)..=(mini.bottom() - 6.0),
            Stroke::new(1.0_f32, BORDER),
        );
        let all = Rect::from_min_size(Pos2::new(x + 116., mini.top()), Vec2::new(94., 32.));
        label_at(ui, all.min + Vec2::new(0., 7.), "ВСЕ ВЕРСИИ", 12., MUTED);
        if hit(ui, all, "all-versions", "Все версии", !self.busy()).clicked() {
            self.open_versions();
        }
        let play = Rect::from_min_size(Pos2::new(x, top + 238. + extra), Vec2::new(256., 60.));
        let boost = Rect::from_min_size(Pos2::new(x + 218., mini.top()), Vec2::new(80., 32.));
        let enabled_mods = if self.mod_scope == super::scope(&self.settings)
            && self.settings.loader != "Vanilla"
        {
            crate::fps_boost::enabled_count(&self.mods)
        } else {
            0
        };
        let boost_active = enabled_mods > 0;
        if boost_active {
            for spread in (1..=4).rev() {
                ui.painter().rect_stroke(
                    boost.expand(spread as f32),
                    9. + spread as f32,
                    Stroke::new(1.0_f32, accent.gamma_multiply(0.025)),
                    egui::StrokeKind::Outside,
                );
            }
            ui.painter().rect(
                boost,
                9.,
                accent.gamma_multiply(0.09),
                Stroke::new(1.0_f32, accent.gamma_multiply(0.8)),
                egui::StrokeKind::Inside,
            );
            ui.painter()
                .circle_filled(boost.right_center() - Vec2::new(7., 0.), 2., accent);
        }
        let boost_response = hit(ui, boost, "fps-boost", "FPS-буст", !self.busy())
            .on_hover_text(if boost_active { format!("FPS-буст активен · включено модов оптимизации: {enabled_mods}. Нажми, чтобы проверить набор.") } else { "Подобрать моды оптимизации для выбранной версии и ядра".into() });
        label_at(
            ui,
            boost.min + Vec2::new(if boost_active { 6. } else { 0. }, 7.),
            "FPS-буст",
            12.,
            if boost_response.hovered() || boost_active {
                accent
            } else {
                MUTED
            },
        );
        if boost_response.clicked() && self.save() {
            self.fps_open = true;
            self.fps_plan = serde_json::Value::Null;
            self.submit("fps-plan", Job::FpsPlan);
        }
        let folder_rect = Rect::from_min_size(Pos2::new(x + 306., mini.top()), Vec2::new(68., 32.));
        let folder_response = hit(ui, folder_rect, "game-folder", "Папка", true)
            .on_hover_text("Открыть папку игры выбранного профиля или сборки");
        let folder_color = if folder_response.hovered() {
            accent
        } else {
            MUTED
        };
        label_at(
            ui,
            folder_rect.min + Vec2::new(23., 7.),
            "Папка",
            12.,
            folder_color,
        );
        let folder_icon = folder_rect.left_center() + Vec2::new(8., 0.);
        ui.painter().add(egui::Shape::line(
            vec![
                folder_icon + Vec2::new(-7., -5.),
                folder_icon + Vec2::new(-1., -5.),
                folder_icon + Vec2::new(1., -2.),
                folder_icon + Vec2::new(7., -2.),
                folder_icon + Vec2::new(7., 6.),
                folder_icon + Vec2::new(-7., 6.),
                folder_icon + Vec2::new(-7., -5.),
            ],
            Stroke::new(
                1.2_f32,
                if folder_response.hovered() {
                    accent
                } else {
                    MUTED
                },
            ),
        ));
        if folder_response.clicked() {
            let result = crate::open_folder(self.app.state(), "instance".into()).map(|_| ());
            self.result(result, "Папка игры открыта");
        }
        let can_cancel = self.app.launch.can_cancel();
        let enabled = can_cancel || (!self.busy() && !self.pending.contains("preflight"));
        let response = hit(
            ui,
            play,
            "play",
            if can_cancel {
                "Отменить"
            } else {
                "Играть"
            },
            enabled,
        );
        for spread in (1..=7).rev() {
            ui.painter().rect_filled(
                play.expand(spread as f32),
                30. + spread as f32,
                accent.gamma_multiply(0.008),
            );
        }
        ui.painter().rect_filled(
            play,
            30.,
            if enabled {
                accent
            } else {
                accent.gamma_multiply(0.4)
            },
        );
        let text_color = if accent.r() as u32 + accent.g() as u32 + accent.b() as u32 > 430 {
            BG
        } else {
            TEXT
        };
        let c = play.center();
        if can_cancel {
            let cross = c + Vec2::new(-37., 0.);
            ui.painter().line_segment(
                [cross - Vec2::splat(5.), cross + Vec2::splat(5.)],
                Stroke::new(1.8_f32, text_color),
            );
            ui.painter().line_segment(
                [cross + Vec2::new(-5., 5.), cross + Vec2::new(5., -5.)],
                Stroke::new(1.8_f32, text_color),
            );
        } else {
            ui.painter().add(egui::Shape::convex_polygon(
                vec![
                    c + Vec2::new(-42., -6.),
                    c + Vec2::new(-42., 6.),
                    c + Vec2::new(-32., 0.),
                ],
                text_color,
                Stroke::NONE,
            ));
        }
        label_at(
            ui,
            c + Vec2::new(-12., -10.),
            if can_cancel {
                "Отменить"
            } else if self.app.launch.is_cancelling() {
                "Отмена…"
            } else if self.app.launch.is_running() {
                "В игре"
            } else if self.busy() {
                "Запуск…"
            } else {
                "Играть"
            },
            16.,
            text_color,
        );
        if response.clicked() && can_cancel {
            self.app.launch.cancel();
        } else if response.clicked() && self.save() {
            self.submit("preflight", Job::Compatibility);
        }
        let note = if !self.settings.instance_id.is_empty() {
            "Выбрана сборка. Нажми «Играть», чтобы запустить её."
        } else if self.account.is_null() {
            "Добавь офлайн-аккаунт по нику или войди через Microsoft — нажми на профиль внизу меню."
        } else {
            "Всё готово к твоей следующей истории."
        };
        let mut note_ui = ui.new_child(
            UiBuilder::new()
                .id_salt("home-note")
                .max_rect(Rect::from_min_size(
                    Pos2::new(x, top + 314. + extra),
                    Vec2::new(360., 58.),
                ))
                .layout(Layout::top_down(Align::Min)),
        );
        note_ui.add(egui::Label::new(RichText::new(note).size(13.).color(MUTED)).wrap());
    }
    pub(super) fn draw(&mut self, ctx: &egui::Context) {
        self.drain(ctx);
        self.theme(ctx);
        if ctx.input(|i| i.viewport().close_requested()) && self.busy() {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.error = Some("Сначала закрой Minecraft или дождись завершения установки.".into());
        }
        egui::CentralPanel::default()
            .frame(Frame::new().fill(ctx.style().visuals.panel_fill))
            .show(ctx, |root| {
                let r = root.max_rect();
                let accent = color(&self.settings.accent);
                let sidebar = Rect::from_min_max(
                    r.min + Vec2::new(12., 12.),
                    Pos2::new(r.left() + 210., r.bottom() - 12.),
                );
                let mut side =
                    root.new_child(UiBuilder::new().id_salt("sidebar").max_rect(sidebar));
                self.sidebar(&mut side);
                let header = Rect::from_min_max(
                    Pos2::new(r.left() + 210., r.top()),
                    Pos2::new(r.right(), r.top() + 48.),
                );
                let mut ui = root.new_child(UiBuilder::new().id_salt("chrome").max_rect(header));
                caps(
                    &ui,
                    header.min + Vec2::new(34., 18.),
                    if self.page == 0 {
                        "MINECRAFT · JAVA EDITION"
                    } else {
                        [
                            "",
                            "KYRO / КАТАЛОГ",
                            "KYRO / СКРИНШОТЫ",
                            "KYRO / ДИАГНОСТИКА",
                            "KYRO / АКТИВНОСТЬ",
                            "KYRO / УВЕДОМЛЕНИЯ",
                            "KYRO / НАСТРОЙКИ",
                        ][self.page]
                    },
                    10.,
                    MUTED.gamma_multiply(0.8),
                    1.4,
                );
                let drag = hit(
                    &mut ui,
                    Rect::from_min_max(header.min, Pos2::new(r.right() - 138., header.bottom())),
                    "drag",
                    "Переместить окно",
                    true,
                );
                let drag = ui.interact(drag.rect, ui.id().with("window-drag"), Sense::drag());
                if drag.drag_started() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
                }
                for i in 0..3 {
                    let wr = Rect::from_min_size(
                        Pos2::new(r.right() - 138. + i as f32 * 46., r.top()),
                        Vec2::new(46., 48.),
                    );
                    let resp = hit(
                        &mut ui,
                        wr,
                        ("control", i),
                        ["Свернуть", "Развернуть", "Закрыть"][i],
                        true,
                    );
                    if resp.hovered() {
                        ui.painter().rect_filled(
                            wr,
                            0.,
                            if i == 2 {
                                Color32::from_rgb(129, 52, 68)
                            } else {
                                accent.gamma_multiply(0.08)
                            },
                        );
                    }
                    let c = wr.center();
                    let stroke = Stroke::new(1.3_f32, MUTED);
                    match i {
                        0 => {
                            ui.painter().line_segment(
                                [c - Vec2::new(4., 0.), c + Vec2::new(4., 0.)],
                                stroke,
                            );
                        }
                        1 => {
                            ui.painter().rect_stroke(
                                Rect::from_center_size(c, Vec2::splat(11.)),
                                0.,
                                stroke,
                                egui::StrokeKind::Inside,
                            );
                        }
                        _ => {
                            ui.painter()
                                .line_segment([c - Vec2::splat(3.5), c + Vec2::splat(3.5)], stroke);
                            ui.painter().line_segment(
                                [c + Vec2::new(-3.5, 3.5), c + Vec2::new(3.5, -3.5)],
                                stroke,
                            );
                        }
                    }
                    if resp.clicked() {
                        match i {
                            0 => ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true)),
                            1 => ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(
                                !ctx.input(|i| i.viewport().maximized.unwrap_or(false)),
                            )),
                            _ => {
                                if self.busy() {
                                    self.error =
                                        Some("Дождись завершения операции или закрой игру.".into());
                                } else {
                                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                                }
                            }
                        }
                    }
                }
                let ornament_rect = Rect::from_min_max(
                    Pos2::new(r.left() + 210., r.top()),
                    Pos2::new(r.right(), r.bottom() - 44.),
                );
                let ornament =
                    root.new_child(UiBuilder::new().id_salt("ornament").max_rect(ornament_rect));
                let c = Pos2::new(r.right() - 80., r.top() - 65.);
                ellipse(
                    &ornament,
                    c,
                    Vec2::new(320., 205.),
                    -25_f32.to_radians(),
                    accent.gamma_multiply(0.07),
                );
                ornament.painter().circle_filled(
                    c + Vec2::new(-93.3, 243.8),
                    2.5,
                    accent.gamma_multiply(0.45),
                );
                let page = Rect::from_min_max(
                    Pos2::new(r.left() + 235., r.top() + 62.),
                    Pos2::new(r.right() - 25., r.bottom() - 54.),
                );
                let mut ui = root.new_child(
                    UiBuilder::new()
                        .id_salt("page")
                        .max_rect(page)
                        .layout(Layout::top_down(Align::Min)),
                );
                ui.set_clip_rect(page);
                if self.page == 0 {
                    self.home(&mut ui);
                } else {
                    egui::ScrollArea::vertical()
                        .id_salt(("page-scroll", self.page, self.settings_tab))
                        .auto_shrink([false, false])
                        .show(&mut ui, |ui| {
                            ui.push_id(self.page, |ui| match self.page {
                                1 => self.catalog(ui),
                                2 => self.screenshots(ui),
                                3 => self.diagnostics(ui),
                                4 => self.activity(ui),
                                5 => self.notifications(ui),
                                6 => self.settings(ui),
                                _ => {}
                            });
                        });
                }
                let footer = Rect::from_min_max(
                    Pos2::new(r.left() + 235., r.bottom() - 44.),
                    Pos2::new(r.right() - 25., r.bottom()),
                );
                egui::Area::new(egui::Id::new("footer-area"))
                    .order(egui::Order::Middle)
                    .fixed_pos(footer.min)
                    .interactable(false)
                    .show(ctx, |ui| {
                        ui.set_min_size(footer.size());
                        ui.set_clip_rect(footer);
                        ui.painter()
                            .rect_filled(footer, 0., ui.visuals().panel_fill);
                        line(ui, footer.top());
                        caps(
                            ui,
                            footer.min + Vec2::new(0., 17.),
                            &format!(
                                "KYRO   /   {} · {}",
                                self.settings.version,
                                self.settings.loader.to_uppercase()
                            ),
                            10.,
                            MUTED,
                            0.5,
                        );
                        let status = format!(
                            "{} ГБ RAM     ·     {}",
                            self.settings.memory_gb,
                            if self.busy() {
                                "ПОДГОТОВКА"
                            } else {
                                "JAVA EDITION"
                            }
                        );
                        ui.painter().text(
                            footer.right_top() + Vec2::new(0., 22.),
                            Align2::RIGHT_CENTER,
                            status,
                            FontId::proportional(10.),
                            MUTED,
                        );
                    });
                if self.busy() {
                    let cancelling = self.app.launch.is_cancelling();
                    let can_cancel = self.app.launch.can_cancel();
                    let show_cancel = can_cancel || cancelling;
                    let rect = Rect::from_min_size(
                        Pos2::new(
                            r.right() - 385.,
                            r.bottom() - if show_cancel { 175. } else { 135. },
                        ),
                        Vec2::new(350., if show_cancel { 105. } else { 65. }),
                    );
                    let mut ui =
                        root.new_child(UiBuilder::new().id_salt("operation").max_rect(rect));
                    Frame::new()
                        .fill(MODAL)
                        .stroke(Stroke::new(1.0_f32, BORDER))
                        .corner_radius(12)
                        .inner_margin(14)
                        .show(&mut ui, |ui| {
                            secondary(
                                ui,
                                if cancelling {
                                    "Остановка загрузки…"
                                } else {
                                    &self.progress.0
                                },
                            );
                            ui.add(egui::ProgressBar::new(self.progress.1).desired_width(320.));
                            if show_cancel
                                && ui
                                    .add_enabled(
                                        can_cancel,
                                        egui::Button::new(if cancelling {
                                            "Отмена…"
                                        } else {
                                            "Отменить запуск"
                                        }),
                                    )
                                    .clicked()
                            {
                                self.app.launch.cancel();
                            }
                        });
                }
            });
        self.dialogs(ctx);
        self.smoke_frame(ctx);
        ctx.request_repaint_after(Duration::from_millis(if self.busy() { 100 } else { 500 }));
    }
}
impl Launcher {
    pub(super) fn catalog(&mut self, ui: &mut Ui) {
        title(ui, "Каталог", "Проекты для твоего Minecraft");
        let accent = color(&self.settings.accent);
        let mut changed = false;
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 25.;
            for (i, name) in [
                "Сборки",
                "Моды",
                "Шейдеры",
                "Ресурспаки",
                "Мои моды",
                "Установленные",
            ]
            .iter()
            .enumerate()
            {
                let selected = if i < 4 {
                    self.catalog_tab == 0
                        && self.kind == ["modpack", "mod", "shader", "resourcepack"][i]
                } else {
                    self.catalog_tab == i - 3
                };
                if tab(ui, i, *name, selected, accent) {
                    if i < 4 {
                        self.catalog_tab = 0;
                        self.kind = ["modpack", "mod", "shader", "resourcepack"][i].into();
                        changed = true;
                    } else {
                        self.catalog_tab = i - 3;
                        if i == 4 {
                            self.scan();
                        }
                    }
                }
            }
        });
        ui.separator();
        ui.add_space(14.);
        if self.catalog_tab == 1 {
            self.mod_manager(ui);
            return;
        }
        if self.catalog_tab == 2 {
            if self.data.modpacks.is_empty() && self.data.installed.is_empty() {
                secondary(
                    ui,
                    "Здесь появятся установленные сборки, моды и ресурспаки.",
                );
            }
            for pack in self.data.modpacks.clone() {
                let (r, _) =
                    ui.allocate_exact_size(Vec2::new(ui.available_width(), 100.), Sense::hover());
                label_at(ui, r.min + Vec2::new(0., 12.), &pack.name, 17., ink(ui));
                label_at(
                    ui,
                    r.min + Vec2::new(0., 43.),
                    format!(
                        "{} · {} · {}",
                        pack.game_version, pack.loader, pack.version_name
                    ),
                    12.,
                    MUTED,
                );
                let b = Rect::from_min_size(
                    r.right_top() + Vec2::new(-145., 12.),
                    Vec2::new(145., 36.),
                );
                if outline(
                    ui,
                    b,
                    (&pack.id, "select"),
                    "Выбрать и играть",
                    false,
                    !self.busy(),
                    accent,
                )
                .clicked()
                {
                    let result = crate::modpacks::select_modpack(self.app.state(), pack.id.clone());
                    self.result(result, "Сборка выбрана");
                    self.profile_changed();
                    self.navigate(0);
                }
                let cover = Rect::from_min_size(b.min + Vec2::new(0., 44.), Vec2::new(145., 30.));
                if outline(
                    ui,
                    cover,
                    (&pack.id, "cover"),
                    "Обложка…",
                    false,
                    !self.busy(),
                    accent,
                )
                .clicked()
                {
                    if let Some(image) = pick_image() {
                        let result = crate::customization::set_pack_cover(
                            self.app.state(),
                            pack.id.clone(),
                            image,
                        );
                        self.result(result, "Обложка сохранена");
                    }
                }
                line(ui, r.bottom());
            }
            for project in self.data.installed.clone() {
                ui.add_space(12.);
                ui.label(
                    RichText::new(project.name)
                        .family(egui::FontFamily::Name("InterSemiBold".into())),
                );
                secondary(
                    ui,
                    format!(
                        "{} · {} · {}",
                        project.kind, project.game_version, project.loader
                    ),
                );
                ui.separator();
            }
            return;
        }
        let width = ui.available_width();
        let mut enter = false;
        Frame::new()
            .stroke(Stroke::new(1.0_f32, BORDER))
            .corner_radius(8)
            .inner_margin(Margin::symmetric(13, 0))
            .show(ui, |ui| {
                ui.set_min_width(width - 26.);
                ui.horizontal(|ui| {
                    let (icon, _) = ui.allocate_exact_size(Vec2::new(20., 40.), Sense::hover());
                    let c = icon.center();
                    ui.painter().circle_stroke(
                        c - Vec2::new(2., 2.),
                        6.,
                        Stroke::new(1.2_f32, MUTED),
                    );
                    ui.painter().line_segment(
                        [c + Vec2::new(2., 3.), c + Vec2::new(7., 8.)],
                        Stroke::new(1.2_f32, MUTED),
                    );
                    let field = ui.add(
                        egui::TextEdit::singleline(&mut self.query)
                            .frame(false)
                            .margin(Margin::symmetric(7, 11))
                            .hint_text("Найти на Modrinth")
                            .desired_width(width - 150.),
                    );
                    enter = field.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                    if ui.add(egui::Button::new("НАЙТИ").frame(false)).clicked() {
                        changed = true;
                    }
                });
            });
        ui.add_space(4.);
        ui.horizontal(|ui| {
            let (v, _) = ui.allocate_exact_size(Vec2::new(153., 40.), Sense::hover());
            if outline(
                ui,
                v,
                "catalog-version",
                &format!("Minecraft {}", self.settings.version),
                false,
                !self.busy(),
                accent,
            )
            .clicked()
            {
                self.open_versions();
            }
            let (c, _) = ui.allocate_exact_size(Vec2::new(145., 40.), Sense::hover());
            self.core_picker(ui, c, false);
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                secondary(
                    ui,
                    format!(
                        "{} проектов · по загрузкам",
                        self.catalog["total_hits"].as_u64().unwrap_or(0)
                    ),
                );
            });
        });
        if changed || enter {
            self.offset = 0;
            self.search();
        }
        if self.pending.contains("search") {
            ui.spinner();
        }
        ui.add_space(8.);
        for project in array(&self.catalog, "hits") {
            let id = text(&project, "project_id");
            let installed = self.data.modpacks.iter().any(|p| p.project_id == id)
                || self.data.installed.iter().any(|p| {
                    p.project_id == id
                        && p.instance_id == self.settings.instance_id
                        && p.game_version == self.settings.version
                        && p.loader == self.settings.loader
                });
            let (r, _) =
                ui.allocate_exact_size(Vec2::new(ui.available_width(), 112.), Sense::hover());
            let image_rect = Rect::from_min_size(r.min + Vec2::new(0., 33.), Vec2::splat(44.));
            ui.painter()
                .rect_filled(image_rect, 9., accent.gamma_multiply(0.08));
            let icon = text(&project, "icon_url");
            if icon.starts_with("https://") {
                ui.place(
                    image_rect,
                    egui::Image::new(icon)
                        .fit_to_exact_size(Vec2::splat(44.))
                        .corner_radius(9),
                );
            }
            let info =
                Rect::from_min_max(r.min + Vec2::new(62., 16.), r.max - Vec2::new(135., 10.));
            let mut body = ui.new_child(UiBuilder::new().id_salt(&id).max_rect(info));
            body.set_clip_rect(info.intersect(ui.clip_rect()));
            body.add(
                egui::Label::new(
                    RichText::new(text(&project, "title"))
                        .size(16.)
                        .family(egui::FontFamily::Name("InterSemiBold".into())),
                )
                .truncate(),
            );
            body.add(
                egui::Label::new(
                    RichText::new(text(&project, "description"))
                        .size(13.)
                        .color(MUTED),
                )
                .truncate(),
            );
            body.label(
                RichText::new(format!(
                    "{} · {} загрузок · {}",
                    text(&project, "author"),
                    downloads(project["downloads"].as_u64().unwrap_or(0)),
                    self.settings.version
                ))
                .size(12.)
                .color(MUTED),
            );
            let button = Rect::from_center_size(
                r.right_center() - Vec2::new(56.5, 0.),
                Vec2::new(113., 36.),
            );
            if outline(
                ui,
                button,
                (&id, "install"),
                if installed {
                    "Установлено"
                } else {
                    "Установить"
                },
                false,
                !self.busy() && !installed,
                accent,
            )
            .clicked()
            {
                self.submit(
                    "install",
                    Job::Install(id, text(&project, "title"), self.kind.clone()),
                );
            }
            line(ui, r.bottom());
        }
        ui.add_space(16.);
        ui.horizontal(|ui| {
            if ui
                .add_enabled(
                    self.offset > 0 && !self.pending.contains("search"),
                    egui::Button::new("← Назад"),
                )
                .clicked()
            {
                self.offset = self.offset.saturating_sub(12);
                self.search();
            }
            secondary(
                ui,
                format!(
                    "{} / {}",
                    self.offset / 12 + 1,
                    self.catalog["total_hits"]
                        .as_u64()
                        .unwrap_or(0)
                        .div_ceil(12)
                        .max(1)
                ),
            );
            if ui
                .add_enabled(
                    self.offset as u64 + 12 < self.catalog["total_hits"].as_u64().unwrap_or(0)
                        && !self.pending.contains("search"),
                    egui::Button::new("Далее →"),
                )
                .clicked()
            {
                self.offset += 12;
                self.search();
            }
        });
    }
    pub(super) fn activity(&mut self, ui: &mut Ui) {
        title(ui, "Активность", "Твоя история Minecraft");
        let accent = color(&self.settings.accent);
        let year_rect = Rect::from_min_size(
            ui.max_rect().right_top() + Vec2::new(-110., 13.),
            Vec2::new(110., 38.),
        );
        let year_response = outline(
            ui,
            year_rect,
            "activity-year",
            &self.activity_year.to_string(),
            false,
            true,
            accent,
        );
        egui::Popup::from_toggle_button_response(&year_response)
            .width(110.)
            .frame(
                Frame::new()
                    .fill(MODAL)
                    .stroke(Stroke::new(1.0_f32, BORDER))
                    .corner_radius(10)
                    .inner_margin(8),
            )
            .show(|ui| {
                let mut years = self
                    .data
                    .sessions
                    .iter()
                    .filter_map(|s| {
                        chrono::DateTime::parse_from_rfc3339(&s.started)
                            .ok()
                            .map(|d| d.with_timezone(&Local).year())
                    })
                    .collect::<Vec<_>>();
                years.push(Local::now().year());
                years.sort_unstable();
                years.dedup();
                for year in years.into_iter().rev() {
                    if ui.button(year.to_string()).clicked() {
                        self.activity_year = year;
                        ui.close();
                    }
                }
            });
        let today = Local::now().date_naive();
        let sessions = self.data.sessions.clone();
        let mut dates = std::collections::BTreeMap::<chrono::NaiveDate, (u64, u32)>::new();
        for s in &sessions {
            if let Ok(d) = chrono::DateTime::parse_from_rfc3339(&s.started) {
                let day = d.with_timezone(&Local).date_naive();
                let entry = dates.entry(day).or_default();
                entry.0 += s.seconds;
                entry.1 += 1;
            }
        }
        let total = sessions.iter().map(|s| s.seconds).sum();
        let week = dates
            .iter()
            .filter(|(d, _)| **d >= today - chrono::Duration::days(6))
            .map(|(_, v)| v.0)
            .sum();
        let record = sessions.iter().map(|s| s.seconds).max().unwrap_or(0);
        let mut streak = 0;
        let mut d = if dates.contains_key(&today) {
            today
        } else {
            today - chrono::Duration::days(1)
        };
        while dates.contains_key(&d) {
            streak += 1;
            d -= chrono::Duration::days(1);
        }
        let (stats, _) =
            ui.allocate_exact_size(Vec2::new(ui.available_width(), 110.), Sense::hover());
        line(ui, stats.top());
        line(ui, stats.bottom());
        let stats_values = [
            ("ВСЕГО В ИГРЕ", duration(total)),
            ("ЭТА НЕДЕЛЯ", duration(week)),
            ("СЕРИЯ", format!("{streak} дн.")),
            ("РЕКОРД СЕССИИ", duration(record)),
        ];
        for (i, (name, value)) in stats_values.iter().enumerate() {
            let x = stats.left() + i as f32 * stats.width() / 4.;
            if i > 0 {
                ui.painter().vline(
                    x - 12.,
                    (stats.top() + 24.0)..=(stats.bottom() - 24.0),
                    Stroke::new(1.0_f32, BORDER),
                );
            }
            caps(ui, Pos2::new(x, stats.top() + 23.), name, 11., MUTED, 0.6);
            label_at(
                ui,
                Pos2::new(x, stats.top() + 54.),
                value,
                if stats.width() < 740. { 21. } else { 25. },
                ink(ui),
            );
        }
        ui.add_space(20.);
        let jan = chrono::NaiveDate::from_ymd_opt(self.activity_year, 1, 1).unwrap();
        let start = jan - chrono::Duration::days(jan.weekday().num_days_from_monday() as i64);
        let cell = (ui.available_width() - 28.) / 53.;
        let (grid, _) = ui.allocate_exact_size(
            Vec2::new(ui.available_width(), cell * 7. + 65.),
            Sense::hover(),
        );
        let left = grid.left() + 28.;
        for month in 1..=12 {
            let date = chrono::NaiveDate::from_ymd_opt(self.activity_year, month, 1).unwrap();
            let week = (date - start).num_days() / 7;
            label_at(
                ui,
                Pos2::new(left + week as f32 * cell, grid.top() + 4.),
                [
                    "янв", "фев", "мар", "апр", "май", "июн", "июл", "авг", "сен", "окт", "ноя",
                    "дек",
                ][month as usize - 1],
                11.,
                MUTED,
            );
        }
        for (name, row) in [("Пн", 0), ("Ср", 2), ("Пт", 4)] {
            label_at(
                ui,
                Pos2::new(grid.left(), grid.top() + 27. + row as f32 * cell),
                name,
                10.,
                MUTED,
            );
        }
        for w in 0..53 {
            for day in 0..7 {
                let date = start + chrono::Duration::days(w * 7 + day);
                let (seconds, count) = dates.get(&date).copied().unwrap_or_default();
                let level = if seconds == 0 {
                    0.11
                } else if seconds < 1800 {
                    0.25
                } else if seconds < 3600 {
                    0.5
                } else if seconds < 7200 {
                    0.8
                } else {
                    1.
                };
                let outside = date.year() != self.activity_year || date > today;
                let c = accent.gamma_multiply(if outside { 0.04 } else { level });
                let square = Rect::from_min_size(
                    Pos2::new(left + w as f32 * cell, grid.top() + 25. + day as f32 * cell),
                    Vec2::splat(cell - 3.),
                );
                ui.painter().rect_filled(square, 2., c);
                ui.interact(square, ui.id().with((w, day)), Sense::hover())
                    .on_hover_text(format!(
                        "{} · {} · сессий: {count}",
                        date.format("%d.%m.%Y"),
                        duration(seconds)
                    ));
            }
        }
        label_at(
            ui,
            Pos2::new(grid.left(), grid.bottom() - 20.),
            "Один квадрат — один день. Наведи курсор для подробностей.",
            12.,
            MUTED,
        );
        let legend_x = grid.right() - 170.;
        label_at(
            ui,
            Pos2::new(legend_x, grid.bottom() - 20.),
            "Меньше",
            11.,
            MUTED,
        );
        for (index, level) in [0.11, 0.25, 0.5, 0.8, 1.0].into_iter().enumerate() {
            ui.painter().rect_filled(
                Rect::from_min_size(
                    Pos2::new(legend_x + 48. + index as f32 * 14., grid.bottom() - 17.),
                    Vec2::splat(10.),
                ),
                2.,
                accent.gamma_multiply(level),
            );
        }
        label_at(
            ui,
            Pos2::new(legend_x + 122., grid.bottom() - 20.),
            "Больше",
            11.,
            MUTED,
        );
        line(ui, grid.bottom() + 8.);
        ui.add_space(20.);
        ui.set_width(stats.width());
        ui.columns(2, |cols| {
            cols[0].label(
                RichText::new("ЛЮБИМЫЕ ВЕРСИИ")
                    .size(12.)
                    .family(egui::FontFamily::Name("InterSemiBold".into())),
            );
            let mut versions = std::collections::BTreeMap::<String, u64>::new();
            for s in &sessions {
                *versions
                    .entry(format!("{} / {}", s.version, s.loader))
                    .or_default() += s.seconds;
            }
            let mut versions = versions.into_iter().collect::<Vec<_>>();
            versions.sort_by_key(|s| std::cmp::Reverse(s.1));
            for (v, time) in versions.into_iter().take(5) {
                cols[0].add_space(10.);
                secondary(&mut cols[0], format!("{v} · {}", duration(time)));
                cols[0].add(
                    egui::ProgressBar::new(time as f32 / total.max(1) as f32).desired_height(6.),
                );
            }
            cols[1].label(
                RichText::new("ПОСЛЕДНИЕ СЕССИИ")
                    .size(12.)
                    .family(egui::FontFamily::Name("InterSemiBold".into())),
            );
            let mut recent = sessions.clone();
            recent.sort_by_key(|s| std::cmp::Reverse(s.started.clone()));
            for s in recent.iter().take(8) {
                cols[1].add_space(9.);
                cols[1].horizontal(|ui| {
                    ui.label(RichText::new(local_time(&s.started)).size(13.));
                    ui.label(
                        RichText::new(format!("{} / {}", s.version, s.loader))
                            .size(12.)
                            .color(accent),
                    );
                    secondary(ui, duration(s.seconds));
                });
                cols[1].separator();
            }
        });
    }
    pub(super) fn notifications(&mut self, ui: &mut Ui) {
        title(ui, "Уведомления", "Загрузки, обновления и события лаунчера");
        ui.horizontal(|ui| {
            if ui.button("Прочитать все").clicked() {
                let result = crate::notices(self.app.state(), "read".into(), None);
                self.result(result, "Уведомления прочитаны");
            }
            if ui.button("Очистить").clicked() {
                let result = crate::notices(self.app.state(), "clear".into(), None);
                self.result(result, "Уведомления очищены");
            }
        });
        ui.add_space(18.);
        let accent = color(&self.settings.accent);
        let mut last = String::new();
        for notice in self.data.notices.clone() {
            let day = local_time(&notice.time).chars().take(5).collect::<String>();
            if last != day {
                ui.add_space(12.);
                ui.label(RichText::new(&day).size(12.).color(MUTED));
                last = day;
            }
            Frame::new()
                .inner_margin(Margin::symmetric(0, 20))
                .show(ui, |ui| {
                    ui.horizontal_top(|ui| {
                        let (icon, _) = ui.allocate_exact_size(Vec2::new(26., 32.), Sense::hover());
                        draw_icon(ui.painter(), icon.center(), 5, accent);
                        ui.vertical(|ui| {
                            ui.set_width((ui.available_width() - 85.).max(100.));
                            ui.label(
                                RichText::new(&notice.title)
                                    .size(14.)
                                    .family(egui::FontFamily::Name("InterSemiBold".into())),
                            );
                            ui.label(RichText::new(&notice.body).size(13.).color(MUTED));
                        });
                        ui.vertical(|ui| {
                            secondary(ui, local_time(&notice.time).get(6..).unwrap_or(""));
                            if ui.add(egui::Button::new("×").frame(false)).clicked() {
                                let result = crate::notices(
                                    self.app.state(),
                                    "dismiss".into(),
                                    Some(notice.id.clone()),
                                );
                                self.result(result, "Удалено");
                            }
                        });
                    });
                });
            ui.separator();
        }
        if self.data.notices.is_empty() {
            secondary(ui, "Новых событий пока нет.");
        }
    }
}
impl Launcher {
    pub(super) fn settings(&mut self, ui: &mut Ui) {
        title(ui, "Настройки", "");
        let accent = color(&self.settings.accent);
        let before = serde_json::to_value(&self.settings).unwrap();
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 25.;
            for (i, name) in ["Основные", "Java и память", "Загрузки", "Профили графики"]
                .iter()
                .enumerate()
            {
                if tab(ui, i, *name, self.settings_tab == i, accent) {
                    self.settings_tab = i;
                    self.reset_focus();
                }
            }
        });
        ui.separator();
        ui.add_enabled_ui(!self.busy(),|ui|{match self.settings_tab{
            0=>{
                section(ui,"ЯЗЫК","Язык интерфейса",|ui|{secondary(ui,"Русский. Английская локализация появится в следующей версии лаунчера.");});
                section(ui,"ОФОРМЛЕНИЕ","Твой стиль. Ничего лишнего.",|ui|{
                    ui.horizontal_wrapped(|ui|{ui.label("Тема");for (id,name) in [("dark","Тёмная"),("midnight","Полночь"),("black","Чёрная"),("light","Светлая")]{ui.selectable_value(&mut self.settings.theme,id.into(),name);}});ui.add_space(8.);
                    ui.horizontal_wrapped(|ui|{ui.label("Палитра");for hex in ["#a185cf","#8752ff","#659aff","#4bc6c5","#80c99a","#eabd68","#e597bb","#eb838b","#d8dbe5"]{let c=color(hex);let (r,_)=ui.allocate_exact_size(Vec2::splat(36.),Sense::hover());let selected=self.settings.accent.eq_ignore_ascii_case(hex);let response=hit(ui,r,hex,hex,true);if selected{ui.painter().circle_stroke(r.center(),17.5,Stroke::new(1.0_f32,c));}ui.painter().circle_filled(r.center(),11.,c);if selected{ui.painter().text(r.center(),Align2::CENTER_CENTER,"✓",FontId::proportional(13.),BG);}if response.clicked(){self.settings.accent=hex.into();}}});
                });
                section(ui,"ПРИ ЗАПУСКЕ ИГРЫ","После закрытия Minecraft лаунчер откроется автоматически",|ui|{ui.horizontal_wrapped(|ui|{for (id,label) in [("hide","Скрыть"),("minimize","Свернуть"),("keep","Оставить открытым")]{ui.selectable_value(&mut self.settings.launch_behavior,id.into(),label);}});});
                section(ui,"ГЛАВНЫЙ ЭКРАН","Оформление твоего Minecraft",|ui|{
                    ui.horizontal(|ui|{ui.selectable_value(&mut self.settings.home_layout,"orbit".into(),"Орбита");ui.selectable_value(&mut self.settings.home_layout,"card".into(),"Карточка");});
                    ui.horizontal(|ui|{if ui.button("Свой фон…").clicked(){if let Some(image)=pick_image(){self.settings.home_background=image;}}if ui.button("Сбросить фон").clicked(){self.settings.home_background.clear();}});
                });
                section(ui,"УВЕДОМЛЕНИЯ","Загрузки и обновления",|ui|{ui.checkbox(&mut self.settings.mod_notifications,"Обновления модов");ui.checkbox(&mut self.settings.system_notifications,"События лаунчера");});
                section(ui,"DISCORD RICH PRESENCE","Активность KYRO в твоём профиле Discord",|ui|{
                    ui.checkbox(&mut self.settings.discord_enabled,"Показывать активность в Discord");
                    ui.checkbox(&mut self.settings.discord_show_server,"Показывать адрес сервера");
                    secondary(ui,self.app.presence.status());
                    secondary(ui,"Работает автоматически с настольным Discord. Иконка KYRO подставляется автоматически; таймер идёт с открытия лаунчера.");
                    secondary(ui,"Адрес сервера определяется по сообщениям Minecraft в журнале игры.");
                });
                section(ui,"ВЕРСИЯ ЛАУНЧЕРА","Обновления KYRO",|ui|{secondary(ui,format!("KYRO {} / Rust + egui",env!("CARGO_PKG_VERSION")));if ui.button("Проверить обновления").clicked(){self.submit("check-update",Job::CheckUpdate);}if !self.update.is_null(){secondary(ui,if self.update["available"]==true{format!("Доступна {}",text(&self.update,"version"))}else{"Установлена актуальная версия".into()});if self.update["installable"]==true&&ui.button("Установить и перезапустить").clicked(){self.submit("update-launcher",Job::UpdateLauncher);}}});
            },
            1=>{
                ui.add_space(30.);ui.columns(2,|cols|{
                    cols[0].label(RichText::new("ПАМЯТЬ").size(12.).family(egui::FontFamily::Name("InterSemiBold".into())));cols[0].add_space(10.);secondary(&mut cols[0],format!("{} ГБ выделено для игры.",self.settings.memory_gb));cols[0].add_space(12.);
                    cols[0].horizontal_wrapped(|ui|{for gb in [2,4,6,8,12]{ui.selectable_value(&mut self.settings.memory_gb,gb,format!("{gb} ГБ"));}});self.settings.memory_gb=slider(&mut cols[0],self.settings.memory_gb,2,32,accent);secondary(&mut cols[0],"Выделяй не больше половины доступной RAM.");
                    cols[1].label(RichText::new("АРГУМЕНТЫ JVM").size(12.).family(egui::FontFamily::Name("InterSemiBold".into())));cols[1].add_space(10.);secondary(&mut cols[1],"Передаются Java напрямую, без командной оболочки.");cols[1].add_space(12.);cols[1].add(egui::TextEdit::multiline(&mut self.settings.jvm_args).font(egui::TextStyle::Monospace).margin(Margin::same(13)).desired_width(f32::INFINITY).desired_rows(4));if cols[1].add_sized([cols[1].available_width(),36.],egui::Button::new("Сбросить к рекомендуемым")).clicked(){self.settings.jvm_args=Settings::default().jvm_args;}
                });ui.add_space(22.);ui.separator();
                section(ui,"JAVA","Версия среды зависит от Minecraft",|ui|{ui.checkbox(&mut self.settings.auto_java,"Подбирать и загружать Java автоматически");secondary(ui,&self.settings.java_path);if !self.settings.auto_java{ui.add(egui::TextEdit::singleline(&mut self.settings.java_path).margin(Margin::symmetric(13,11)).desired_width(f32::INFINITY));}if ui.button("Подготовить Java для этой версии").clicked(){if self.save(){self.submit("java",Job::Java);}}});
            },
            2=>{
                section(ui,"ЗАГРУЗКА","Скорость и целостность файлов",|ui|{ui.label(format!("Потоков загрузки: {}",self.settings.download_threads));self.settings.download_threads=slider(ui,self.settings.download_threads as u32,1,16,accent) as usize;if ui.button("Перепроверить файлы").clicked(){if self.save(){self.submit("prepare",Job::Prepare);}}});
                section(ui,"ПАПКИ","Лаунчер, данные и игровой профиль",|ui|{for (target,name) in [("launcher","Папка лаунчера"),("instance","Папка профиля"),("data","Данные и кэш"),("log","Лог запуска")]{if ui.button(name).clicked(){let result=crate::open_folder(self.app.state(),target.into()).map(|_|());self.result(result,"Открыто");}if let Ok(path)=self.app.state().target_path(target,&self.settings){ui.label(RichText::new(path.to_string_lossy()).size(12.).color(MUTED));}ui.add_space(5.);}});
            },
            _=>{
                ui.add_space(25.);secondary(ui,"Пресеты меняют только графику выбранного профиля. Перед изменением сохраняется резервная копия options.txt.");ui.add_space(12.);
                ui.columns(3,|cols|{for (col,(id,name,desc,values)) in cols.iter_mut().zip([("fps","Максимум FPS","Для плавной игры и высокого FPS.","8 чанков · минимум частиц\nVSync выключен · без лимита FPS"),("beautiful","Красиво","Дальность, освещение и эффекты.","16 чанков · красивая графика\nОблака и частицы · VSync"),("low","Для слабого ПК","Меньше нагрузки на процессор и GPU.","4 чанка · простая графика\nМинимум эффектов · до 60 FPS")]){Frame::new().fill(PANEL).stroke(Stroke::new(1.0_f32,BORDER)).corner_radius(14).inner_margin(18).show(col,|ui|{ui.set_min_width(ui.available_width());ui.label(RichText::new(name).size(18.).family(egui::FontFamily::Name("InterSemiBold".into())));secondary(ui,desc);ui.add_space(12.);ui.label(RichText::new(values).size(13.));ui.add_space(15.);if ui.add_sized([ui.available_width(),36.],egui::Button::new("Применить")).clicked(){self.submit("preset",Job::Preset(id.into()));}});}});ui.add_space(18.);if ui.button("Восстановить предыдущие настройки").clicked(){self.submit("restore-preset",Job::RestorePreset);}
            }
        }});
        ui.add_space(18.);
        secondary(ui, "Изменения сохраняются автоматически");
        if serde_json::to_value(&self.settings).unwrap() != before {
            self.dirty = Some(Instant::now());
        }
    }
    pub(super) fn dialogs(&mut self, ctx: &egui::Context) {
        let accent = color(&self.settings.accent);
        if self.fps_open {
            modal(ctx, "fps-boost-dialog", 24).show(ctx, |ui| {
                ui.set_width((ctx.content_rect().width() - 96.).min(480.));
                if modal_heading(ui, "ОПТИМИЗАЦИЯ", "FPS-буст", accent) && !self.busy() { self.fps_open = false; }
                ui.add_space(12.);
                ui.label(RichText::new(format!("{} · {}", self.settings.version, self.settings.loader)).color(accent));
                ui.label("Моды оптимизации с Modrinth. Настройки графики не меняются.");
                ui.add_space(12.);
                if self.pending.contains("fps-plan") {
                    ui.horizontal(|ui| { ui.spinner(); ui.label("Проверяем совместимость и зависимости…"); });
                } else if self.fps_plan["scope"].as_str() == Some(super::scope(&self.settings).as_str()) {
                    egui::ScrollArea::vertical().max_height((ctx.content_rect().height() - 410.).clamp(100., 270.)).show(ui, |ui| {
                        for row in array(&self.fps_plan, "rows") {
                            ui.label(RichText::new(text(&row, "name")).family(egui::FontFamily::Name("InterSemiBold".into())).size(15.));
                            ui.label(RichText::new(text(&row, "description")).size(13.).color(MUTED));
                            ui.label(RichText::new(text(&row, "status")).size(12.).color(accent));
                            ui.add_space(10.);
                        }
                    });
                    let count = self.fps_plan["items"].as_array().map_or(0, Vec::len);
                    ui.add_space(10.);
                    if self.fps_plan["unknown"].as_u64().unwrap_or(0) > 0 {
                        ui.label(RichText::new("Есть моды, не опознанные Modrinth. Их совместимость нужно проверить отдельно.").size(12.).color(MUTED));
                    }
                    ui.label(format!("К скачиванию: {count} файлов с зависимостями"));
                    if ui.add_enabled(count > 0 && !self.busy(), egui::Button::new("Установить FPS-буст").min_size(Vec2::new(220., 38.))).clicked() {
                        self.submit("fps-install", Job::FpsInstall(self.fps_plan.clone()));
                    }
                } else if ui.button("Повторить подбор").clicked() && !self.busy() && self.save() {
                    self.submit("fps-plan", Job::FpsPlan);
                }
                if self.pending.contains("fps-install") { ui.spinner(); ui.label(&self.progress.0); }
            });
        }
        if self.versions_open {
            let response = modal(ctx, "versions-dialog", 28).show(ctx, |ui| {
                ui.set_width((ctx.content_rect().width() - 96.).min(524.));
                ui.spacing_mut().item_spacing.y = 0.;
                if modal_heading(ui, "ТВОЙ MINECRAFT", "Версии и ядра", accent) {
                    self.versions_open = false;
                }
                ui.add_space(16.);
                let width = ui.available_width();
                let (tools, _) = ui.allocate_exact_size(Vec2::new(width, 44.), Sense::hover());
                let search = Rect::from_min_size(tools.min, Vec2::new(width - 160., 44.));
                ui.painter()
                    .rect_filled(search, 10., Color32::from_rgb(32, 25, 43));
                let mut field = ui.new_child(
                    UiBuilder::new()
                        .id_salt("version-search")
                        .max_rect(search.shrink2(Vec2::new(13., 10.))),
                );
                field.add(
                    egui::TextEdit::singleline(&mut self.version_query)
                        .frame(false)
                        .margin(0)
                        .font(FontId::proportional(13.))
                        .hint_text("Поиск версии или ядра")
                        .desired_width(search.width() - 26.),
                );
                let downloaded = Rect::from_min_size(
                    search.right_top() + Vec2::new(9., 0.),
                    Vec2::new(102., 44.),
                );
                if outline(
                    ui,
                    downloaded,
                    "downloaded",
                    "↓ Скачанные",
                    self.downloaded,
                    true,
                    accent,
                )
                .clicked()
                {
                    self.downloaded = !self.downloaded;
                }
                let plus = Rect::from_min_size(
                    downloaded.right_top() + Vec2::new(9., 0.),
                    Vec2::splat(40.),
                );
                if outline(ui, plus, "prepare", "+", false, !self.busy(), accent).clicked() {
                    self.submit("prepare", Job::Prepare);
                    self.versions_open = false;
                }
                ui.add_space(16.);
                let warning = text(&self.library, "warning");
                if !warning.is_empty() {
                    secondary(ui, &warning);
                    if ui.button("Повторить загрузку списка").clicked() {
                        self.submit("versions", Job::Versions);
                    }
                }
                let releases = array(&self.library, "releases");
                let prepared = array(&self.library, "prepared");
                let q = self.version_query.trim().to_lowercase();
                egui::ScrollArea::vertical()
                    .id_salt("version-list")
                    .max_height(
                        (ctx.content_rect().height()
                            - 288.
                            - if warning.is_empty() { 0. } else { 65. })
                        .max(160.),
                    )
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        for release in &releases {
                            let version = text(release, "id");
                            let version_matches = version.to_lowercase().contains(&q);
                            if !version_matches
                                && !crate::cores::LOADERS
                                    .iter()
                                    .any(|c| c.to_lowercase().contains(&q))
                            {
                                continue;
                            }
                            if self.downloaded && !prepared.iter().any(|p| p["version"] == version)
                            {
                                continue;
                            }
                            let (group, _) = ui.allocate_exact_size(
                                Vec2::new(ui.available_width(), 42.),
                                Sense::hover(),
                            );
                            let expanded = self.expanded_versions.contains(&version)
                                || (!q.is_empty() && version_matches);
                            let gr = hit(ui, group, (&version, "group"), &version, true);
                            chevron(
                                ui,
                                group.left_center() + Vec2::new(10., 0.),
                                MUTED,
                                expanded,
                            );
                            label_at(ui, group.min + Vec2::new(28., 13.), &version, 14., TEXT);
                            label_at(
                                ui,
                                group.min + Vec2::new(28. + version.len() as f32 * 8. + 12., 14.),
                                "5 ядер",
                                12.,
                                MUTED,
                            );
                            if gr.clicked() {
                                if !self.expanded_versions.insert(version.clone()) {
                                    self.expanded_versions.remove(&version);
                                }
                            }
                            if expanded {
                                for core in crate::cores::LOADERS {
                                    if !version_matches && !core.to_lowercase().contains(&q) {
                                        continue;
                                    }
                                    let installed = prepared
                                        .iter()
                                        .any(|p| p["version"] == version && p["loader"] == core);
                                    if self.downloaded && !installed {
                                        continue;
                                    }
                                    let (row, _) = ui.allocate_exact_size(
                                        Vec2::new(ui.available_width(), 54.),
                                        Sense::hover(),
                                    );
                                    let row =
                                        Rect::from_min_max(row.min + Vec2::new(19., 0.), row.max);
                                    let selected = self.settings.version == version
                                        && self.settings.loader == core
                                        && self.settings.instance_id.is_empty();
                                    let response = hit(
                                        ui,
                                        row,
                                        (&version, core),
                                        &format!("{version} {core}"),
                                        !self.busy(),
                                    );
                                    if selected || response.hovered() {
                                        ui.painter().rect(
                                            row,
                                            8.,
                                            accent.gamma_multiply(0.11),
                                            Stroke::new(
                                                1.0_f32,
                                                if selected {
                                                    accent.gamma_multiply(0.55)
                                                } else {
                                                    Color32::TRANSPARENT
                                                },
                                            ),
                                            egui::StrokeKind::Inside,
                                        );
                                    }
                                    label_at(
                                        ui,
                                        row.min + Vec2::new(13., 17.),
                                        &version,
                                        15.,
                                        TEXT,
                                    );
                                    let tag = Rect::from_min_size(
                                        row.min + Vec2::new(107., 14.),
                                        Vec2::new(core.len() as f32 * 7.2 + 22., 27.),
                                    );
                                    ui.painter()
                                        .rect_filled(tag, 7., accent.gamma_multiply(0.1));
                                    label_at(ui, tag.min + Vec2::new(11., 6.), core, 13., accent);
                                    let date = text(release, "date");
                                    let date = chrono::DateTime::parse_from_rfc3339(&date)
                                        .ok()
                                        .map(|d| {
                                            format!(
                                                "{} {}",
                                                d.day(),
                                                [
                                                    "янв", "фев", "мар", "апр", "май", "июн",
                                                    "июл", "авг", "сент", "окт", "ноя", "дек"
                                                ]
                                                    [d.month0() as usize]
                                            )
                                        })
                                        .unwrap_or_default();
                                    ui.painter().text(
                                        row.right_center() - Vec2::new(12., 0.),
                                        Align2::RIGHT_CENTER,
                                        if installed {
                                            format!("↓  {date}")
                                        } else {
                                            date
                                        },
                                        FontId::proportional(12.),
                                        MUTED,
                                    );
                                    if response.clicked() {
                                        self.settings.version = version.clone();
                                        self.settings.loader = core.into();
                                        self.settings.instance_id.clear();
                                        self.settings.loader_version.clear();
                                        if self.save() {
                                            self.profile_changed();
                                            self.versions_open = false;
                                        }
                                    }
                                }
                            }
                        }
                        if self.pending.contains("versions") {
                            ui.spinner();
                        }
                    });
                ui.add_space(14.);
                ui.separator();
                let (foot, _) =
                    ui.allocate_exact_size(Vec2::new(ui.available_width(), 30.), Sense::hover());
                label_at(
                    ui,
                    foot.min + Vec2::new(0., 11.),
                    format!(
                        "Версий: {} · Вариантов: {}",
                        releases.len(),
                        releases.len() * 5
                    ),
                    12.,
                    MUTED,
                );
                ui.painter().text(
                    foot.right_top() + Vec2::new(0., 18.),
                    Align2::RIGHT_CENTER,
                    format!("{} / {}", self.settings.version, self.settings.loader),
                    FontId::proportional(12.),
                    MUTED,
                );
            });
            if response.should_close() {
                self.versions_open = false;
            }
        }
        if self.account_open {
            let response=modal(ctx,"account-dialog",38).show(ctx,|ui|{
                ui.set_width((ctx.content_rect().width()-130.).min(454.));egui::ScrollArea::vertical().id_salt("account-body").auto_shrink([false,false]).min_scrolled_height((ctx.content_rect().height()-136.).max(280.)).max_height((ctx.content_rect().height()-136.).max(280.)).show(ui,|ui|{
                    if modal_heading(ui,"УЧЁТНЫЕ ЗАПИСИ","Твой Minecraft.",accent){self.account_open=false;}ui.add_space(12.);secondary(ui,"Добавь офлайн-аккаунт по нику или войди через Microsoft.");ui.add_space(20.);
                    for account in self.data.offline_accounts.clone(){let id=format!("offline:{}",account.uuid);let (r,_)=ui.allocate_exact_size(Vec2::new(ui.available_width(),68.),Sense::hover());let avatar=Rect::from_min_size(r.min+Vec2::new(8.,10.),Vec2::splat(34.));ui.painter().rect(avatar,9.,accent.gamma_multiply(0.1),Stroke::new(1.0_f32,accent.gamma_multiply(0.4)),egui::StrokeKind::Inside);ui.painter().text(avatar.center(),Align2::CENTER_CENTER,account.name.chars().next().unwrap_or('K'),FontId::proportional(18.),accent);label_at(ui,r.min+Vec2::new(56.,11.),&account.name,14.,TEXT);label_at(ui,r.min+Vec2::new(56.,32.),"Офлайн-аккаунт",11.,MUTED);let select=Rect::from_min_max(r.min,r.right_bottom()-Vec2::new(38.,0.));if hit(ui,select,(&id,"select"),&account.name,!self.busy()).clicked(){let result=crate::offline_account(self.app.state(),"select".into(),id.clone());self.result(result,"Аккаунт выбран");}if self.data.selected_account.as_deref()==Some(&id){label_at(ui,r.right_center()+Vec2::new(-68.,-8.),"✓",16.,accent);}let remove=Rect::from_center_size(r.right_center()-Vec2::new(15.,0.),Vec2::splat(28.));label_at(ui,remove.min+Vec2::new(8.,4.),"×",15.,TEXT);if hit(ui,remove,(&id,"remove"),"Удалить аккаунт",!self.busy()).clicked(){let result=crate::offline_account(self.app.state(),"remove".into(),id);self.result(result,"Аккаунт удалён");}line(ui,r.bottom());}
                    ui.add_space(24.);ui.label(RichText::new("Ник офлайн-аккаунта").size(13.));ui.add(egui::TextEdit::singleline(&mut self.nickname).hint_text("Например, Nova").char_limit(16).margin(Margin::symmetric(13,13)).desired_width(f32::INFINITY));ui.add_space(8.);let (r,_)=ui.allocate_exact_size(Vec2::new(215.,40.),Sense::hover());let response=hit(ui,r,"add-offline","Добавить офлайн-аккаунт",!self.busy());ui.painter().rect_filled(r,8.,accent);ui.painter().text(r.center(),Align2::CENTER_CENTER,"Добавить офлайн-аккаунт",FontId::proportional(13.),BG);if response.clicked(){let result=crate::offline_account(self.app.state(),"add".into(),self.nickname.clone());self.result(result,"Офлайн-аккаунт добавлен и выбран");self.nickname.clear();}ui.add_space(14.);secondary(ui,"От 3 до 16 латинских букв, цифр или _. Без пароля и Microsoft. Для серверов с проверкой лицензии нужен Microsoft-аккаунт.");ui.add_space(16.);ui.separator();
                    egui::CollapsingHeader::new(RichText::new("Вход через Microsoft").size(13.).color(MUTED)).show(ui,|ui|{
                        secondary(ui,"Для входа нужен OAuth Client ID приложения Microsoft.");ui.add(egui::TextEdit::singleline(&mut self.settings.microsoft_client_id).margin(Margin::symmetric(13,11)).hint_text("Microsoft OAuth Client ID").desired_width(f32::INFINITY));ui.horizontal_wrapped(|ui|{if ui.add_enabled(!self.busy(),egui::Button::new("Войти в Microsoft")).clicked(){if self.save(){self.submit("login-begin",Job::LoginBegin);}}if ui.add_enabled(!self.busy(),egui::Button::new("Выбрать Microsoft")).clicked(){let result=crate::offline_account(self.app.state(),"select".into(),"microsoft".into());self.result(result,"Аккаунт выбран");}if ui.add_enabled(!self.busy(),egui::Button::new("Выйти")).clicked(){let result=crate::logout();self.result(result,"Выполнен выход");}});
                    });
                    if !self.login.is_null(){ui.label("Введи код на странице Microsoft:");ui.label(RichText::new(text(&self.login,"user_code")).size(25.).family(egui::FontFamily::Name("InterSemiBold".into())));if ui.button("Открыть страницу входа").clicked(){let result=crate::open_microsoft_login(text(&self.login,"verification_uri"));self.result(result,"Страница открыта");}if ui.button("Отменить вход").clicked(){self.login=Value::Null;}}
                });
            });
            if response.should_close() {
                self.account_open = false;
            }
        }
        if self.confirm_launch {
            let response = modal(ctx, "compatibility-warning", 28).show(ctx, |ui| {
                ui.set_width(500.);
                if modal_heading(ui, "ПЕРЕД ЗАПУСКОМ", "Есть несовместимые моды", accent)
                {
                    self.confirm_launch = false;
                }
                ui.add_space(15.);
                ui.label("Обнаружены ошибки совместимости. Minecraft может завершиться с ошибкой.");
                ui.add_space(15.);
                ui.horizontal(|ui| {
                    if ui.button("Посмотреть ошибки").clicked() {
                        self.confirm_launch = false;
                        self.navigate(3);
                    }
                    if ui.button("Запустить всё равно").clicked() {
                        self.confirm_launch = false;
                        self.submit("launch", Job::Launch);
                    }
                    if ui.button("Отмена").clicked() {
                        self.confirm_launch = false;
                    }
                });
            });
            if response.should_close() {
                self.confirm_launch = false;
            }
        }
        if let Some(image) = self.image.clone() {
            let response = modal(ctx, "screenshot-view", 18).show(ctx, |ui| {
                ui.set_width((ctx.content_rect().width() - 80.).min(1100.));
                if modal_heading(ui, "ГАЛЕРЕЯ", "Скриншот", accent) {
                    self.image = None;
                }
                if let Some(uri) = include_data(ctx, &image, "screenshot") {
                    ui.add(
                        egui::Image::new(uri)
                            .max_size(Vec2::new(
                                ui.available_width(),
                                ctx.content_rect().height() - 150.,
                            ))
                            .corner_radius(9),
                    );
                }
            });
            if response.should_close() {
                self.image = None;
            }
        }
        if let Some(error) = self.error.clone() {
            let response = modal(ctx, "error-dialog", 28).show(ctx, |ui| {
                ui.set_width(440.);
                if modal_heading(ui, "KYRO", "Не удалось выполнить действие", accent)
                {
                    self.error = None;
                }
                ui.add_space(14.);
                ui.label(error);
                ui.add_space(18.);
                if ui.button("Понятно").clicked() {
                    self.error = None;
                }
            });
            if response.should_close() {
                self.error = None;
            }
        }
    }
}
