//! Barra de tarefas, tela cheia, flyouts do shell e hooks de WinEvent (fora de contexto).

use super::queue;
use super::util::{from_wide, wide};
use crate::event::Event;
use crate::layout::{Edge, Rect, TaskbarInfo};
use std::cell::Cell;
use windows_sys::Win32::Foundation::{HWND, RECT};
use windows_sys::Win32::Graphics::Gdi::{
    GetMonitorInfoW, HMONITOR, MONITOR_DEFAULTTONEAREST, MONITOR_DEFAULTTOPRIMARY, MONITORINFO,
    MONITORINFOEXW, MonitorFromWindow,
};
use windows_sys::Win32::UI::Accessibility::{HWINEVENTHOOK, SetWinEventHook, UnhookWinEvent};
use windows_sys::Win32::UI::HiDpi::{GetDpiForMonitor, MDT_EFFECTIVE_DPI};
use windows_sys::Win32::UI::Shell::{
    ABE_BOTTOM, ABE_LEFT, ABE_RIGHT, ABE_TOP, ABM_GETSTATE, ABM_GETTASKBARPOS, ABS_AUTOHIDE,
    APPBARDATA, QUNS_BUSY, QUNS_PRESENTATION_MODE, QUNS_RUNNING_D3D_FULL_SCREEN, SHAppBarMessage,
    SHQueryUserNotificationState,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    EVENT_OBJECT_LOCATIONCHANGE, EVENT_SYSTEM_FOREGROUND, FindWindowExW, FindWindowW,
    GetClassNameW, GetForegroundWindow, GetWindowRect, GetWindowThreadProcessId, IsHungAppWindow,
    WINEVENT_OUTOFCONTEXT, WINEVENT_SKIPOWNPROCESS,
};

/// Classes de janela de flyouts do shell (Menu Iniciar, Pesquisa, central de notificações,
/// overflow da bandeja, Alt+Tab). Com uma delas em primeiro plano, a faixa não reafirma o topo.
/// Essas classes mudam entre versões do Windows: manter esta lista atualizada (docs/06, B2/B14).
pub const SHELL_FLYOUT_CLASSES: &[&str] = &[
    "Windows.UI.Core.CoreWindow",
    "XamlExplorerHostIslandWindow",
    "TopLevelWindowForOverflowXamlIsland",
    "NotifyIconOverflowWindow",
    "MultitaskingViewFrame",
    "ForegroundStaging",
    "TaskListThumbnailWnd",
    "Shell_Flyout",
];

/// Classes que representam a área de trabalho (não contam como tela cheia).
const DESKTOP_CLASSES: &[&str] = &[
    "Progman",
    "WorkerW",
    "Shell_TrayWnd",
    "Shell_SecondaryTrayWnd",
];

thread_local! {
    static TASKBAR_HWND: Cell<usize> = const { Cell::new(0) };
    static FOREGROUND_HWND: Cell<usize> = const { Cell::new(0) };
}

fn rect(r: RECT) -> Rect {
    Rect {
        left: r.left,
        top: r.top,
        right: r.right,
        bottom: r.bottom,
    }
}

fn class_of(hwnd: HWND) -> String {
    let mut buf = [0u16; 128];
    // SAFETY: `buf` tem 128 posições e o tamanho informado é o mesmo.
    let n = unsafe { GetClassNameW(hwnd, buf.as_mut_ptr(), buf.len() as i32) };
    if n <= 0 {
        return String::new();
    }
    from_wide(&buf)
}

fn find_taskbar() -> HWND {
    let class = wide("Shell_TrayWnd");
    // SAFETY: string terminada em zero que vive até o fim da chamada.
    unsafe { FindWindowW(class.as_ptr(), std::ptr::null()) }
}

fn monitor_info(mon: HMONITOR) -> Option<(Rect, Rect, String)> {
    let mut info = MONITORINFOEXW::default();
    info.monitorInfo.cbSize = std::mem::size_of::<MONITORINFOEXW>() as u32;
    // SAFETY: MONITORINFOEXW começa com MONITORINFO e `cbSize` indica o tamanho estendido.
    let ok = unsafe {
        GetMonitorInfoW(
            mon,
            (&mut info as *mut MONITORINFOEXW).cast::<MONITORINFO>(),
        )
    };
    if ok == 0 {
        return None;
    }
    Some((
        rect(info.monitorInfo.rcMonitor),
        rect(info.monitorInfo.rcWork),
        from_wide(&info.szDevice),
    ))
}

