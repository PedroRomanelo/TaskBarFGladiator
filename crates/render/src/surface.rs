//! Superfície de desenho em memória (BGRA de 32 bits com alpha premultiplicado).
//!
//! Cada pixel é um `u32` no formato `0xAARRGGBB`. Em memória little-endian, a ordem dos bytes é
//! B, G, R, A, que é exatamente o formato que `UpdateLayeredWindow` espera numa DIB de 32 bits.

/// Cor com alpha não premultiplicado (`0xAARRGGBB`), usada na API de desenho.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rgba(pub u32);

impl Rgba {
    /// Totalmente transparente.
    pub const TRANSPARENT: Rgba = Rgba(0);

    /// Cria uma cor opaca a partir de `0xRRGGBB`.
    pub const fn rgb(rgb: u32) -> Rgba {
        Rgba(0xff00_0000 | (rgb & 0x00ff_ffff))
    }

    /// Mesma cor com outro alpha.
    pub const fn with_alpha(self, a: u8) -> Rgba {
        Rgba(((a as u32) << 24) | (self.0 & 0x00ff_ffff))
    }

    /// Alpha.
    pub const fn alpha(self) -> u32 {
        self.0 >> 24
    }

    /// Converte para premultiplicado.
    pub fn premultiplied(self) -> u32 {
        let a = self.alpha();
        if a == 0 {
            return 0;
        }
        if a == 255 {
            return self.0;
        }
        let r = ((self.0 >> 16) & 0xff) * a / 255;
        let g = ((self.0 >> 8) & 0xff) * a / 255;
        let b = (self.0 & 0xff) * a / 255;
        (a << 24) | (r << 16) | (g << 8) | b
    }
}

/// Mistura `src` (premultiplicado) sobre `dst` (premultiplicado): `src + dst × (1 − αsrc)`.
#[inline]
pub fn blend_premul(dst: u32, src: u32) -> u32 {
    let sa = src >> 24;
    if sa == 255 {
        return src;
    }
    if sa == 0 {
        return dst;
    }
    let inv = 255 - sa;
    let ch = |shift: u32| -> u32 {
        let s = (src >> shift) & 0xff;
        let d = (dst >> shift) & 0xff;
        (s + (d * inv + 127) / 255).min(255)
    };
    (ch(24) << 24) | (ch(16) << 16) | (ch(8) << 8) | ch(0)
}

/// Superfície de desenho.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Surface {
    width: usize,
    height: usize,
    pixels: Vec<u32>,
}

impl Surface {
    /// Cria uma superfície transparente.
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            pixels: vec![0; width * height],
        }
    }

    /// Largura.
    pub fn width(&self) -> usize {
        self.width
    }

    /// Altura.
    pub fn height(&self) -> usize {
        self.height
    }

    /// Pixels (linha a linha, de cima para baixo).
    pub fn pixels(&self) -> &[u32] {
        &self.pixels
    }

    /// Redimensiona sem realocar quando a capacidade é suficiente. O conteúdo fica transparente.
    pub fn resize(&mut self, width: usize, height: usize) {
        self.width = width;
        self.height = height;
        self.pixels.clear();
        self.pixels.resize(width * height, 0);
    }

    /// Preenche tudo com uma cor.
    pub fn clear(&mut self, color: Rgba) {
        let p = color.premultiplied();
        self.pixels.fill(p);
    }

    /// Pixel em `(x, y)`, se estiver dentro.
    pub fn get(&self, x: i32, y: i32) -> Option<u32> {
        if x < 0 || y < 0 {
            return None;
        }
        let (x, y) = (x as usize, y as usize);
        if x >= self.width || y >= self.height {
            return None;
        }
        self.pixels.get(y * self.width + x).copied()
    }

    /// Mistura um pixel premultiplicado em `(x, y)`; ignora coordenadas fora da superfície.
    #[inline]
    pub fn blend_pixel(&mut self, x: i32, y: i32, premul: u32) {
        if x < 0 || y < 0 {
            return;
        }
        let (x, y) = (x as usize, y as usize);
        if x >= self.width || y >= self.height {
            return;
        }
        let i = y * self.width + x;
        if let Some(d) = self.pixels.get_mut(i) {
            *d = blend_premul(*d, premul);
        }
    }

    /// Preenche um retângulo (com recorte nas bordas).
    pub fn fill_rect(&mut self, x: i32, y: i32, w: i32, h: i32, color: Rgba) {
        let p = color.premultiplied();
        if p == 0 {
            return;
        }
        let x0 = x.max(0);
        let y0 = y.max(0);
        let x1 = (x + w).min(self.width as i32);
        let y1 = (y + h).min(self.height as i32);
        for yy in y0..y1 {
            for xx in x0..x1 {
                self.blend_pixel(xx, yy, p);
            }
        }
    }

    /// Contorno de 1 px de espessura × `t`.
    pub fn stroke_rect(&mut self, x: i32, y: i32, w: i32, h: i32, t: i32, color: Rgba) {
        self.fill_rect(x, y, w, t, color);
        self.fill_rect(x, y + h - t, w, t, color);
        self.fill_rect(x, y + t, t, h - 2 * t, color);
        self.fill_rect(x + w - t, y + t, t, h - 2 * t, color);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn premultiply_zero_alpha_is_zero() {
        assert_eq!(Rgba(0x00ff_ffff).premultiplied(), 0);
        assert_eq!(Rgba(0x80ff_0000).premultiplied(), 0x8080_0000);
    }

    #[test]
    fn opaque_over_anything_is_opaque() {
        assert_eq!(blend_premul(0x8012_3456, 0xff00_ff00), 0xff00_ff00);
    }

    #[test]
    fn transparent_over_keeps_dst() {
        assert_eq!(blend_premul(0xff12_3456, 0), 0xff12_3456);
    }

    #[test]
    fn fill_rect_clips() {
        let mut s = Surface::new(4, 4);
        s.fill_rect(-10, -10, 100, 100, Rgba::rgb(0x112233));
        assert!(s.pixels().iter().all(|&p| p == 0xff11_2233));
    }
}
