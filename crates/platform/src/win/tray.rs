//! Ícone da bandeja e menu de contexto.

use super::util::{copy_wide, wide};
use super::window::TRAY_CALLBACK;
use windows_sys::Win32::Foundation::HWND;
use windows_sys::Win32::UI::Shell::{
    NIF_ICON, NIF_MESSAGE, NIF_SHOWTIP, NIF_TIP, NIM_ADD, NIM_DELETE, NIM_SETVERSION,
    NOTIFYICON_VERSION_4, NOTIFYICONDATAW, Shell_NotifyIconW,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    AppendMenuW, CreatePopupMenu, DestroyMenu, HMENU, IDI_APPLICATION, LoadIconW, MF_CHECKED,
    MF_GRAYED, MF_SEPARATOR, MF_STRING, PostMessageW, SetForegroundWindow, TPM_NONOTIFY,
    TPM_RETURNCMD, TPM_RIGHTBUTTON, TrackPopupMenu, WM_NULL,
};

const TRAY_ID: u32 = 1;

/// Item do menu da bandeja.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrayMenuItem<'a> {
    /// Item com identificador, texto, marcado e habilitado.
    Item {
        /// Identificador devolvido ao escolher (≥ 1).
        id: u32,
        /// Texto.
        text: &'a str,
        /// Marca de seleção.
        checked: bool,
        /// Habilitado.
        enabled: bool,
    },
    /// Separador.
    Separator,
}

/// Ícone da bandeja (removido no `Drop`).
#[derive(Debug)]
pub struct Tray {
    hwnd: HWND,
}

fn data(hwnd: HWND) -> NOTIFYICONDATAW {
    NOTIFYICONDATAW {
        cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
        hWnd: hwnd,
        uID: TRAY_ID,
        ..Default::default()
    }
}

impl Tray {
    /// Adiciona o ícone. Retorna `None` se o shell recusar (por exemplo, Explorer ausente); o jogo
    /// continua funcionando sem bandeja.
    pub fn add(hwnd: HWND, tooltip: &str) -> Option<Self> {
        let mut d = data(hwnd);
        d.uFlags = NIF_ICON | NIF_TIP | NIF_MESSAGE | NIF_SHOWTIP;
        d.uCallbackMessage = TRAY_CALLBACK;
        // SAFETY: ícone padrão do sistema (instância nula).
        d.hIcon = unsafe { LoadIconW(std::ptr::null_mut(), IDI_APPLICATION) };
        copy_wide(&mut d.szTip, tooltip);
        // Sem NIF_GUID: o GUID fica associado ao caminho do executável e quebraria ao mover o exe
        // de pasta (docs/06, B17).
        // SAFETY: `d` é válido e `cbSize` está preenchido.
        if unsafe { Shell_NotifyIconW(NIM_ADD, &d) } == 0 {
            return None;
        }
        d.Anonymous.uVersion = NOTIFYICON_VERSION_4;
        // SAFETY: idem.
        unsafe { Shell_NotifyIconW(NIM_SETVERSION, &d) };
        Some(Self { hwnd })
    }
}

impl Drop for Tray {
    fn drop(&mut self) {
        let d = data(self.hwnd);
        // SAFETY: remove o ícone adicionado com o mesmo `hWnd`/`uID`.
        unsafe { Shell_NotifyIconW(NIM_DELETE, &d) };
    }
}

/// Menu criado com `CreatePopupMenu` (destruído no `Drop`).
struct Menu(HMENU);

impl Drop for Menu {
    fn drop(&mut self) {
        // SAFETY: menu criado por `CreatePopupMenu` e não anexado a uma janela.
        unsafe { DestroyMenu(self.0) };
    }
}

/// Mostra o menu na posição do cursor e devolve o identificador escolhido.
pub fn show_menu(hwnd: HWND, items: &[TrayMenuItem<'_>]) -> Option<u32> {
    // SAFETY: sem pré-condições.
    let menu = Menu(unsafe { CreatePopupMenu() });
    if menu.0.is_null() {
        return None;
    }
    for item in items {
        match *item {
            TrayMenuItem::Separator => {
                // SAFETY: `menu.0` é válido.
                unsafe { AppendMenuW(menu.0, MF_SEPARATOR, 0, std::ptr::null()) };
            }
            TrayMenuItem::Item {
                id,
                text,
                checked,
                enabled,
            } => {
                let t = wide(text);
                let mut flags = MF_STRING;
                if checked {
                    flags |= MF_CHECKED;
                }
                if !enabled {
                    flags |= MF_GRAYED;
                }
                // SAFETY: `menu.0` é válido; `t` vive até o fim da chamada (o menu copia o texto).
                unsafe { AppendMenuW(menu.0, flags, id as usize, t.as_ptr()) };
            }
        }
    }
    let (x, y) = super::util::cursor_pos();
    // Sequência documentada pela Microsoft para o menu fechar ao clicar fora: janela em primeiro
    // plano antes e WM_NULL depois (docs/06, B18).
    // SAFETY: janela desta thread.
    unsafe { SetForegroundWindow(hwnd) };
    // SAFETY: `menu.0` e `hwnd` são válidos; TPM_RETURNCMD devolve o item escolhido.
    let cmd = unsafe {
        TrackPopupMenu(
            menu.0,
            TPM_RETURNCMD | TPM_RIGHTBUTTON | TPM_NONOTIFY,
            x,
            y,
            0,
            hwnd,
            std::ptr::null(),
        )
    };
    // SAFETY: janela desta thread.
    unsafe { PostMessageW(hwnd, WM_NULL, 0, 0) };
    (cmd > 0).then_some(cmd as u32)
}
