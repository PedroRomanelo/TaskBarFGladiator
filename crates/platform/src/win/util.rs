//! Utilidades: strings UTF-16, último erro, loop de mensagens.

use windows_sys::Win32::Foundation::{GetLastError, POINT};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, GetCursorPos, GetMessageW, MSG, PostQuitMessage, TranslateMessage,
};

/// Converte para UTF-16 terminado em zero. O `Vec` precisa viver até o fim da chamada que usa o
/// ponteiro.
pub fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

/// Copia `s` para um buffer UTF-16 de tamanho fixo, truncando e terminando em zero.
pub fn copy_wide(dst: &mut [u16], s: &str) {
    let max = dst.len().saturating_sub(1);
    let mut n = 0;
    for (slot, c) in dst.iter_mut().zip(s.encode_utf16()).take(max) {
        *slot = c;
        n += 1;
    }
    if let Some(z) = dst.get_mut(n) {
        *z = 0;
    }
}

/// Converte um buffer UTF-16 terminado em zero.
pub fn from_wide(buf: &[u16]) -> String {
    let end = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
    String::from_utf16_lossy(&buf[..end])
}

/// Código do último erro da thread. Deve ser lido imediatamente após a chamada que falhou.
pub fn last_error() -> u32 {
    // SAFETY: sem pré-condições.
    unsafe { GetLastError() }
}

/// Processa uma mensagem. Retorna `false` em `WM_QUIT` ou erro.
pub fn pump_one() -> bool {
    let mut msg = MSG::default();
    // SAFETY: `msg` é um buffer válido; janela nula = todas as janelas da thread.
    let r = unsafe { GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0) };
    if r == 0 || r == -1 {
        return false;
    }
    // SAFETY: `msg` foi preenchida por `GetMessageW`.
    unsafe { TranslateMessage(&msg) };
    // SAFETY: idem.
    unsafe { DispatchMessageW(&msg) };
    true
}

/// Pede o fim do loop de mensagens.
pub fn post_quit() {
    // SAFETY: sem pré-condições.
    unsafe { PostQuitMessage(0) };
}

/// Posição do cursor em coordenadas de tela.
pub fn cursor_pos() -> (i32, i32) {
    let mut p = POINT::default();
    // SAFETY: `p` é um buffer válido.
    if unsafe { GetCursorPos(&mut p) } == 0 {
        return (0, 0);
    }
    (p.x, p.y)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wide_round_trip_with_accents() {
        let w = wide("João");
        assert_eq!(w.last(), Some(&0));
        assert_eq!(from_wide(&w), "João");
    }

    #[test]
    fn copy_wide_truncates() {
        let mut buf = [1u16; 4];
        copy_wide(&mut buf, "abcdef");
        assert_eq!(from_wide(&buf), "abc");
        assert_eq!(buf[3], 0);
    }
}
