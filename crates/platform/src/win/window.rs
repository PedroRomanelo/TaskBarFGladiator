//! Janelas em camadas (faixa e painel) e o procedimento de janela.

use super::gdi::LayeredBuffer;
use super::util::{last_error, wide};
use super::{Error, queue};
use crate::event::{Event, MouseButton, WindowId};
use crate::layout::Rect;
use std::cell::Cell;
use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    ReleaseCapture, SetCapture, SetFocus, TME_LEAVE, TRACKMOUSEEVENT, TrackMouseEvent,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CREATESTRUCTW, CS_HREDRAW, CS_VREDRAW, CreateWindowExW, DefWindowProcW, DestroyWindow,
    GWLP_USERDATA, GetWindowLongPtrW, HWND_TOPMOST, IDC_ARROW, IsWindowVisible, KillTimer,
    LoadCursorW, MA_NOACTIVATE, PBT_APMRESUMEAUTOMATIC, PBT_APMSUSPEND, RegisterClassExW,
    RegisterWindowMessageW, SW_HIDE, SW_SHOWNOACTIVATE, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE,
    SetCoalescableTimer, SetForegroundWindow, SetWindowLongPtrW, SetWindowPos, ShowWindow,
    WA_INACTIVE, WM_ACTIVATE, WM_APP, WM_CLOSE, WM_DISPLAYCHANGE, WM_DPICHANGED, WM_KEYDOWN,
    WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MOUSEACTIVATE, WM_MOUSEMOVE, WM_NCCREATE, WM_POWERBROADCAST,
    WM_QUERYENDSESSION, WM_RBUTTONDOWN, WM_RBUTTONUP, WM_SETTINGCHANGE, WM_SYSCOLORCHANGE,
    WM_TIMER, WM_WTSSESSION_CHANGE, WNDCLASSEXW, WS_EX_LAYERED, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW,
    WS_EX_TOPMOST, WS_POPUP, WTS_CONSOLE_CONNECT, WTS_CONSOLE_DISCONNECT, WTS_REMOTE_CONNECT,
    WTS_REMOTE_DISCONNECT, WTS_SESSION_LOCK, WTS_SESSION_UNLOCK,
};

/// `WM_MOUSELEAVE`: o `windows-sys` só expõe esta constante no módulo de controles comuns, que o
/// projeto não usa.
const WM_MOUSELEAVE: u32 = 0x02A3;

/// Mensagem de callback do ícone da bandeja.
pub const TRAY_CALLBACK: u32 = WM_APP + 1;

thread_local! {
    static MSG_TASKBAR_CREATED: Cell<u32> = const { Cell::new(0) };
    static MSG_SHOW_INSTANCE: Cell<u32> = const { Cell::new(0) };
    /// Verdadeiro enquanto a faixa aguarda o aviso de saída do mouse (`WM_MOUSELEAVE`).
    static STRIP_MOUSE_TRACKED: Cell<bool> = const { Cell::new(false) };
}

fn class_name(prefix: &str, id: WindowId) -> String {
    match id {
        WindowId::Strip => format!("{prefix}.Strip"),
        WindowId::Panel => format!("{prefix}.Panel"),
    }
}

/// Nome da mensagem registrada que uma segunda instância envia à primeira.
pub fn show_instance_message_name(prefix: &str) -> String {
    format!("{prefix}.ShowInstance")
}

