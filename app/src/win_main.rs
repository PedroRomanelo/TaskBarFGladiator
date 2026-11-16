//! Integração com o Windows: liga os eventos da plataforma ao controlador.
//!
//! Esta é a única parte do aplicativo que não pode ser testada fora do Windows. Ela não contém
//! regras de jogo: só posiciona, desenha, agenda temporizadores e grava arquivos.

use crate::config::{Config, MAX_CONFIG_BYTES};
use crate::controller::{App, Effects};
use crate::i18n::{Key, Lang, t};
use crate::{identity, logging, storage};
use game::{Content, GameState, Mode};
use platform::layout::{Placement, Rect, offset_from_x, offset_key, place_stage};
use platform::{Event, KnownFolder, MouseButton, Platform, TrayMenuItem, WindowId};
use render::scene::{PANEL_HEIGHT, PANEL_WIDTH, STAGE_HEIGHT, STAGE_WIDTH};
use render::{Hit, Surface, Theme, draw_panel, draw_stage, hit_test};
use std::path::PathBuf;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

const TIMER_ANIM: usize = 1;
const TIMER_AUTO: usize = 2;
const TIMER_SAVE: usize = 3;
const TIMER_RETRY: usize = 4;
const TIMER_STATS: usize = 5;

const MENU_TOGGLE_VISIBLE: u32 = 1;
const MENU_PAUSE: u32 = 2;
const MENU_PANEL: u32 = 3;
const MENU_REPOSITION: u32 = 4;
const MENU_QUIT: u32 = 5;

/// Arraste: o botão precisa ficar pressionado e o mouse andar alguns pixels.
const DRAG_MIN_PX: i32 = 4;

struct Drag {
    start_cursor_x: i32,
    start_left: i32,
    active: bool,
}

struct Shell {
    app: App,
    platform: Platform,
    save_path: PathBuf,
    config_path: PathBuf,
    worker: Option<storage::SaveWorker>,
    strip: Surface,
    strip_hits: Vec<Hit>,
    panel: Surface,
    panel_hits: Vec<Hit>,
    placement: Placement,
    taskbar: Option<platform::TaskbarInfo>,
    drag: Option<Drag>,
    drag_left: Option<i32>,
    theme: platform::SystemTheme,
    fullscreen: bool,
    locked: bool,
    /// Intervalo do temporizador de animação ligado (33 ms na rodada; mais lento parado).
    anim_timer: Option<u32>,
    auto_timer: Option<u32>,
    last_anim: Instant,
    quitting: bool,
}

fn now_seed() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(1, |d| d.as_nanos() as u64)
}

