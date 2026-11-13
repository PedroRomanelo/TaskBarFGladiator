//! Implementação Windows. Único lugar do projeto com `unsafe`.
//!
//! Regras (ver `docs/01-arquitetura.md`, seção 5.2, e `docs/07-seguranca.md`):
//! - o procedimento de janela e os callbacks só convertem mensagens em [`Event`] e enfileiram;
//! - nenhuma função que envia mensagens é chamada enquanto a fila está emprestada;
//! - todo recurso Win32 tem dono com `Drop`.

mod gdi;
mod queue;
mod shell;
mod system;
mod tray;
mod util;
mod window;

pub use shell::{foreground_is_shell_flyout, fullscreen_app_active, taskbar_info};
pub use system::{
    InstanceGuard, MemoryStats, harden_process, known_folder, memory_stats,
    notify_existing_instance, run_key_get, run_key_set, single_instance, theme,
    ui_language_is_portuguese,
};
pub use tray::TrayMenuItem;

use crate::event::{Event, WindowId};
use crate::layout::Rect;
use std::time::{Duration, Instant};

/// Pasta conhecida do Windows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KnownFolder {
    /// `%APPDATA%` (Roaming).
    RoamingAppData,
    /// `%LOCALAPPDATA%`.
    LocalAppData,
}

/// Tema do sistema.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SystemTheme {
    /// A barra de tarefas usa tema claro.
    pub light_taskbar: bool,
    /// Alto contraste ligado.
    pub high_contrast: bool,
}

/// Erro de inicialização.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Error {
    /// Função que falhou.
    pub what: &'static str,
    /// Código de `GetLastError`.
    pub code: u32,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} falhou (erro {})", self.what, self.code)
    }
}

impl std::error::Error for Error {}

/// Limite de reafirmações de z-order por segundo (disputa com outro programa topmost).
const TOPMOST_PER_SECOND: u32 = 4;
/// Pausa após detectar disputa de z-order.
const TOPMOST_BACKOFF: Duration = Duration::from_secs(30);

/// Janelas, bandeja e hooks do jogo.
#[derive(Debug)]
pub struct Platform {
    strip: window::Window,
    panel: Option<window::Window>,
    tray: Option<tray::Tray>,
    hooks: shell::Hooks,
    tooltip: String,
    topmost_window_start: Instant,
    topmost_count: u32,
    topmost_backoff_until: Option<Instant>,
    contention_seconds: u32,
}

impl Platform {
    /// Cria a faixa (oculta), o ícone da bandeja e os hooks de eventos.
    pub fn new(class_prefix: &str, tooltip: &str) -> Result<Self, Error> {
        window::register_classes(class_prefix)?;
        let strip = window::Window::create(class_prefix, WindowId::Strip)?;
        system::register_session_notifications(strip.hwnd());
        let tray = tray::Tray::add(strip.hwnd(), tooltip);
        let hooks = shell::Hooks::install();
        Ok(Self {
            strip,
            panel: None,
            tray,
            hooks,
            tooltip: tooltip.to_owned(),
            topmost_window_start: Instant::now(),
            topmost_count: 0,
            topmost_backoff_until: None,
            contention_seconds: 0,
        })
    }

    /// Passa a observar a janela em primeiro plano atual (chamar em [`Event::ForegroundChanged`]).
    pub fn track_foreground(&mut self) {
        self.hooks.track_foreground();
    }

    /// Recria o que o reinício do Explorer destrói: hooks da barra e ícone da bandeja.
    pub fn on_explorer_restarted(&mut self) {
        self.hooks = shell::Hooks::install();
        self.tray = None;
        self.tray = tray::Tray::add(self.strip.hwnd(), &self.tooltip);
    }

    /// Posiciona e redimensiona a faixa e desenha os pixels (BGRA premultiplicado).
    pub fn present_strip(&mut self, rect: Rect, pixels: &[u32]) {
        self.strip.present(rect, pixels);
    }