/// Registra as classes de janela e as mensagens registradas.
pub fn register_classes(prefix: &str) -> Result<(), Error> {
    let tb = wide("TaskbarCreated");
    // SAFETY: string terminada em zero que vive até o fim da chamada.
    MSG_TASKBAR_CREATED.with(|m| m.set(unsafe { RegisterWindowMessageW(tb.as_ptr()) }));
    let show = wide(&show_instance_message_name(prefix));
    // SAFETY: idem.
    MSG_SHOW_INSTANCE.with(|m| m.set(unsafe { RegisterWindowMessageW(show.as_ptr()) }));

    // SAFETY: módulo do próprio executável.
    let instance = unsafe { GetModuleHandleW(std::ptr::null()) };
    // SAFETY: cursor padrão do sistema (instância nula).
    let cursor = unsafe { LoadCursorW(std::ptr::null_mut(), IDC_ARROW) };
    for id in [WindowId::Strip, WindowId::Panel] {
        let name = wide(&class_name(prefix, id));
        let class = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(wndproc),
            hInstance: instance,
            hCursor: cursor,
            lpszClassName: name.as_ptr(),
            ..Default::default()
        };
        // SAFETY: `class` é válido; `name` vive até o fim da chamada (o Windows copia o nome).
        if unsafe { RegisterClassExW(&class) } == 0 {
            let code = last_error();
            // ERROR_CLASS_ALREADY_EXISTS (1410): registro repetido é aceitável.
            if code != 1410 {
                return Err(Error {
                    what: "RegisterClassExW",
                    code,
                });
            }
        }
    }
    Ok(())
}

/// Janela em camadas com seu buffer de pixels.
#[derive(Debug)]
pub struct Window {
    hwnd: HWND,
    buffer: Option<LayeredBuffer>,
    visible: bool,
}

impl Window {
    /// Cria a janela (oculta).
    pub fn create(prefix: &str, id: WindowId) -> Result<Self, Error> {
        let class = wide(&class_name(prefix, id));
        let title = wide(prefix);
        let ex_style = match id {
            WindowId::Strip => WS_EX_LAYERED | WS_EX_TOOLWINDOW | WS_EX_TOPMOST | WS_EX_NOACTIVATE,
            WindowId::Panel => WS_EX_LAYERED | WS_EX_TOOLWINDOW | WS_EX_TOPMOST,
        };
        // SAFETY: módulo do próprio executável.
        let instance = unsafe { GetModuleHandleW(std::ptr::null()) };
        // SAFETY: classe registrada em `register_classes`; strings vivem até o fim da chamada;
        // `lpParam` carrega o identificador da janela, lido em WM_NCCREATE.
        let hwnd = unsafe {
            CreateWindowExW(
                ex_style,
                class.as_ptr(),
                title.as_ptr(),
                WS_POPUP,
                0,
                0,
                1,
                1,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                instance,
                (id as usize + 1) as *const core::ffi::c_void,
            )
        };
        if hwnd.is_null() {
            return Err(Error {
                what: "CreateWindowExW",
                code: last_error(),
            });
        }
        Ok(Self {
            hwnd,
            buffer: None,
            visible: false,
        })
    }

    /// Handle.
    pub fn hwnd(&self) -> HWND {
        self.hwnd
    }

    /// Verdadeiro se está visível.
    pub fn is_visible(&self) -> bool {
        // SAFETY: `hwnd` pertence a esta thread e vive enquanto `self` viver.
        self.visible && unsafe { IsWindowVisible(self.hwnd) } != 0
    }

    /// Ajusta o tamanho do buffer, copia os pixels e mostra a janela sem ativá-la.
    pub fn present(&mut self, rect: Rect, pixels: &[u32]) {
        let (w, h) = (rect.width(), rect.height());
        let resize = self
            .buffer
            .as_ref()
            .is_none_or(|b| b.width() != w || b.height() != h);
        if resize {
            // O buffer antigo é liberado antes de criar o novo (pico de memória menor).
            self.buffer = None;
            self.buffer = LayeredBuffer::new(w, h);
        }
        if let Some(b) = &mut self.buffer {
            b.present(self.hwnd, rect.left, rect.top, pixels);
        }
        if !self.visible {
            // SAFETY: janela desta thread.
            unsafe { ShowWindow(self.hwnd, SW_SHOWNOACTIVATE) };
            self.visible = true;
        }
    }

    /// Oculta.
    pub fn hide(&mut self) {
        if self.visible {
            // SAFETY: janela desta thread.
            unsafe { ShowWindow(self.hwnd, SW_HIDE) };
            self.visible = false;
        }
    }

    /// Coloca no topo do grupo topmost sem ativar.
    pub fn bring_to_top(&self) {
        // SAFETY: janela desta thread; flags não movem nem redimensionam.
        unsafe {
            SetWindowPos(
                self.hwnd,
                HWND_TOPMOST,
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
            )
        };
    }