/// Ponto de entrada no Windows.
pub fn run() {
    platform::harden_process();
    let local = platform::known_folder(KnownFolder::LocalAppData)
        .map(|p| p.join(identity::DATA_DIR_NAME))
        .unwrap_or_else(|| PathBuf::from("."));
    let roaming = platform::known_folder(KnownFolder::RoamingAppData)
        .map(|p| p.join(identity::DATA_DIR_NAME))
        .unwrap_or_else(|| local.clone());
    let logs = local.join("logs");
    let profile = std::env::var("USERPROFILE").ok();
    logging::init(&logs, profile);
    logging::install_panic_hook(logs, env!("CARGO_PKG_VERSION"));
    logging::info(&format!("início v{}", env!("CARGO_PKG_VERSION")));

    let Some(_instance) = platform::single_instance(identity::ID_PREFIX) else {
        platform::notify_existing_instance(identity::ID_PREFIX);
        return;
    };

    let content = match Content::embedded() {
        Ok(c) => c,
        Err(e) => {
            logging::error(&format!("conteúdo inválido: {e}"));
            return;
        }
    };

    let config_path = roaming.join("config.json");
    let mut config =
        match storage::load_with_recovery(&config_path, MAX_CONFIG_BYTES, Config::from_bytes) {
            storage::Recovered::Primary(c) | storage::Recovered::Backup(c) => c,
            storage::Recovered::Missing => Config::default(),
            storage::Recovered::Corrupt => {
                logging::warn("config.json corrompido; usando padrão");
                Config::default()
            }
        };
    config.start_with_windows = platform::run_key_get(identity::DISPLAY_NAME).is_some();

    let save_path = roaming.join("save.json");
    let state = match storage::load_with_recovery(&save_path, game::save::MAX_SAVE_BYTES, |b| {
        match game::save::from_bytes(&content, b) {
            Ok(l) => {
                for c in &l.corrections {
                    logging::warn(&format!("save corrigido: {c}"));
                }
                Some(l.state)
            }
            Err(e) => {
                logging::warn(&format!("save inválido: {e}"));
                None
            }
        }
    }) {
        storage::Recovered::Primary(s) => s,
        storage::Recovered::Backup(s) => {
            logging::warn("save.json inválido; recuperado de save.json.bak");
            s
        }
        storage::Recovered::Missing => GameState::new(&content, "Gladiador", now_seed()),
        storage::Recovered::Corrupt => {
            logging::error(
                "save e backup inválidos; novo jogo (arquivos preservados como .corrupt)",
            );
            GameState::new(&content, "Gladiador", now_seed())
        }
    };

    let lang = if platform::ui_language_is_portuguese() {
        Lang::Pt
    } else {
        Lang::En
    };
    let platform = match Platform::new(identity::ID_PREFIX, identity::DISPLAY_NAME) {
        Ok(p) => p,
        Err(e) => {
            logging::error(&format!("plataforma: {e}"));
            return;
        }
    };
    let worker = storage::SaveWorker::start()
        .map_err(|e| logging::error(&format!("thread de gravação: {e}")))
        .ok();
    let mut shell = Shell {
        app: App::new(content, state, config, lang),
        theme: platform::theme(),
        platform,
        save_path,
        config_path,
        worker,
        strip: Surface::new(1, 1),
        strip_hits: Vec::with_capacity(16),
        panel: Surface::new(1, 1),
        panel_hits: Vec::with_capacity(64),
        placement: Placement::Hidden,
        taskbar: None,
        drag: None,
        drag_left: None,
        fullscreen: false,
        locked: false,
        anim_timer: None,
        auto_timer: None,
        last_anim: Instant::now(),
        quitting: false,
    };
    shell.platform.track_foreground();
    shell.fullscreen = platform::fullscreen_app_active();
    shell.relayout();
    shell.platform.set_timer(TIMER_STATS, 600_000, 60_000);
    shell.sync_timers();

    platform::run(|event| {
        shell.handle(event);
        shell.sync_timers();
    });
    logging::info("fim");
}

