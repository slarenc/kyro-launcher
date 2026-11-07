use crate::{
    app_context::{AppContext, Event, WindowAction},
    store::{Data, Settings, State},
};
use base64::Engine;
use eframe::egui::{
    self, Align, Color32, CornerRadius, Frame, Layout, Margin, RichText, Stroke, Vec2,
};
use serde_json::{json, Value};
use std::{
    collections::{HashMap, HashSet},
    sync::{mpsc, Arc},
    time::{Duration, Instant},
};

const BG: Color32 = Color32::from_rgb(8, 7, 12);
const PANEL: Color32 = Color32::from_rgb(17, 16, 22);
const MUTED: Color32 = Color32::from_rgb(137, 132, 143);
const BORDER: Color32 = Color32::from_rgb(42, 38, 49);
const MENU: [&str; 7] = [
    "Играть",
    "Каталог",
    "Скриншоты",
    "Диагностика",
    "Активность",
    "Уведомления",
    "Настройки",
];

pub fn run() -> eframe::Result {
    let icon = image::load_from_memory(include_bytes!("../icons/icon.png"))
        .ok()
        .map(|i| {
            let i = i.into_rgba8();
            egui::IconData {
                width: i.width(),
                height: i.height(),
                rgba: i.into_raw(),
            }
        });
    let mut viewport = egui::ViewportBuilder::default()
        .with_inner_size([1060., 600.])
        .with_min_inner_size([960., 600.])
        .with_decorations(false)
        .with_title("KYRO · Native");
    if let Some(icon) = icon {
        viewport = viewport.with_icon(icon);
    }
    eframe::run_native(
        "KYRO",
        eframe::NativeOptions {
            viewport,
            renderer: eframe::Renderer::Glow,
            centered: true,
            ..Default::default()
        },
        Box::new(|cc| Ok(Box::new(Launcher::new(&cc.egui_ctx)))),
    )
}

#[derive(Clone)]
enum Job {
    Versions,
    FpsPlan,
    FpsInstall(Value),
    Search(String, String, Settings, u32),
    Scan,
    Launch,
    Prepare,
    Install(String, String, String),
    Toggle(String, bool),
    UpdateMod(String),
    Java,
    Diagnose,
    Compatibility,
    Screens,
    Preset(String),
    RestorePreset,
    CheckUpdate,
    UpdateLauncher,
    LoginBegin,
    LoginPoll(String),
    Image(String),
    Thumbnails(Vec<String>),
}
struct Reply {
    tag: String,
    scope: String,
    mod_epoch: u64,
    result: Result<Value, String>,
}
async fn execute(app: AppContext, job: Job) -> Result<Value, String> {
    let state = app.state();
    match job {
        Job::FpsPlan => crate::fps_boost::plan(state).await,
        Job::FpsInstall(plan) => crate::fps_boost::install(app, plan).await,
        Job::Versions => crate::version_library(state).await,
        Job::Search(q, k, s, o) => crate::search_modrinth(q, k, s.version, s.loader, o).await,
        Job::Scan => crate::managed_mods::scan_mods(state, true).await,
        Job::Launch => crate::launch_game(app).await.map(|_| json!(null)),
        Job::Prepare => crate::verify_files(app).await.map(|_| json!(null)),
        Job::Install(p, n, k) => crate::install_mod(app, p, n, k).await.map(|_| json!(null)),
        Job::Toggle(f, b) => crate::managed_mods::toggle_mod(state, f, b).map(|_| json!(null)),
        Job::UpdateMod(f) => crate::managed_mods::update_mod(app, f)
            .await
            .map(|_| json!(null)),
        Job::Java => crate::runtime::install_java(app).await.map(|s| json!(s)),
        Job::Diagnose => crate::diagnostics::diagnose_launch(state),
        Job::Compatibility => crate::compatibility::check_compatibility(state),
        Job::Screens => crate::playtools::list_screenshots(state),
        Job::Thumbnails(names) => {
            let mut previews = serde_json::Map::new();
            for name in names {
                if let Ok(data) = crate::playtools::screenshot_image(state, name.clone()) {
                    if let Some((_, encoded)) = data.split_once(',') {
                        if let Ok(bytes) = base64::engine::general_purpose::STANDARD.decode(encoded)
                        {
                            let mut reader = image::ImageReader::new(std::io::Cursor::new(bytes));
                            reader.set_format(image::ImageFormat::Png);
                            let mut limits = image::Limits::default();
                            limits.max_image_width = Some(16384);
                            limits.max_image_height = Some(16384);
                            limits.max_alloc = Some(128 * 1024 * 1024);
                            reader.limits(limits);
                            if let Ok(image) = reader.decode() {
                                let mut out = std::io::Cursor::new(Vec::new());
                                if image
                                    .thumbnail(480, 270)
                                    .write_to(&mut out, image::ImageFormat::Png)
                                    .is_ok()
                                {
                                    previews.insert(
                                        name,
                                        json!(format!(
                                            "data:image/png;base64,{}",
                                            base64::engine::general_purpose::STANDARD
                                                .encode(out.into_inner())
                                        )),
                                    );
                                }
                            }
                        }
                    }
                }
            }
            Ok(Value::Object(previews))
        }
        Job::Image(name) => crate::playtools::screenshot_image(state, name).map(|s| json!(s)),
        Job::Preset(id) => crate::playtools::apply_graphics_preset(state, id),
        Job::RestorePreset => crate::playtools::restore_graphics_preset(state).map(|_| json!(null)),
        Job::CheckUpdate => crate::updates::check_launcher_update().await,
        Job::UpdateLauncher => crate::updates::install_launcher_update(app)
            .await
            .map(|_| json!(null)),
        Job::LoginBegin => crate::login_begin(state).await,
        Job::LoginPoll(code) => crate::login_poll(state, code).await,
    }
}
fn scope(s: &Settings) -> String {
    format!("{}|{}|{}", s.version, s.loader, s.instance_id)
}
fn text(v: &Value, key: &str) -> String {
    v[key].as_str().unwrap_or("").to_owned()
}
fn array(v: &Value, key: &str) -> Vec<Value> {
    v[key].as_array().cloned().unwrap_or_default()
}
fn draw_icon(p: &egui::Painter, c: egui::Pos2, index: usize, color: Color32) {
    let stroke = Stroke::new(1.5_f32, color);
    let line = |points: &[(f32, f32)]| {
        p.add(egui::Shape::line(
            points.iter().map(|&(x, y)| c + Vec2::new(x, y)).collect(),
            stroke,
        ));
    };
    match index {
        0 => line(&[(-5., -7.), (7., 0.), (-5., 7.), (-5., -7.)]),
        1 => {
            line(&[(0., -7.), (8., -3.), (0., 1.), (-8., -3.), (0., -7.)]);
            line(&[(-8., 1.), (0., 5.), (8., 1.)]);
            line(&[(-8., 5.), (0., 9.), (8., 5.)]);
        }
        2 => {
            p.rect_stroke(
                egui::Rect::from_center_size(c, Vec2::new(17., 14.)),
                2,
                stroke,
                egui::StrokeKind::Inside,
            );
            p.circle_filled(c + Vec2::new(-4., -3.), 1.5, color);
            line(&[(-7., 5.), (-1., -1.), (3., 3.), (6., 0.), (8., 3.)]);
        }
        3 => line(&[
            (-9., 0.),
            (-5., 0.),
            (-2., -7.),
            (2., 7.),
            (5., -2.),
            (8., -2.),
        ]),
        4 => {
            for (x, h) in [(-6., 3.), (0., 8.), (6., 5.)] {
                p.line_segment(
                    [c + Vec2::new(x, 7.), c + Vec2::new(x, 7. - h * 2.)],
                    stroke,
                );
            }
        }
        5 => {
            line(&[
                (-8., 5.),
                (-5., 2.),
                (-5., -3.),
                (-3., -7.),
                (3., -7.),
                (5., -3.),
                (5., 2.),
                (8., 5.),
                (-8., 5.),
            ]);
            p.circle_filled(c + Vec2::new(0., 8.), 1.5, color);
        }
        _ => {
            p.circle_stroke(c, 5.5, stroke);
            p.circle_stroke(c, 2., stroke);
            for n in 0..8 {
                let a = n as f32 * std::f32::consts::TAU / 8.;
                let d = Vec2::angled(a);
                p.line_segment([c + d * 6., c + d * 9.], stroke);
            }
        }
    }
}
fn card() -> Frame {
    Frame::new()
        .fill(PANEL)
        .stroke(Stroke::new(1.0_f32, BORDER))
        .corner_radius(16)
        .inner_margin(18)
}
fn secondary(ui: &mut egui::Ui, label: impl Into<String>) {
    ui.label(RichText::new(label).color(MUTED).size(13.));
}
fn title(ui: &mut egui::Ui, label: &str, subtitle: &str) {
    ui.label(
        RichText::new(label)
            .size(36.)
            .family(egui::FontFamily::Name("InterSemiBold".into())),
    );
    if !subtitle.is_empty() {
        secondary(ui, subtitle);
        ui.add_space(18.);
    } else {
        ui.add_space(26.);
    }
}
fn local_time(s: &str) -> String {
    chrono::DateTime::parse_from_rfc3339(s)
        .map(|d| {
            d.with_timezone(&chrono::Local)
                .format("%d.%m %H:%M")
                .to_string()
        })
        .unwrap_or_else(|_| s.into())
}
fn duration(seconds: u64) -> String {
    format!("{} ч {} мин", seconds / 3600, seconds % 3600 / 60)
}
fn color(hex: &str) -> Color32 {
    Color32::from_hex(hex).unwrap_or(Color32::from_rgb(161, 133, 207))
}