fn dpi_of(mon: HMONITOR) -> u32 {
    let (mut x, mut y) = (96u32, 96u32);
    // SAFETY: ponteiros para variáveis locais.
    let hr = unsafe { GetDpiForMonitor(mon, MDT_EFFECTIVE_DPI, &mut x, &mut y) };
    if hr < 0 { 96 } else { x.max(48) }
}

/// Lê as informações da barra de tarefas principal. `None` se a barra não existe, se o Explorer
/// está travado ou se os dados são incoerentes (o chamador usa o modo flutuante).
pub fn taskbar_info() -> Option<TaskbarInfo> {
    let hwnd = find_taskbar();
    if hwnd.is_null() {
        return None;
    }
    TASKBAR_HWND.with(|c| c.set(hwnd as usize));
    // `SHAppBarMessage` envia mensagens síncronas ao Explorer: com o Explorer travado, a faixa
    // travaria junto. Testa antes.
    // SAFETY: `hwnd` é uma janela (possivelmente de outro processo); a função só consulta estado.
    if unsafe { IsHungAppWindow(hwnd) } != 0 {
        return None;
    }
    let mut abd = APPBARDATA {
        cbSize: std::mem::size_of::<APPBARDATA>() as u32,
        hWnd: hwnd,
        ..Default::default()
    };
    // SAFETY: `abd` é válido e `cbSize` está preenchido.
    if unsafe { SHAppBarMessage(ABM_GETTASKBARPOS, &mut abd) } == 0 {
        return None;
    }
    let mut state = APPBARDATA {
        cbSize: std::mem::size_of::<APPBARDATA>() as u32,
        ..Default::default()
    };
    // SAFETY: idem.
    let st = unsafe { SHAppBarMessage(ABM_GETSTATE, &mut state) } as u32;
    let edge = match abd.uEdge {
        ABE_BOTTOM => Edge::Bottom,
        ABE_TOP => Edge::Top,
        ABE_LEFT => Edge::Left,
        ABE_RIGHT => Edge::Right,
        _ => return None,
    };
    // SAFETY: `hwnd` é válido.
    let mon = unsafe { MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST) };
    let (monitor, _work, device) = monitor_info(mon)?;
    let tray_class = wide("TrayNotifyWnd");
    // SAFETY: `hwnd` é válido; string terminada em zero.
    let tray_hwnd = unsafe {
        FindWindowExW(
            hwnd,
            std::ptr::null_mut(),
            tray_class.as_ptr(),
            std::ptr::null(),
        )
    };
    let tray = if tray_hwnd.is_null() {
        None
    } else {
        let mut r = RECT::default();
        // SAFETY: `tray_hwnd` é válido; `r` é um buffer local.
        (unsafe { GetWindowRect(tray_hwnd, &mut r) } != 0).then(|| rect(r))
    };
    Some(TaskbarInfo {
        rect: rect(abd.rc),
        edge,
        autohide: st & ABS_AUTOHIDE != 0,
        monitor,
        dpi: dpi_of(mon),
        device,
        tray,
    })
}

/// Área de trabalho e DPI do monitor principal.
pub fn primary_work_area() -> (Rect, u32) {
    // SAFETY: janela nula + flag "primário" = monitor principal.
    let mon = unsafe { MonitorFromWindow(std::ptr::null_mut(), MONITOR_DEFAULTTOPRIMARY) };
    match monitor_info(mon) {
        Some((_m, work, _)) => (work, dpi_of(mon)),
        None => (
            Rect {
                left: 0,
                top: 0,
                right: 1280,
                bottom: 720,
            },
            96,
        ),
    }
}

/// Verdadeiro se a janela em primeiro plano é um flyout do shell.
pub fn foreground_is_shell_flyout() -> bool {
    // SAFETY: sem pré-condições.
    let fg = unsafe { GetForegroundWindow() };
    if fg.is_null() {
        return false;
    }
    let class = class_of(fg);
    SHELL_FLYOUT_CLASSES.iter().any(|c| *c == class)
}

/// Verdadeiro se há um aplicativo em tela cheia (exclusiva, sem borda ou apresentação).
pub fn fullscreen_app_active() -> bool {
    let mut state = 0;
    // SAFETY: `state` é um buffer local.
    if unsafe { SHQueryUserNotificationState(&mut state) } >= 0
        && matches!(
            state,
            QUNS_BUSY | QUNS_RUNNING_D3D_FULL_SCREEN | QUNS_PRESENTATION_MODE
        )
    {
        return true;
    }
    // Heurística para tela cheia sem borda: janela em primeiro plano cobre o monitor inteiro e não
    // é a área de trabalho.
    // SAFETY: sem pré-condições.
    let fg = unsafe { GetForegroundWindow() };
    if fg.is_null() {
        return false;
    }
    let class = class_of(fg);
    if DESKTOP_CLASSES.iter().any(|c| *c == class) {
        return false;
    }
    let mut r = RECT::default();
    // SAFETY: `fg` é uma janela; `r` é um buffer local.
    if unsafe { GetWindowRect(fg, &mut r) } == 0 {
        return false;
    }
    // SAFETY: `fg` é uma janela.
    let mon = unsafe { MonitorFromWindow(fg, MONITOR_DEFAULTTONEAREST) };
    monitor_info(mon).is_some_and(|(m, _, _)| crate::layout::covers_monitor(&rect(r), &m))
}

