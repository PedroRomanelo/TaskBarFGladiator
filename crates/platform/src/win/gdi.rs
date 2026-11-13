//! DIB section + `UpdateLayeredWindow`, com donos RAII para os objetos GDI.

use windows_sys::Win32::Foundation::{HWND, POINT, SIZE};
use windows_sys::Win32::Graphics::Gdi::{
    AC_SRC_ALPHA, AC_SRC_OVER, BI_RGB, BITMAPINFO, BITMAPINFOHEADER, BLENDFUNCTION,
    CreateCompatibleDC, CreateDIBSection, DIB_RGB_COLORS, DeleteDC, DeleteObject, GetDC, HBITMAP,
    HDC, HGDIOBJ, ReleaseDC, SelectObject,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{ULW_ALPHA, UpdateLayeredWindow};

/// DC de memória com uma DIB section de 32 bits selecionada.
#[derive(Debug)]
pub struct LayeredBuffer {
    dc: HDC,
    bitmap: HBITMAP,
    previous: HGDIOBJ,
    bits: *mut u32,
    width: i32,
    height: i32,
}

impl LayeredBuffer {
    /// Cria um buffer `width × height`. Retorna `None` se o GDI recusar (memória ou limite de
    /// objetos).
    pub fn new(width: i32, height: i32) -> Option<Self> {
        if width <= 0 || height <= 0 {
            return None;
        }
        // SAFETY: DC nulo = DC compatível com a tela.
        let dc = unsafe { CreateCompatibleDC(std::ptr::null_mut()) };
        if dc.is_null() {
            return None;
        }
        let info = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: width,
                biHeight: -height, // negativo: origem no topo (top-down)
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB,
                ..Default::default()
            },
            ..Default::default()
        };
        let mut bits: *mut core::ffi::c_void = std::ptr::null_mut();
        // SAFETY: `info` descreve uma DIB válida; `bits` recebe o ponteiro dos pixels.
        let bitmap = unsafe {
            CreateDIBSection(
                dc,
                &info,
                DIB_RGB_COLORS,
                &mut bits,
                std::ptr::null_mut(),
                0,
            )
        };
        if bitmap.is_null() || bits.is_null() {
            // SAFETY: `dc` foi criado acima e não está em uso.
            unsafe { DeleteDC(dc) };
            return None;
        }
        // SAFETY: `dc` e `bitmap` são válidos; o objeto anterior é guardado para restaurar.
        let previous = unsafe { SelectObject(dc, bitmap) };
        Some(Self {
            dc,
            bitmap,
            previous,
            bits: bits.cast(),
            width,
            height,
        })
    }

    /// Largura.
    pub fn width(&self) -> i32 {
        self.width
    }

    /// Altura.
    pub fn height(&self) -> i32 {
        self.height
    }

    /// Copia os pixels (BGRA premultiplicado, linha a linha) e atualiza a janela em camadas.
    pub fn present(&mut self, hwnd: HWND, x: i32, y: i32, pixels: &[u32]) -> bool {
        let n = (self.width as usize) * (self.height as usize);
        if pixels.len() < n {
            return false;
        }
        // SAFETY: `bits` aponta para `width × height` pixels de 32 bits pertencentes à DIB section,
        // que vive enquanto `self` viver; `pixels` tem pelo menos `n` elementos; não há sobreposição.
        unsafe { std::ptr::copy_nonoverlapping(pixels.as_ptr(), self.bits, n) };
        // SAFETY: DC da tela; liberado logo abaixo.
        let screen = unsafe { GetDC(std::ptr::null_mut()) };
        let dst = POINT { x, y };
        let size = SIZE {
            cx: self.width,
            cy: self.height,
        };
        let src = POINT { x: 0, y: 0 };
        let blend = BLENDFUNCTION {
            BlendOp: AC_SRC_OVER as u8,
            BlendFlags: 0,
            SourceConstantAlpha: 255,
            AlphaFormat: AC_SRC_ALPHA as u8,
        };
        // SAFETY: todos os ponteiros apontam para variáveis locais válidas; `self.dc` tem a DIB
        // selecionada; `hwnd` é uma janela WS_EX_LAYERED desta thread.
        let ok = unsafe {
            UpdateLayeredWindow(
                hwnd, screen, &dst, &size, self.dc, &src, 0, &blend, ULW_ALPHA,
            )
        };
        if !screen.is_null() {
            // SAFETY: `screen` veio de `GetDC(NULL)`.
            unsafe { ReleaseDC(std::ptr::null_mut(), screen) };
        }
        ok != 0
    }
}

impl Drop for LayeredBuffer {
    fn drop(&mut self) {
        // SAFETY: restaura o objeto original antes de apagar o bitmap (um objeto ainda selecionado
        // não pode ser apagado e vazaria).
        unsafe { SelectObject(self.dc, self.previous) };
        // SAFETY: `bitmap` foi criado por `CreateDIBSection` e não está mais selecionado.
        unsafe { DeleteObject(self.bitmap) };
        // SAFETY: `dc` foi criado por `CreateCompatibleDC`.
        unsafe { DeleteDC(self.dc) };
    }
}