    /// Ativa (painel recebe foco e teclado).
    pub fn activate(&self) {
        // SAFETY: janela desta thread.
        unsafe { SetForegroundWindow(self.hwnd) };
        // SAFETY: idem.
        unsafe { SetFocus(self.hwnd) };
    }

    /// Liga um temporizador agrupável.
    pub fn set_timer(&self, id: usize, ms: u32, tolerance_ms: u32) {
        // SAFETY: janela desta thread; sem callback (chega como WM_TIMER).
        unsafe { SetCoalescableTimer(self.hwnd, id, ms, None, tolerance_ms) };
    }

    /// Desliga um temporizador.
    pub fn kill_timer(&self, id: usize) {
        // SAFETY: janela desta thread.
        unsafe { KillTimer(self.hwnd, id) };
    }

    /// Captura/libera o mouse.
    pub fn capture(&self, on: bool) {
        if on {
            // SAFETY: janela desta thread.
            unsafe { SetCapture(self.hwnd) };
        } else {
            // SAFETY: sem pré-condições.
            unsafe { ReleaseCapture() };
        }
    }
}

impl Drop for Window {
    fn drop(&mut self) {
        // O buffer GDI é liberado antes da janela.
        self.buffer = None;
        // SAFETY: janela criada por esta thread; não é usada depois.
        unsafe { DestroyWindow(self.hwnd) };
    }
}

fn window_id(hwnd: HWND) -> Option<WindowId> {
    // SAFETY: leitura de um valor inteiro gravado em WM_NCCREATE (0 se ainda não gravado).
    match unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) } {
        1 => Some(WindowId::Strip),
        2 => Some(WindowId::Panel),
        _ => None,
    }
}

fn xy(lparam: LPARAM) -> (i32, i32) {
    let x = (lparam & 0xffff) as u16 as i16 as i32;
    let y = ((lparam >> 16) & 0xffff) as u16 as i16 as i32;
    (x, y)
}

fn setting_name_is(lparam: LPARAM, name: &str) -> bool {
    if lparam == 0 {
        return false;
    }
    let ptr = lparam as *const u16;
    let mut buf = Vec::with_capacity(32);
    for i in 0..64 {
        // SAFETY: em WM_SETTINGCHANGE, `lParam` não nulo aponta para uma string UTF-16 terminada em
        // zero; a leitura para no terminador ou em 64 caracteres.
        let c = unsafe { *ptr.add(i) };
        if c == 0 {
            break;
        }
        buf.push(c);
    }
    String::from_utf16_lossy(&buf) == name
}