/// Hook de WinEvent com dono.
#[derive(Debug)]
struct WinEventHook(HWINEVENTHOOK);

impl WinEventHook {
    fn new(event: u32, pid: u32, tid: u32) -> Option<Self> {
        // SAFETY: hook fora de contexto (sem DLL injetada); o callback é uma função estática.
        let h = unsafe {
            SetWinEventHook(
                event,
                event,
                std::ptr::null_mut(),
                Some(win_event),
                pid,
                tid,
                WINEVENT_OUTOFCONTEXT | WINEVENT_SKIPOWNPROCESS,
            )
        };
        (!h.is_null()).then_some(Self(h))
    }
}

impl Drop for WinEventHook {
    fn drop(&mut self) {
        // SAFETY: hook instalado por esta thread.
        unsafe { UnhookWinEvent(self.0) };
    }
}

/// Hooks ativos.
#[derive(Debug)]
pub struct Hooks {
    _foreground: Option<WinEventHook>,
    _taskbar: Option<WinEventHook>,
    foreground_location: Option<WinEventHook>,
    tracked_pid: u32,
}

impl Hooks {
    /// Instala o hook de troca de primeiro plano (global, baixa frequência) e o de posição restrito
    /// à thread da barra de tarefas (processo do Explorer).
    pub fn install() -> Self {
        let foreground = WinEventHook::new(EVENT_SYSTEM_FOREGROUND, 0, 0);
        let tb = find_taskbar();
        let taskbar = if tb.is_null() {
            None
        } else {
            TASKBAR_HWND.with(|c| c.set(tb as usize));
            let mut pid = 0;
            // SAFETY: `tb` é uma janela; `pid` é um buffer local.
            let tid = unsafe { GetWindowThreadProcessId(tb, &mut pid) };
            WinEventHook::new(EVENT_OBJECT_LOCATIONCHANGE, pid, tid)
        };
        Self {
            _foreground: foreground,
            _taskbar: taskbar,
            foreground_location: None,
            tracked_pid: 0,
        }
    }

    /// Observa mudanças de posição da janela em primeiro plano atual (vídeo que entra em tela
    /// cheia sem trocar de janela). O hook é restrito ao processo dessa janela.
    pub fn track_foreground(&mut self) {
        // SAFETY: sem pré-condições.
        let fg = unsafe { GetForegroundWindow() };
        FOREGROUND_HWND.with(|c| c.set(fg as usize));
        if fg.is_null() {
            self.foreground_location = None;
            self.tracked_pid = 0;
            return;
        }
        let mut pid = 0;
        // SAFETY: `fg` é uma janela; `pid` é um buffer local.
        let tid = unsafe { GetWindowThreadProcessId(fg, &mut pid) };
        if pid == self.tracked_pid && self.foreground_location.is_some() {
            return;
        }
        self.foreground_location = None;
        self.foreground_location = WinEventHook::new(EVENT_OBJECT_LOCATIONCHANGE, pid, tid);
        self.tracked_pid = pid;
    }
}

/// Callback de WinEvent: só enfileira.
unsafe extern "system" fn win_event(
    _hook: HWINEVENTHOOK,
    event: u32,
    hwnd: HWND,
    id_object: i32,
    _id_child: i32,
    _thread: u32,
    _time: u32,
) {
    // OBJID_WINDOW = 0: eventos da própria janela (ignora cursor, caret e controles internos).
    if event == EVENT_SYSTEM_FOREGROUND {
        queue::push(Event::ForegroundChanged);
        return;
    }
    if event == EVENT_OBJECT_LOCATIONCHANGE && id_object == 0 {
        let h = hwnd as usize;
        if h != 0 && h == TASKBAR_HWND.with(Cell::get) {
            queue::push(Event::TaskbarMoved);
        } else if h != 0 && h == FOREGROUND_HWND.with(Cell::get) {
            queue::push(Event::ForegroundChanged);
        }
    }
}