impl Shell {
    fn handle(&mut self, event: Event) {
        match event {
            Event::Timer(TIMER_ANIM) => {
                let now = Instant::now();
                let dt = now.duration_since(self.last_anim).as_millis().min(100) as u32;
                self.last_anim = now;
                let fx = self.app.on_anim_tick(dt);
                self.apply(fx);
            }
            Event::Timer(TIMER_AUTO) => {
                self.platform.kill_timer(TIMER_AUTO);
                self.auto_timer = None;
                if self.visible() {
                    let fx = self.app.on_auto_tick();
                    self.apply(fx);
                }
            }
            Event::Timer(TIMER_SAVE) => {
                self.platform.kill_timer(TIMER_SAVE);
                self.save_game(false);
            }
            Event::Timer(TIMER_RETRY) => {
                self.platform.kill_timer(TIMER_RETRY);
                self.relayout();
            }
            Event::Timer(TIMER_STATS) => {
                let m = platform::memory_stats();
                logging::info(&format!(
                    "memória: private={}KB ws={}KB gdi={} user={}",
                    m.private_bytes / 1024,
                    m.working_set / 1024,
                    m.gdi_objects,
                    m.user_objects
                ));
                if let Some(w) = &self.worker {
                    for e in w.take_errors() {
                        logging::error(&format!("gravação: {e}"));
                    }
                }
            }
            Event::Timer(_) => {}
            Event::MouseDown {
                window: WindowId::Strip,
                button: MouseButton::Left,
                ..
            } => {
                let (cx, _) = self.platform.cursor_pos();
                if let Placement::AboveTaskbar(r) | Placement::Floating(r) = self.placement {
                    self.drag = Some(Drag {
                        start_cursor_x: cx,
                        start_left: r.left,
                        active: false,
                    });
                    self.platform.capture_mouse(true);
                }
            }
            Event::MouseMove {
                window: WindowId::Strip,
                x,
                y,
            } => {
                let (cx, _) = self.platform.cursor_pos();
                if let Some(d) = &mut self.drag {
                    let dx = cx - d.start_cursor_x;
                    if dx.abs() >= DRAG_MIN_PX {
                        d.active = true;
                    }
                    if d.active {
                        self.drag_left = Some(d.start_left + dx);
                        self.relayout_with_drag();
                    }
                } else {
                    // Realce do botão sob o mouse e, na luta, a explicação da ação na legenda.
                    let fx = self.app.set_hover(hit_test(&self.strip_hits, x, y));
                    self.apply(fx);
                }
            }
            Event::MouseLeave {
                window: WindowId::Strip,
            } => {
                let fx = self.app.set_hover(None);
                self.apply(fx);
            }
            Event::MouseLeave { .. } => {}
            Event::MouseUp {
                window: WindowId::Strip,
                button: MouseButton::Left,
                x,
                y,
            } => {
                self.platform.capture_mouse(false);
                let drag = self.drag.take();
                if drag.as_ref().is_some_and(|d| d.active) {
                    self.finish_drag();
                } else if let Some(id) = hit_test(&self.strip_hits, x, y) {
                    let fx = self.app.on_strip_hit(id);
                    self.apply(fx);
                }
            }
            Event::MouseUp {
                window: WindowId::Strip,
                button: MouseButton::Right,
                ..
            } => self.open_panel(),
            Event::MouseUp {
                window: WindowId::Panel,
                button: MouseButton::Left,
                x,
                y,
            } => {
                if let Some(id) = hit_test(&self.panel_hits, x, y) {
                    let fx = self.app.on_panel_hit(id);
                    self.apply(fx);
                }
            }
            Event::MouseDown { .. } | Event::MouseMove { .. } | Event::MouseUp { .. } => {}
            Event::Key(0x1B) => self.close_panel(),
            Event::Key(_) => {}
            Event::PanelDeactivated => self.close_panel(),
            Event::TaskbarMoved | Event::DisplayChanged | Event::Resumed => self.relayout(),
            Event::ExplorerRestarted => {
                logging::info("Explorer reiniciado");
                self.platform.on_explorer_restarted();
                self.relayout();
            }
            Event::ForegroundChanged => {
                self.platform.track_foreground();
                let fs = platform::fullscreen_app_active();
                if fs != self.fullscreen {
                    self.fullscreen = fs;
                    self.relayout();
                } else if self.visible() {
                    self.platform.reassert_topmost();
                }
            }
            Event::ThemeChanged => {
                self.theme = platform::theme();
                self.redraw_strip();
            }
            Event::SessionLocked => {
                self.locked = true;
                self.relayout();
            }
            Event::SessionUnlocked => {
                self.locked = false;
                self.relayout();
            }
            Event::Suspending | Event::EndingSession => {
                self.save_game(true);
                self.save_config(true);
            }
            Event::TrayClick => {
                if self.app.config().hidden {
                    self.app.config_mut().hidden = false;
                    self.save_config(false);
                    self.relayout();
                } else {
                    self.open_panel();
                }
            }
            Event::TrayMenu => self.tray_menu(),
            Event::ShowRequested => {
                self.app.config_mut().hidden = false;
                self.relayout();
                self.open_panel();
            }
            Event::CloseRequested => self.quit(),
        }
    }

    fn visible(&self) -> bool {
        !self.app.config().hidden
            && !self.fullscreen
            && !self.locked
            && !matches!(self.placement, Placement::Hidden)
    }

    fn render_theme(&self) -> Theme {
        Theme {
            light: self.theme.light_taskbar,
            high_contrast: self.theme.high_contrast,
        }
    }

    fn relayout(&mut self) {
        self.taskbar = platform::taskbar_info();
        if self.taskbar.is_none() {
            // Explorer ausente ou travado: tenta de novo em 2 s (modo flutuante enquanto isso).
            self.platform.set_timer(TIMER_RETRY, 2_000, 500);
        }
        self.drag_left = None;
        self.relayout_with_drag();
    }