struct Smoke {
    dir: std::path::PathBuf,
    step: usize,
    waiting: bool,
    since: Instant,
}
mod design;
struct Launcher {
    context: egui::Context,
    app: AppContext,
    runtime: tokio::runtime::Runtime,
    events: mpsc::Receiver<Event>,
    replies: mpsc::Receiver<Reply>,
    sender: mpsc::Sender<Reply>,
    pending: HashSet<String>,
    data: Data,
    settings: Settings,
    page: usize,
    settings_tab: usize,
    activity_year: i32,
    expanded_versions: HashSet<String>,
    error: Option<String>,
    toast: Option<(String, Instant)>,
    progress: (String, f32),
    account: Value,
    account_open: bool,
    nickname: String,
    versions_open: bool,
    fps_open: bool,
    fps_plan: Value,
    library: Value,
    version_query: String,
    downloaded: bool,
    catalog: Value,
    search_queued: bool,
    query: String,
    kind: String,
    catalog_tab: usize,
    offset: u32,
    mods: Value,
    mod_scope: String,
    mod_epoch: u64,
    mod_checked: Option<Instant>,
    mod_cache: HashMap<String, (Value, Instant)>,
    diagnostics: Value,
    compatibility: Value,
    confirm_launch: bool,
    screenshots: Value,
    image: Option<String>,
    thumbnails: Value,
    screenshot_page: usize,
    update: Value,
    login: Value,
    next_poll: Instant,
    dirty: Option<Instant>,
    background: bool,
    smoke: Option<Smoke>,
}
impl Launcher {
    fn new(ctx: &egui::Context) -> Self {
        Self::with_state(ctx, State::new(), true)
    }
    fn with_state(ctx: &egui::Context, state: State, background: bool) -> Self {
        egui_extras::install_image_loaders(ctx);
        let mut fonts = egui::FontDefinitions::default();
        fonts.font_data.insert(
            "Inter".into(),
            egui::FontData::from_static(include_bytes!("../assets/InterVariable.ttf")).into(),
        );
        fonts
            .families
            .get_mut(&egui::FontFamily::Proportional)
            .unwrap()
            .insert(0, "Inter".into());
        for (name, bytes) in [
            (
                "InterMedium",
                include_bytes!("../assets/Inter-Medium.ttf").as_slice(),
            ),
            (
                "InterSemiBold",
                include_bytes!("../assets/Inter-SemiBold.ttf").as_slice(),
            ),
            (
                "InterBold",
                include_bytes!("../assets/Inter-Bold.ttf").as_slice(),
            ),
        ] {
            fonts
                .font_data
                .insert(name.into(), egui::FontData::from_static(bytes).into());
            let mut fallback = fonts.families[&egui::FontFamily::Proportional].clone();
            fallback.insert(0, name.into());
            fonts
                .families
                .insert(egui::FontFamily::Name(name.into()), fallback);
        }
        ctx.set_fonts(fonts);
        let (tx, events) = mpsc::channel();
        let (sender, replies) = mpsc::channel();
        let wake = ctx.clone();
        let app = AppContext::new(
            Arc::new(state),
            tx,
            Arc::new(move || wake.request_repaint()),
        );
        let data = app.state().data.lock().unwrap().clone();
        let mut this = Self {
            context: ctx.clone(),
            settings: data.settings.clone(),
            data,
            app,
            runtime: tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .expect("Background runtime"),
            events,
            replies,
            sender,
            pending: HashSet::new(),
            page: 0,
            settings_tab: 0,
            activity_year: chrono::Datelike::year(&chrono::Local::now()),
            expanded_versions: HashSet::new(),
            error: None,
            toast: None,
            progress: (String::new(), 0.),
            account: Value::Null,
            account_open: false,
            nickname: String::new(),
            versions_open: false,
            fps_open: false,
            fps_plan: Value::Null,
            library: Value::Null,
            version_query: String::new(),
            downloaded: false,
            catalog: Value::Null,
            search_queued: false,
            query: String::new(),
            kind: "modpack".into(),
            catalog_tab: 0,
            offset: 0,
            mods: Value::Null,
            mod_scope: String::new(),
            mod_epoch: 0,
            mod_checked: None,
            mod_cache: HashMap::new(),
            diagnostics: Value::Null,
            compatibility: Value::Null,
            confirm_launch: false,
            screenshots: Value::Null,
            image: None,
            thumbnails: json!({}),
            screenshot_page: 0,
            update: Value::Null,
            login: Value::Null,
            next_poll: Instant::now(),
            dirty: None,
            background,
            smoke: std::env::var_os("KYRO_NATIVE_SMOKE").map(|p| Smoke {
                dir: p.into(),
                step: 0,
                waiting: false,
                since: Instant::now(),
            }),
        };
        this.refresh();
        if background {
            this.submit("versions", Job::Versions);
            this.scan();
        }
        this
    }
    fn busy(&self) -> bool {
        *self.app.state().busy.lock().unwrap()
            || self.pending.iter().any(|s| {
                [
                    "launch",
                    "prepare",
                    "install",
                    "fps-install",
                    "update-mod",
                    "toggle",
                    "java",
                    "update-launcher",
                    "preset",
                    "restore-preset",
                ]
                .contains(&s.as_str())
            })
    }
    fn refresh(&mut self) {
        self.data = self.app.state().data.lock().unwrap().clone();
        self.settings = self.data.settings.clone();
        if let Ok(s) = crate::snapshot(self.app.state()) {
            self.account = s["account"].clone();
        }
    }
    fn submit(&mut self, tag: &str, job: Job) {
        if self.pending.contains(tag) {
            return;
        }
        self.pending.insert(tag.into());
        let tag = tag.to_string();
        let app = self.app.clone();
        let tx = self.sender.clone();
        let job_scope = scope(&self.settings);
        let mod_epoch = self.mod_epoch;
        self.runtime.spawn(async move {
            let result = execute(app.clone(), job).await;
            let _ = tx.send(Reply {
                tag,
                scope: job_scope,
                mod_epoch,
                result,
            });
            let _ = app.emit("reply", ());
        });
    }
    fn scan(&mut self) {
        if !self.busy() {
            self.submit("scan", Job::Scan);
        }
    }
    fn result(&mut self, r: Result<(), String>, message: &str) {
        match r {
            Ok(()) => {
                self.refresh();
                self.toast = Some((message.into(), Instant::now()));
            }
            Err(e) => self.error = Some(e),
        }
    }
    fn save(&mut self) -> bool {
        let r = crate::save_settings(self.app.state(), self.settings.clone());
        self.dirty = None;
        if let Err(e) = r {
            self.error = Some(e);
            self.refresh();
            false
        } else {
            self.refresh();
            true
        }
    }
    fn profile_changed(&mut self) {
        let key = scope(&self.settings);
        if let Some((data, checked)) = self.mod_cache.get(&key).cloned() {
            self.mods = data;
            self.mod_checked = Some(checked);
            self.mod_scope = key;
        } else {
            self.mods = Value::Null;
            self.mod_checked = None;
            self.mod_scope.clear();
        }
        self.compatibility = Value::Null;
        self.screenshots = Value::Null;
        self.thumbnails = json!({});
        self.screenshot_page = 0;
        self.catalog = Value::Null;
        if self
            .mod_checked
            .is_none_or(|t| t.elapsed() > Duration::from_secs(300))
        {
            self.scan();
        }
    }
    fn drain(&mut self, ctx: &egui::Context) {
        while let Ok(event) = self.events.try_recv() {
            match event {
                Event::Message(name, v) if name == "progress" => {
                    self.progress = (
                        text(&v, "message"),
                        v["fraction"].as_f64().unwrap_or(0.) as f32,
                    )
                }
                Event::Message(name, _) if name == "state-changed" => self.refresh(),
                Event::Window(a) => match a {
                    WindowAction::Hide => {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false))
                    }
                    WindowAction::Minimize => {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true))
                    }
                    WindowAction::Restore => {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
                        ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(false));
                        ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
                    }
                    WindowAction::Exit => ctx.send_viewport_cmd(egui::ViewportCommand::Close),
                },
                _ => {}
            }
        }
        while let Ok(reply) = self.replies.try_recv() {
            self.pending.remove(&reply.tag);
            if reply.tag == "search" && self.search_queued {
                self.search_queued = false;
                self.search();
                continue;
            }
            if reply.tag == "scan" && reply.mod_epoch != self.mod_epoch {
                continue;
            }
            if [
                "scan",
                "fps-plan",
                "screens",
                "image",
                "compatibility",
                "preflight",
                "diagnose",
                "search",
                "thumbnails",
            ]
            .contains(&reply.tag.as_str())
                && reply.scope != scope(&self.settings)
            {
                continue;
            }
            match reply.result {
                Err(e) => {
                    if reply.tag == "login-poll" {
                        self.login = Value::Null;
                    }
                    if reply.tag == "scan" {
                        self.mods = json!({"warning":e});
                        self.mod_checked = Some(Instant::now());
                        self.mod_scope = reply.scope;
                    } else {
                        self.error = Some(e);
                    }
                }
                Ok(v) => match reply.tag.as_str() {
                    "fps-plan" => self.fps_plan = v,
                    "fps-install" => {
                        self.fps_plan = Value::Null;
                        self.fps_open = false;
                        self.mod_epoch += 1;
                        self.mod_checked = None;
                        self.mod_cache.remove(&scope(&self.settings));
                        self.refresh();
                        self.progress = (String::new(), 0.);
                        self.toast = Some((
                            format!("FPS-буст: установлено {} файлов", v["installed"]),
                            Instant::now(),
                        ));
                        self.scan();
                    }
                    "versions" => self.library = v,
                    "search" => self.catalog = v,
                    "scan" => {
                        self.mods = v;
                        self.mod_scope = reply.scope;
                        self.mod_checked = Some(Instant::now());
                        self.mod_cache.insert(
                            self.mod_scope.clone(),
                            (self.mods.clone(), self.mod_checked.unwrap()),
                        );
                    }
                    "diagnose" => self.diagnostics = v,
                    "compatibility" => self.compatibility = v,
                    "preflight" => {
                        self.compatibility = v;
                        if self.compatibility["errors"].as_u64().unwrap_or(0) > 0 {
                            self.confirm_launch = true;
                        } else {
                            self.submit("launch", Job::Launch);
                        }
                    }
                    "screens" => {
                        self.screenshots = v;
                        self.screenshot_page = 0;
                        self.load_thumbnails();
                    }
                    "thumbnails" => self.thumbnails = v,
                    "image" => {
                        if let Some(s) = v.as_str() {
                            self.image = Some(s.into());
                        }
                    }
                    "check-update" => self.update = v,
                    "login-begin" => {
                        self.login = v;
                        self.next_poll = Instant::now()
                            + Duration::from_secs(self.login["interval"].as_u64().unwrap_or(5));
                    }
                    "login-poll" => {
                        if v["status"] == "complete" {
                            self.login = Value::Null;
                            self.refresh();
                            self.toast = Some(("Вход выполнен".into(), Instant::now()));
                        } else {
                            self.next_poll = Instant::now()
                                + Duration::from_secs(if v["status"] == "slow_down" {
                                    10
                                } else {
                                    5
                                });
                        }
                    }
                    _ => {
                        self.mod_epoch += 1;
                        self.mod_checked = None;
                        self.mods = Value::Null;
                        self.mod_cache.remove(&scope(&self.settings));
                        self.refresh();
                        self.progress = (String::new(), 0.);
                        self.toast = Some(("Готово".into(), Instant::now()));
                        self.scan();
                    }
                },
            }
        }
        if self.background
            && self
                .mod_checked
                .is_none_or(|t| t.elapsed() > Duration::from_secs(300))
            && !self.pending.contains("scan")
        {
            self.scan();
        }
        if !self.login.is_null()
            && Instant::now() >= self.next_poll
            && !self.pending.contains("login-poll")
        {
            self.submit(
                "login-poll",
                Job::LoginPoll(text(&self.login, "device_code")),
            );
        }
        if self
            .dirty
            .is_some_and(|t| t.elapsed() > Duration::from_millis(700))
            && !self.busy()
        {
            self.save();
        }
    }
    fn navigate(&mut self, page: usize) {
        if self.dirty.is_some() && !self.save() {
            return;
        }
        self.reset_focus();
        self.page = page;
        match page {
            1 if self.catalog.is_null() => self.search(),
            2 => self.submit("screens", Job::Screens),
            3 => {
                self.submit("diagnose", Job::Diagnose);
                self.submit("compatibility", Job::Compatibility);
            }
            _ => {}
        }
    }
    fn reset_focus(&self) {
        self.context.memory_mut(|m| {
            if let Some(id) = m.focused() {
                m.surrender_focus(id);
            }
        });
    }
    fn search(&mut self) {
        if self.pending.contains("search") {
            self.search_queued = true;
            return;
        }
        self.submit(
            "search",
            Job::Search(
                self.query.clone(),
                self.kind.clone(),
                self.settings.clone(),
                self.offset,
            ),
        );
    }
    fn open_versions(&mut self) {
        self.versions_open = true;
        self.expanded_versions.insert(self.settings.version.clone());
        if self.library.is_null() || !text(&self.library, "warning").is_empty() {
            self.submit("versions", Job::Versions);
        }
    }
    fn mod_manager(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui
                .add_enabled(!self.busy(), egui::Button::new("Проверить обновления"))
                .clicked()
            {
                self.scan();
            }
            if ui.button("Совместимость").clicked() {
                self.navigate(3);
            }
            if self.pending.contains("scan") {
                ui.spinner();
            }
        });
        let warning = text(&self.mods, "warning");
        if !warning.is_empty() {
            ui.label(RichText::new(warning).color(Color32::LIGHT_YELLOW));
        }
        let files = array(&self.mods, "files");
        if files.is_empty() {
            secondary(ui, "В текущем профиле пока нет модов.");
        }
        for m in files {
            Frame::new()
                .inner_margin(Margin::symmetric(0, 16))
                .show(ui, |ui| {
                    ui.set_min_width(ui.available_width());
                    ui.label(
                        RichText::new(text(&m, "name"))
                            .family(egui::FontFamily::Name("InterSemiBold".into()))
                            .size(16.),
                    );
                    secondary(ui, text(&m, "filename"));
                    ui.horizontal(|ui| {
                        let mut enabled = m["enabled"] == true;
                        if ui
                            .add_enabled(!self.busy(), egui::Checkbox::new(&mut enabled, "Включён"))
                            .changed()
                        {
                            self.submit("toggle", Job::Toggle(text(&m, "filename"), enabled));
                        }
                        let version = text(&m, "version_number");
                        if !version.is_empty() {
                            secondary(ui, version);
                        }
                        if !m["update"].is_null()
                            && ui
                                .add_enabled(
                                    !self.busy(),
                                    egui::Button::new(format!(
                                        "Обновить до {}",
                                        text(&m["update"], "version")
                                    )),
                                )
                                .clicked()
                        {
                            self.submit("update-mod", Job::UpdateMod(text(&m, "filename")));
                        }
                    });
                    let deps = array(&m, "dependencies");
                    if !deps.is_empty() {
                        secondary(
                            ui,
                            format!(
                                "Требует: {}",
                                deps.iter()
                                    .map(|d| text(d, "name"))
                                    .collect::<Vec<_>>()
                                    .join(", ")
                            ),
                        );
                    }
                });
            ui.separator();
        }
    }
    fn diagnostics(&mut self, ui: &mut egui::Ui) {
        title(
            ui,
            "Диагностика",
            "Причины вылетов и проверка совместимости модов",
        );
        if ui.button("Проверить снова").clicked() {
            self.submit("diagnose", Job::Diagnose);
            self.submit("compatibility", Job::Compatibility);
        }
        card().show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.label(
                RichText::new(text(&self.diagnostics, "title"))
                    .size(18.)
                    .family(egui::FontFamily::Name("InterSemiBold".into())),
            );
            for f in array(&self.diagnostics, "findings") {
                ui.label(RichText::new(text(&f, "title")).color(Color32::LIGHT_YELLOW));
                ui.label(text(&f, "detail"));
                secondary(ui, text(&f, "action"));
            }
            for line in array(&self.diagnostics, "evidence") {
                ui.label(
                    RichText::new(line.as_str().unwrap_or(""))
                        .monospace()
                        .size(12.),
                );
            }
            secondary(ui, text(&self.diagnostics, "note"));
            if ui.button("Открыть лог профиля").clicked() {
                let r = crate::open_folder(self.app.state(), "profile-log".into()).map(|_| ());
                self.result(r, "Лог открыт");
            }
        });
        ui.add_space(14.);
        card().show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.label(
                RichText::new("Совместимость модов")
                    .size(18.)
                    .family(egui::FontFamily::Name("InterSemiBold".into())),
            );
            secondary(
                ui,
                format!(
                    "Проверено: {} · Ошибок: {} · Предупреждений: {}",
                    self.compatibility["checked"].as_u64().unwrap_or(0),
                    self.compatibility["errors"].as_u64().unwrap_or(0),
                    self.compatibility["warnings"].as_u64().unwrap_or(0)
                ),
            );
            for issue in array(&self.compatibility, "issues") {
                ui.label(
                    RichText::new(text(&issue, "title"))
                        .family(egui::FontFamily::Name("InterSemiBold".into()))
                        .color(if issue["severity"] == "error" {
                            Color32::LIGHT_RED
                        } else {
                            Color32::LIGHT_YELLOW
                        }),
                );
                secondary(ui, text(&issue, "file"));
                ui.label(text(&issue, "detail"));
            }
        });
    }
    fn screenshots(&mut self, ui: &mut egui::Ui) {
        title(ui, "Скриншоты", "Снимки из выбранного профиля Minecraft");
        ui.horizontal(|ui| {
            if ui.button("Обновить").clicked() {
                self.submit("screens", Job::Screens);
            }
            if ui.button("Открыть папку").clicked() {
                let r = crate::open_folder(self.app.state(), "screenshots".into()).map(|_| ());
                self.result(r, "Папка открыта");
            }
        });
        let images = array(&self.screenshots, "images");
        if images.is_empty() {
            secondary(
                ui,
                "Пока нет снимков. Нажми F2 в игре — скриншот появится здесь.",
            );
        }
        let start = self.screenshot_page * 12;
        let page = images
            .iter()
            .skip(start)
            .take(12)
            .cloned()
            .collect::<Vec<_>>();
        let columns = if ui.available_width() < 900. { 2 } else { 3 };
        for row in page.chunks(columns) {
            ui.columns(columns, |cols| {
                for (col, image) in cols.iter_mut().zip(row) {
                    let name = text(image, "name");
                    card().inner_margin(10).show(col, |ui| {
                        if let Some(data) = self.thumbnails[&name].as_str() {
                            if let Some(uri) = include_data(ui.ctx(), data, &name) {
                                if ui
                                    .add(
                                        egui::Image::new(uri)
                                            .fit_to_exact_size(Vec2::new(
                                                ui.available_width(),
                                                110.,
                                            ))
                                            .corner_radius(9)
                                            .sense(egui::Sense::click()),
                                    )
                                    .clicked()
                                {
                                    self.submit("image", Job::Image(name.clone()));
                                }
                            }
                        } else {
                            ui.allocate_space(Vec2::new(ui.available_width(), 80.));
                        }
                        ui.add(egui::Label::new(RichText::new(&name).size(12.)).truncate());
                        if ui.button("Открыть").clicked() {
                            self.submit("image", Job::Image(name));
                        }
                    });
                }
            });
            ui.add_space(12.);
        }
        ui.horizontal(|ui| {
            if ui
                .add_enabled(
                    self.screenshot_page > 0 && !self.pending.contains("thumbnails"),
                    egui::Button::new("← Назад"),
                )
                .clicked()
            {
                self.screenshot_page -= 1;
                self.load_thumbnails();
            }
            secondary(ui, format!("Снимков: {}", images.len()));
            if ui
                .add_enabled(
                    start + 12 < images.len() && !self.pending.contains("thumbnails"),
                    egui::Button::new("Далее →"),
                )
                .clicked()
            {
                self.screenshot_page += 1;
                self.load_thumbnails();
            }
        });
    }
    fn load_thumbnails(&mut self) {
        let names = array(&self.screenshots, "images")
            .iter()
            .skip(self.screenshot_page * 12)
            .take(12)
            .map(|v| text(v, "name"))
            .collect();
        self.submit("thumbnails", Job::Thumbnails(names));
    }

    fn smoke_frame(&mut self, ctx: &egui::Context) {
        let Some(mut smoke) = self.smoke.take() else {
            return;
        };
        for e in ctx.input(|i| i.events.clone()) {
            if let egui::Event::Screenshot {
                image, user_data, ..
            } = e
            {
                if user_data
                    .data
                    .as_ref()
                    .and_then(|d| d.downcast_ref::<usize>())
                    .is_some_and(|n| *n == smoke.step)
                {
                    let bytes: Vec<u8> = image.pixels.iter().flat_map(|p| p.to_array()).collect();
                    let _ = std::fs::create_dir_all(&smoke.dir);
                    if let Err(e) = image::save_buffer(
                        smoke.dir.join(format!("screen-{}.png", smoke.step)),
                        &bytes,
                        image.width() as u32,
                        image.height() as u32,
                        image::ColorType::Rgba8,
                    ) {
                        self.error = Some(e.to_string());
                    }
                    smoke.step += 1;
                    smoke.waiting = false;
                    smoke.since = Instant::now();
                    match smoke.step {
                        1..=6 => self.navigate(smoke.step),
                        7 => {
                            self.page = 0;
                            self.open_versions();
                        }
                        8 => {
                            self.versions_open = false;
                            self.account_open = true;
                        }
                        9 => {
                            self.account_open = false;
                            self.navigate(6);
                            self.settings_tab = 1;
                        }
                        10 => {
                            self.settings_tab = 2;
                        }
                        11 => {
                            self.settings_tab = 3;
                        }
                        12 => {
                            self.navigate(0);
                        }
                        13 => {
                            egui::Popup::close_all(ctx);
                            self.settings.accent = "#659aff".into();
                        }
                        14 => {
                            self.settings.home_layout = "card".into();
                        }
                        15 => {
                            self.settings.home_layout = "orbit".into();
                            self.fps_open = true;
                            self.submit("fps-plan", Job::FpsPlan);
                        }
                        16 => {
                            self.fps_open = false;
                            self.mod_scope = scope(&self.settings);
                            self.mods = json!({"files":[{"filename":"sodium-fabric.jar","project_id":"AANobbMI","enabled":true}]});
                        }
                        _ => {
                            let report = json!({"release_count":array(&self.library,"releases").len(),"version_warning":self.library["warning"],"error":self.error,"pending":self.pending});
                            let _ = std::fs::write(
                                smoke.dir.join("report.json"),
                                serde_json::to_vec_pretty(&report).unwrap_or_default(),
                            );
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    }
                }
            }
        }
        if smoke.step < 17
            && !smoke.waiting
            && smoke.since.elapsed() > Duration::from_millis(1200)
            && (self.pending.is_empty() || smoke.since.elapsed() > Duration::from_secs(15))
        {
            ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::new(
                smoke.step,
            )));
            smoke.waiting = true;
        }
        self.smoke = Some(smoke);
        ctx.request_repaint_after(Duration::from_millis(100));
    }
}
fn include_data(ctx: &egui::Context, data: &str, name: &str) -> Option<String> {
    let cache_id = egui::Id::new(("kyro-image", name));
    if let Some((old, uri)) = ctx.data(|d| d.get_temp::<(Arc<str>, String)>(cache_id)) {
        if old.as_ref() == data {
            return Some(uri);
        }
        ctx.forget_image(&uri);
    }
    let (_, encoded) = data.split_once(",")?;
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(encoded)
        .ok()?;
    use sha1::{Digest, Sha1};
    let hash = format!("{:x}", Sha1::digest(&bytes));
    let uri = format!("bytes://{name}-{hash}.png");
    ctx.include_bytes(uri.clone(), bytes);
    ctx.data_mut(|d| d.insert_temp(cache_id, (Arc::<str>::from(data), uri.clone())));
    Some(uri)
}
fn pick_image() -> Option<String> {
    let path = rfd::FileDialog::new()
        .add_filter("Изображение", &["png", "jpg", "jpeg", "webp"])
        .pick_file()?;
    let meta = std::fs::metadata(&path).ok()?;
    if meta.len() > 8 * 1024 * 1024 {
        return None;
    }
    let bytes = std::fs::read(&path).ok()?;
    let mime = match image::guess_format(&bytes).ok()? {
        image::ImageFormat::Png => "image/png",
        image::ImageFormat::Jpeg => "image/jpeg",
        image::ImageFormat::WebP => "image/webp",
        _ => return None,
    };
    Some(format!(
        "data:{mime};base64,{}",
        base64::engine::general_purpose::STANDARD.encode(bytes)
    ))
}
fn plural(n: usize) -> &'static str {
    if (11..=14).contains(&(n % 100)) {
        "обновлений"
    } else {
        match n % 10 {
            1 => "обновление",
            2..=4 => "обновления",
            _ => "обновлений",
        }
    }
}
impl eframe::App for Launcher {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.draw(ctx);
    }
    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.app.presence.shutdown();
        if self.dirty.is_some() && !self.busy() {
            let _ = crate::save_settings(self.app.state(), self.settings.clone());
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn test_app(ctx: &egui::Context) -> Launcher {
        let root = std::env::temp_dir().join(format!(
            "kyro-egui-{}",
            uuid::Uuid::new_v3(
                &uuid::Uuid::NAMESPACE_OID,
                format!("{:?}", Instant::now()).as_bytes()
            )
        ));
        let mut data = Data::default();
        data.add_offline("NativeTest").unwrap();
        for day in 0..25 {
            data.sessions.push(crate::store::Session {
                started: (chrono::Utc::now() - chrono::Duration::days(day)).to_rfc3339(),
                seconds: 3600,
                version: "1.21.1".into(),
                loader: "Fabric".into(),
                exit_code: Some(0),
            });
        }
        let state = State {
            root,
            data: std::sync::Mutex::new(data),
            busy: std::sync::Mutex::new(false),
        };
        Launcher::with_state(ctx, state, false)
    }
    fn strings(shapes: &[egui::epaint::ClippedShape]) -> String {
        fn shape(s: &egui::Shape, out: &mut String) {
            match s {
                egui::Shape::Text(t) => {
                    out.push_str(&t.galley.job.text);
                    out.push('\n');
                }
                egui::Shape::Vec(v) => {
                    for s in v {
                        shape(s, out)
                    }
                }
                _ => {}
            }
        }
        let mut out = String::new();
        for s in shapes {
            shape(&s.shape, &mut out);
        }
        out
    }
    #[test]
    fn native_screens_render_at_minimum_size() {
        let ctx = egui::Context::default();
        let mut app = test_app(&ctx);
        app.library = json!({"releases":[{"id":"1.21.1"}],"prepared":[]});
        for page in 0..7 {
            app.page = page;
            let mut out = None;
            for _ in 0..3 {
                out = Some(ctx.run(
                    egui::RawInput {
                        screen_rect: Some(egui::Rect::from_min_size(
                            egui::Pos2::ZERO,
                            Vec2::new(960., 600.),
                        )),
                        ..Default::default()
                    },
                    |ctx| app.draw(ctx),
                ));
            }
            let out = out.unwrap();
            let rendered = strings(&out.shapes);
            assert!(
                rendered.contains(MENU[page]),
                "Missing Russian label on page {page}: {rendered}"
            );
            let content = [
                "MINECRAFT · JAVA EDITION",
                "Проекты для твоего Minecraft",
                "Снимки из выбранного",
                "Причины вылетов",
                "Твоя история Minecraft",
                "Загрузки, обновления",
                "Язык интерфейса",
            ];
            assert!(
                rendered.contains(content[page]),
                "Page {page} content is missing: {rendered}"
            );
            for clipped in &out.shapes {
                if let egui::Shape::Text(t) = &clipped.shape {
                    if page > 0 && t.galley.job.text == MENU[page] {
                        assert!(t.pos.y >= 32., "Page title scrolled away: {}", t.pos.y);
                    }
                    if t.galley.job.text.contains("NativeTest") {
                        assert!(
                            t.pos.y + t.galley.size().y <= 600.,
                            "Account clipped at minimum height"
                        );
                    }
                }
            }
            assert!(!rendered.contains("PLAY"));
            assert!(!rendered.contains("Серверы"));
            assert!(!ctx.tessellate(out.shapes, out.pixels_per_point).is_empty());
        }
        app.versions_open = true;
        let mut out = None;
        for _ in 0..3 {
            out = Some(ctx.run(egui::RawInput::default(), |ctx| app.dialogs(ctx)));
        }
        let rendered = strings(&out.unwrap().shapes);
        assert!(rendered.contains("Версии и ядра"));
        app.versions_open = false;
        app.account_open = true;
        let mut account_out = None;
        for _ in 0..3 {
            account_out = Some(ctx.run(egui::RawInput::default(), |ctx| app.dialogs(ctx)));
        }
        let account_out = account_out.unwrap();
        let button = account_out
            .shapes
            .iter()
            .find_map(|s| match &s.shape {
                egui::Shape::Text(t) if t.galley.job.text == "Добавить офлайн-аккаунт" => {
                    Some((t, s.clip_rect))
                }
                _ => None,
            })
            .expect("Offline account action");
        assert!(
            button.0.pos.y + button.0.galley.size().y <= button.1.bottom() + 1.,
            "Account button is clipped"
        );
    }
    #[test]
    fn approved_home_composition_is_preserved() {
        let ctx = egui::Context::default();
        let mut app = test_app(&ctx);
        let mut output = None;
        for _ in 0..3 {
            output = Some(ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        Vec2::new(1060., 600.),
                    )),
                    ..Default::default()
                },
                |ctx| app.draw(ctx),
            ));
        }
        let output = output.unwrap();
        let texts = output
            .shapes
            .iter()
            .filter_map(|s| {
                if let egui::Shape::Text(t) = &s.shape {
                    Some(t)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        let version = texts
            .iter()
            .find(|t| {
                t.galley.job.text == "1.21.1"
                    && t.galley
                        .job
                        .sections
                        .iter()
                        .any(|s| s.format.font_id.size >= 60.)
            })
            .expect("large version");
        assert!(
            (248.0..=255.0).contains(&version.pos.x),
            "Version must stay in the left column, got {}",
            version.pos.x
        );
        let core = texts
            .iter()
            .find(|t| {
                t.galley.job.text == "Fabric"
                    && t.galley
                        .job
                        .sections
                        .iter()
                        .any(|s| s.format.font_id.size >= 28.)
            })
            .expect("upright core selector");
        assert!((248.0..=255.0).contains(&core.pos.x));
        let play = texts
            .iter()
            .find(|t| t.galley.job.text == "Играть" && t.pos.x > 250.)
            .expect("Russian play button");
        assert!((350.0..=390.0).contains(&play.pos.x));
        assert!((350.0..=390.0).contains(&play.pos.y));
        assert!(!strings(&output.shapes).contains("NATIVE /"));
    }
    #[test]
    fn memory_slider_accepts_mouse_input_and_persists() {
        let ctx = egui::Context::default();
        let mut app = test_app(&ctx);
        app.page = 6;
        app.settings_tab = 1;
        let rect = egui::Rect::from_min_size(egui::Pos2::ZERO, Vec2::new(1060., 600.));
        for _ in 0..3 {
            let _ = ctx.run(
                egui::RawInput {
                    screen_rect: Some(rect),
                    ..Default::default()
                },
                |ctx| app.draw(ctx),
            );
        }
        let point = egui::Pos2::new(520., 366.);
        for pressed in [true, false] {
            let _ = ctx.run(
                egui::RawInput {
                    screen_rect: Some(rect),
                    events: vec![
                        egui::Event::PointerMoved(point),
                        egui::Event::PointerButton {
                            pos: point,
                            button: egui::PointerButton::Primary,
                            pressed,
                            modifiers: egui::Modifiers::default(),
                        },
                    ],
                    ..Default::default()
                },
                |ctx| app.draw(ctx),
            );
        }
        assert!(
            app.settings.memory_gb > 16,
            "Clicking the right side of the RAM slider must increase memory, got {}",
            app.settings.memory_gb
        );
        assert!(app.save());
        let saved: Data = serde_json::from_slice(
            &std::fs::read(app.app.state().root.join("state.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(saved.settings.memory_gb, app.settings.memory_gb);
    }
    #[test]
    fn catalog_icons_do_not_move_the_next_row() {
        let ctx = egui::Context::default();
        let mut app = test_app(&ctx);
        app.page = 1;
        let icon = "https://kyro.test/catalog-icon.png";
        ctx.include_bytes(icon, include_bytes!("../icons/icon.png").as_slice());
        app.catalog = json!({"total_hits":3,"hits":[
            {"project_id":"row-a","title":"ROW_A_TITLE","description":"First project description","author":"FirstAuthor","downloads":1000,"icon_url":icon},
            {"project_id":"row-b","title":"ROW_B_TITLE","description":"Second project description","author":"SecondAuthor","downloads":2000,"icon_url":icon},
            {"project_id":"row-c","title":"ROW_C_TITLE","description":"Third project description","author":"ThirdAuthor","downloads":3000,"icon_url":icon}
        ]});
        let mut output = None;
        for _ in 0..4 {
            output = Some(ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        Vec2::new(1060., 900.),
                    )),
                    ..Default::default()
                },
                |ctx| app.draw(ctx),
            ));
        }
        let output = output.unwrap();
        let mut tops = Vec::new();
        for name in ["ROW_A_TITLE", "ROW_B_TITLE", "ROW_C_TITLE"] {
            let title = output
                .shapes
                .iter()
                .find_map(|s| match &s.shape {
                    egui::Shape::Text(t) if t.galley.job.text == name => Some((t, s.clip_rect)),
                    _ => None,
                })
                .expect("Project title");
            let bottom = title.0.pos.y + title.0.galley.size().y;
            assert!(
                title.0.pos.y >= title.1.top() && bottom <= title.1.bottom(),
                "Title escaped its row"
            );
            tops.push(title.0.pos.y);
        }
        for rows in tops.windows(2) {
            assert!(
                rows[1] - rows[0] >= 112.,
                "Icons compressed catalog rows: {:?}",
                tops
            );
        }
    }
    #[test]
    fn stale_profile_results_are_discarded() {
        let ctx = egui::Context::default();
        let mut app = test_app(&ctx);
        app.pending.insert("scan".into());
        app.sender
            .send(Reply {
                tag: "scan".into(),
                scope: "different-profile".into(),
                mod_epoch: 0,
                result: Ok(json!({"files":[{"update":{"id":"bad"}}]})),
            })
            .unwrap();
        app.drain(&ctx);
        assert!(app.mods.is_null());
        assert!(!app.pending.contains("scan"));
        app.mod_epoch = 1;
        app.pending.insert("scan".into());
        app.sender
            .send(Reply {
                tag: "scan".into(),
                scope: scope(&app.settings),
                mod_epoch: 0,
                result: Ok(json!({"files":[{"update":{"id":"stale"}}]})),
            })
            .unwrap();
        app.drain(&ctx);
        assert!(
            app.mods.is_null(),
            "A scan from before installation must be discarded"
        );
    }
    #[test]
    fn native_jobs_keep_profile_files_and_create_previews() {
        let ctx = egui::Context::default();
        let app = test_app(&ctx);
        let dir = app.app.state().instance_dir(&app.settings).unwrap();
        std::fs::create_dir_all(dir.join("screenshots")).unwrap();
        let original = "key_key.forward:key.keyboard.w\nrenderDistance:18\n";
        std::fs::write(dir.join("options.txt"), original).unwrap();
        app.runtime
            .block_on(execute(app.app.clone(), Job::Preset("low".into())))
            .unwrap();
        let changed = std::fs::read_to_string(dir.join("options.txt")).unwrap();
        assert!(changed.contains("renderDistance:4"));
        assert!(changed.contains("key_key.forward:key.keyboard.w"));
        app.runtime
            .block_on(execute(app.app.clone(), Job::RestorePreset))
            .unwrap();
        assert_eq!(
            std::fs::read_to_string(dir.join("options.txt")).unwrap(),
            original
        );
        std::fs::write(
            dir.join("screenshots/test.png"),
            include_bytes!("../icons/icon.png"),
        )
        .unwrap();
        let list = app
            .runtime
            .block_on(execute(app.app.clone(), Job::Screens))
            .unwrap();
        assert_eq!(array(&list, "images").len(), 1);
        let previews = app
            .runtime
            .block_on(execute(
                app.app.clone(),
                Job::Thumbnails(vec!["test.png".into()]),
            ))
            .unwrap();
        assert!(previews["test.png"]
            .as_str()
            .unwrap()
            .starts_with("data:image/png;base64,"));
        let mods = app
            .runtime
            .block_on(execute(app.app.clone(), Job::Scan))
            .unwrap();
        assert!(array(&mods, "files").is_empty());
        let report = app
            .runtime
            .block_on(execute(app.app.clone(), Job::Compatibility))
            .unwrap();
        assert_eq!(report["errors"], 0);
    }
    #[test]
    #[ignore = "requires an isolated existing Minecraft cache in KYRO_INTEGRATION_DATA"]
    fn native_prepare_uses_existing_game_backend() {
        let root = std::path::PathBuf::from(
            std::env::var_os("KYRO_INTEGRATION_DATA").expect("isolated test data"),
        );
        let root = root.canonicalize().unwrap();
        let target = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../target")
            .canonicalize()
            .unwrap();
        assert!(
            root.starts_with(target),
            "Integration data must be under the workspace target directory"
        );
        let data: Data =
            serde_json::from_slice(&std::fs::read(root.join("state.json")).unwrap()).unwrap();
        let state = State {
            root,
            data: std::sync::Mutex::new(data),
            busy: std::sync::Mutex::new(false),
        };
        let (tx, rx) = mpsc::channel();
        let app = AppContext::new(Arc::new(state), tx, Arc::new(|| {}));
        tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(execute(app.clone(), Job::Prepare))
            .unwrap();
        assert!(!*app.state().busy.lock().unwrap());
        assert!(rx
            .try_iter()
            .any(|event| matches!(event,Event::Message(name,_) if name=="progress")));
    }
    #[test]
    fn russian_update_count() {
        assert_eq!(plural(1), "обновление");
        assert_eq!(plural(3), "обновления");
        assert_eq!(plural(11), "обновлений");
        assert_eq!(plural(21), "обновление");
    }
}