/// Procedimento de janela: converte mensagens em eventos e retorna rápido.
unsafe extern "system" fn wndproc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if msg == WM_NCCREATE {
        let cs = lparam as *const CREATESTRUCTW;
        if !cs.is_null() {
            // SAFETY: em WM_NCCREATE, `lParam` aponta para a CREATESTRUCTW da criação.
            let id = unsafe { (*cs).lpCreateParams } as isize;
            // SAFETY: grava um inteiro na janela que está sendo criada.
            unsafe { SetWindowLongPtrW(hwnd, GWLP_USERDATA, id) };
        }
        // SAFETY: repassa ao procedimento padrão.
        return unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) };
    }
    let Some(id) = window_id(hwnd) else {
        // SAFETY: repassa ao procedimento padrão.
        return unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) };
    };
    let taskbar_created = MSG_TASKBAR_CREATED.with(Cell::get);
    let show_instance = MSG_SHOW_INSTANCE.with(Cell::get);
    match msg {
        WM_MOUSEACTIVATE if id == WindowId::Strip => return MA_NOACTIVATE as LRESULT,
        WM_LBUTTONDOWN | WM_RBUTTONDOWN => {
            let (x, y) = xy(lparam);
            let button = if msg == WM_LBUTTONDOWN {
                MouseButton::Left
            } else {
                MouseButton::Right
            };
            queue::push(Event::MouseDown {
                window: id,
                button,
                x,
                y,
            });
            return 0;
        }
        WM_LBUTTONUP | WM_RBUTTONUP => {
            let (x, y) = xy(lparam);
            let button = if msg == WM_LBUTTONUP {
                MouseButton::Left
            } else {
                MouseButton::Right
            };
            queue::push(Event::MouseUp {
                window: id,
                button,
                x,
                y,
            });
            return 0;
        }
        WM_MOUSEMOVE => {
            // O aviso de saída precisa ser pedido de novo a cada entrada do mouse. Em janela em
            // camadas ele chega assim que o cursor passa para um pixel transparente.
            if id == WindowId::Strip && !STRIP_MOUSE_TRACKED.with(Cell::get) {
                let mut track = TRACKMOUSEEVENT {
                    cbSize: std::mem::size_of::<TRACKMOUSEEVENT>() as u32,
                    dwFlags: TME_LEAVE,
                    hwndTrack: hwnd,
                    dwHoverTime: 0,
                };
                // SAFETY: estrutura válida durante a chamada; `hwnd` é a janela desta mensagem. A
                // função só registra o pedido (não envia mensagens de forma síncrona).
                if unsafe { TrackMouseEvent(&mut track) } != 0 {
                    STRIP_MOUSE_TRACKED.with(|t| t.set(true));
                }
            }
            let (x, y) = xy(lparam);
            queue::push(Event::MouseMove { window: id, x, y });
            return 0;
        }
        WM_MOUSELEAVE => {
            if id == WindowId::Strip {
                STRIP_MOUSE_TRACKED.with(|t| t.set(false));
            }
            queue::push(Event::MouseLeave { window: id });
            return 0;
        }
        WM_KEYDOWN => {
            queue::push(Event::Key(wparam as u32));
            return 0;
        }
        WM_TIMER => {
            queue::push(Event::Timer(wparam));
            return 0;
        }
        WM_ACTIVATE if id == WindowId::Panel => {
            if (wparam & 0xffff) as u32 == WA_INACTIVE {
                queue::push(Event::PanelDeactivated);
            }
        }
        WM_DISPLAYCHANGE | WM_DPICHANGED => queue::push(Event::DisplayChanged),
        WM_SETTINGCHANGE => {
            if setting_name_is(lparam, "ImmersiveColorSet") {
                queue::push(Event::ThemeChanged);
            } else if setting_name_is(lparam, "TraySettings") {
                queue::push(Event::TaskbarMoved);
            } else {
                queue::push(Event::ThemeChanged);
            }
        }
        WM_SYSCOLORCHANGE => queue::push(Event::ThemeChanged),
        WM_WTSSESSION_CHANGE => match wparam as u32 {
            WTS_SESSION_LOCK | WTS_CONSOLE_DISCONNECT | WTS_REMOTE_DISCONNECT => {
                queue::push(Event::SessionLocked);
            }
            WTS_SESSION_UNLOCK | WTS_CONSOLE_CONNECT | WTS_REMOTE_CONNECT => {
                queue::push(Event::SessionUnlocked);
            }
            _ => {}
        },
        WM_POWERBROADCAST => match wparam as u32 {
            PBT_APMSUSPEND => queue::push(Event::Suspending),
            PBT_APMRESUMEAUTOMATIC => queue::push(Event::Resumed),
            _ => {}
        },
        WM_QUERYENDSESSION => {
            // O evento é processado logo após este retorno, antes de WM_ENDSESSION: o aplicativo
            // salva de forma síncrona nesse momento.
            queue::push(Event::EndingSession);
            return 1;
        }
        WM_CLOSE => {
            queue::push(Event::CloseRequested);
            return 0;
        }
        TRAY_CALLBACK => {
            // NOTIFYICON_VERSION_4: o evento do mouse vem na palavra baixa de `lParam`.
            let ev = (lparam & 0xffff) as u32;
            match ev {
                WM_LBUTTONUP => queue::push(Event::TrayClick),
                WM_RBUTTONUP | 0x007B /* WM_CONTEXTMENU */ => queue::push(Event::TrayMenu),
                _ => {}
            }
            return 0;
        }
        m if m != 0 && m == taskbar_created => queue::push(Event::ExplorerRestarted),
        m if m != 0 && m == show_instance => queue::push(Event::ShowRequested),
        _ => {}
    }
    // SAFETY: repassa ao procedimento padrão.
    unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
}