    fn relayout_with_drag(&mut self) {
        let offset = match (&self.taskbar, self.drag_left) {
            (Some(tb), Some(left)) => Some(offset_from_x(tb, left)),
            (Some(tb), None) => self.app.config().offsets.get(&offset_key(tb)).copied(),
            (None, _) => None,
        };
        self.placement = place_stage(
            self.taskbar.as_ref(),
            (STAGE_WIDTH, STAGE_HEIGHT),
            offset,
            self.platform.primary_work_area(),
            i32::from(self.app.config().size),
        );
        self.redraw_strip();
    }

    fn finish_drag(&mut self) {
        if let (Some(tb), Placement::AboveTaskbar(r)) = (&self.taskbar, self.placement) {
            let key = offset_key(tb);
            let off = offset_from_x(tb, r.left);
            self.app.config_mut().offsets.insert(key, off);
            self.save_config(false);
        }
        self.drag_left = None;
    }

    fn strip_rect(&self) -> Option<Rect> {
        match self.placement {
            Placement::AboveTaskbar(r) | Placement::Floating(r) => Some(r),
            Placement::Hidden => None,
        }
    }

    fn redraw_strip(&mut self) {
        if !self.visible() {
            self.platform.hide_strip();
            return;
        }
        let Some(r) = self.strip_rect() else {
            self.platform.hide_strip();
            return;
        };
        let (w, h) = (r.width().max(1) as usize, r.height().max(1) as usize);
        if self.strip.width() != w || self.strip.height() != h {
            self.strip.resize(w, h);
        }
        let theme = self.render_theme();
        draw_stage(
            &mut self.strip,
            self.app.stage(),
            theme,
            self.app.hover(),
            &mut self.strip_hits,
        );
        self.platform.present_strip(r, self.strip.pixels());
        self.platform.reassert_topmost();
    }

    fn panel_rect(&self) -> Rect {
        let (work, dpi) = self.platform.primary_work_area();
        let scale = ((dpi as i32 * 2) / 96).max(2);
        let w = PANEL_WIDTH * scale;
        let h = PANEL_HEIGHT * scale;
        let anchor = self.strip_rect().unwrap_or(Rect {
            left: work.right - w,
            top: work.bottom,
            right: work.right,
            bottom: work.bottom,
        });
        let left = anchor
            .left
            .clamp(work.left, (work.right - w).max(work.left));
        let top = (anchor.top - h - 4).clamp(work.top, (work.bottom - h).max(work.top));
        Rect {
            left,
            top,
            right: left + w,
            bottom: top + h,
        }
    }

    fn open_panel(&mut self) {
        let r = self.panel_rect();
        let (w, h) = (r.width() as usize, r.height() as usize);
        if self.panel.width() != w || self.panel.height() != h {
            self.panel = Surface::new(w, h);
        }
        let view = self.app.panel_view();
        draw_panel(&mut self.panel, &view, &mut self.panel_hits);
        self.platform
            .present_panel(identity::ID_PREFIX, r, self.panel.pixels());
    }

    fn close_panel(&mut self) {
        if self.platform.panel_open() {
            self.platform.close_panel();
            // Os buffers do painel são liberados ao fechar (docs/08, seção 3).
            self.panel = Surface::new(1, 1);
            self.panel_hits = Vec::new();
        }
    }

    fn apply(&mut self, fx: Effects) {
        if fx.quit {
            self.quit();
            return;
        }
        if fx.save_now {
            self.save_game(false);
        } else if fx.save_soon {
            self.platform.set_timer(TIMER_SAVE, 2_000, 1_000);
        }
        if fx.save_config {
            self.save_config(false);
        }
        if let Some(on) = fx.start_with_windows {
            let exe = std::env::current_exe().ok();
            let ok = platform::run_key_set(
                identity::DISPLAY_NAME,
                if on { exe.as_deref() } else { None },
            );
            if !ok {
                logging::warn("não foi possível alterar a inicialização automática");
            }
        }
        if fx.reset_position {
            if let Some(tb) = &self.taskbar {
                let key = offset_key(tb);
                self.app.config_mut().offsets.remove(&key);
            }
            self.relayout();
        } else if fx.relayout {
            self.relayout();
        }
        if fx.close_panel {
            self.close_panel();
        } else if fx.open_panel || (fx.redraw_panel && self.platform.panel_open()) {
            self.open_panel();
        }
        if fx.redraw_strip {
            self.redraw_strip();
        }
    }