    /// Oculta a faixa.
    pub fn hide_strip(&mut self) {
        self.strip.hide();
    }

    /// Reafirma a faixa no topo do grupo topmost, exceto quando um flyout do shell está em primeiro
    /// plano, e com limite de frequência para não entrar em disputa com outro programa topmost.
    pub fn reassert_topmost(&mut self) {
        let now = Instant::now();
        if let Some(until) = self.topmost_backoff_until {
            if now < until {
                return;
            }
            self.topmost_backoff_until = None;
            self.contention_seconds = 0;
        }
        if shell::foreground_is_shell_flyout() {
            return;
        }
        if now.duration_since(self.topmost_window_start) >= Duration::from_secs(1) {
            self.contention_seconds = if self.topmost_count >= TOPMOST_PER_SECOND {
                self.contention_seconds + 1
            } else {
                0
            };
            if self.contention_seconds >= 5 {
                self.topmost_backoff_until = Some(now + TOPMOST_BACKOFF);
                return;
            }
            self.topmost_window_start = now;
            self.topmost_count = 0;
        }
        if self.topmost_count >= TOPMOST_PER_SECOND {
            return;
        }
        self.topmost_count += 1;
        self.strip.bring_to_top();
    }

    /// Verdadeiro se houve disputa de z-order recente (para o log).
    pub fn topmost_contention(&self) -> bool {
        self.topmost_backoff_until.is_some()
    }

    /// Abre (ou move) o painel e desenha os pixels.
    pub fn present_panel(&mut self, class_prefix: &str, rect: Rect, pixels: &[u32]) {
        if self.panel.is_none() {
            self.panel = window::Window::create(class_prefix, WindowId::Panel).ok();
        }
        if let Some(p) = &mut self.panel {
            let first = !p.is_visible();
            p.present(rect, pixels);
            if first {
                p.activate();
            }
        }
    }

    /// Fecha o painel (a janela é destruída e seus buffers liberados).
    pub fn close_panel(&mut self) {
        self.panel = None;
    }

    /// Verdadeiro se o painel está aberto.
    pub fn panel_open(&self) -> bool {
        self.panel.is_some()
    }

    /// Liga um temporizador agrupável (o Windows pode atrasar até `tolerance_ms` para economizar
    /// energia).
    pub fn set_timer(&self, id: usize, ms: u32, tolerance_ms: u32) {
        self.strip.set_timer(id, ms, tolerance_ms);
    }

    /// Desliga um temporizador.
    pub fn kill_timer(&self, id: usize) {
        self.strip.kill_timer(id);
    }

    /// Captura o mouse na faixa (arrastar).
    pub fn capture_mouse(&self, on: bool) {
        self.strip.capture(on);
    }

    /// Posição do cursor em coordenadas de tela.
    pub fn cursor_pos(&self) -> (i32, i32) {
        util::cursor_pos()
    }

    /// Mostra o menu da bandeja e devolve o identificador escolhido.
    pub fn tray_menu(&self, items: &[TrayMenuItem<'_>]) -> Option<u32> {
        tray::show_menu(self.strip.hwnd(), items)
    }

    /// Área de trabalho e DPI do monitor principal (modo flutuante).
    pub fn primary_work_area(&self) -> (Rect, u32) {
        shell::primary_work_area()
    }

    /// Encerra o loop de mensagens.
    pub fn quit(&self) {
        util::post_quit();
    }
}

/// Executa o loop de mensagens até [`Platform::quit`]. Cada evento é entregue ao `handler` depois
/// que o procedimento de janela retornou (sem reentrância). O `handler` normalmente é dono da
/// [`Platform`].
pub fn run(mut handler: impl FnMut(Event)) {
    loop {
        if !util::pump_one() {
            break;
        }
        while let Some(event) = queue::pop() {
            handler(event);
        }
    }
}