    fn sync_timers(&mut self) {
        if self.quitting {
            return;
        }
        let interval = if !self.visible() {
            None
        } else if self.app.anim_busy() {
            Some(33)
        } else {
            self.app.ambient_ms()
        };
        if interval != self.anim_timer {
            match interval {
                Some(ms) => {
                    if self.anim_timer.is_none() {
                        self.last_anim = Instant::now();
                    }
                    self.platform.set_timer(TIMER_ANIM, ms, ms / 4);
                }
                None => self.platform.kill_timer(TIMER_ANIM),
            }
            self.anim_timer = interval;
        }
        let want = if self.visible() {
            if self.app.anim_busy() {
                self.app.auto_delay_ms().map(|_| 200)
            } else {
                self.app.auto_delay_ms()
            }
        } else {
            None
        };
        if want != self.auto_timer {
            match want {
                Some(ms) => self.platform.set_timer(TIMER_AUTO, ms, ms / 10),
                None => self.platform.kill_timer(TIMER_AUTO),
            }
            self.auto_timer = want;
        }
    }

    fn save_game(&mut self, synchronous: bool) {
        match game::save::to_bytes(self.app.content(), self.app.state()) {
            Ok(bytes) => {
                if synchronous || self.worker.is_none() {
                    if let Err(e) = storage::write_atomic(&self.save_path, &bytes) {
                        logging::error(&format!("save: {e}"));
                    }
                } else if let Some(w) = &self.worker {
                    w.save(self.save_path.clone(), bytes);
                }
            }
            Err(e) => logging::error(&format!("serialização do save: {e}")),
        }
    }

    fn save_config(&mut self, synchronous: bool) {
        let bytes = self.app.config().to_bytes();
        if synchronous || self.worker.is_none() {
            if let Err(e) = storage::write_atomic(&self.config_path, &bytes) {
                logging::error(&format!("config: {e}"));
            }
        } else if let Some(w) = &self.worker {
            w.save(self.config_path.clone(), bytes);
        }
    }

    fn tray_menu(&mut self) {
        let l = self.app.lang();
        let hidden = self.app.config().hidden;
        let auto = matches!(self.app.state().mode, Mode::Auto(_));
        let paused = self.app.paused();
        let items = [
            TrayMenuItem::Item {
                id: MENU_TOGGLE_VISIBLE,
                text: t(l, if hidden { Key::TrayShow } else { Key::TrayHide }),
                checked: false,
                enabled: true,
            },
            TrayMenuItem::Item {
                id: MENU_PAUSE,
                text: t(
                    l,
                    if paused {
                        Key::TrayResume
                    } else {
                        Key::TrayPause
                    },
                ),
                checked: false,
                enabled: auto,
            },
            TrayMenuItem::Item {
                id: MENU_PANEL,
                text: t(l, Key::TrayPanel),
                checked: false,
                enabled: true,
            },
            TrayMenuItem::Item {
                id: MENU_REPOSITION,
                text: t(l, Key::TrayReposition),
                checked: false,
                enabled: true,
            },
            TrayMenuItem::Separator,
            TrayMenuItem::Item {
                id: MENU_QUIT,
                text: t(l, Key::TrayQuit),
                checked: false,
                enabled: true,
            },
        ];
        match self.platform.tray_menu(&items) {
            Some(MENU_TOGGLE_VISIBLE) => {
                let c = self.app.config_mut();
                c.hidden = !c.hidden;
                self.save_config(false);
                self.relayout();
            }
            Some(MENU_PAUSE) => {
                let fx = self.app.set_paused(!paused);
                self.apply(fx);
            }
            Some(MENU_PANEL) => self.open_panel(),
            Some(MENU_REPOSITION) => self.apply(Effects {
                reset_position: true,
                save_config: true,
                ..Default::default()
            }),
            Some(MENU_QUIT) => self.quit(),
            _ => {}
        }
    }

    fn quit(&mut self) {
        if self.quitting {
            return;
        }
        self.quitting = true;
        self.close_panel();
        self.save_game(true);
        self.save_config(true);
        if let Some(mut w) = self.worker.take() {
            w.finish();
        }
        for id in [TIMER_ANIM, TIMER_AUTO, TIMER_SAVE, TIMER_RETRY, TIMER_STATS] {
            self.platform.kill_timer(id);
        }
        self.platform.quit();
    }
}
