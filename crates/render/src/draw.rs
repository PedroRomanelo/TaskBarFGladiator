//! Primitivas: sprites de lutador, ícones e texto com contorno, com escala inteira.

use crate::surface::{Rgba, Surface};
use assets::{CharMap, Frame, GLYPH_H, GLYPH_W, Icon, Ink};

/// Cores de um time.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TeamColors {
    /// Cor principal.
    pub main: Rgba,
    /// Sombra.
    pub shade: Rgba,
}

/// Paleta do sprite.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpritePalette {
    /// Contorno.
    pub outline: Rgba,
    /// Pele.
    pub skin: Rgba,
    /// Pele em sombra.
    pub skin_shade: Rgba,
    /// Metal.
    pub metal: Rgba,
    /// Metal escuro.
    pub metal_shade: Rgba,
    /// Couro.
    pub leather: Rgba,
    /// Cinto.
    pub belt: Rgba,
    /// Time.
    pub team: TeamColors,
}

impl SpritePalette {
    fn color(&self, ink: Ink) -> Option<Rgba> {
        Some(match ink {
            Ink::None => return None,
            Ink::Outline => self.outline,
            Ink::Skin => self.skin,
            Ink::SkinShade => self.skin_shade,
            Ink::Metal => self.metal,
            Ink::MetalShade => self.metal_shade,
            Ink::Team => self.team.main,
            Ink::TeamShade => self.team.shade,
            Ink::Leather => self.leather,
            Ink::Belt => self.belt,
        })
    }
}

/// Desenha um quadro de lutador com o canto superior esquerdo em `(x, y)` (pixels físicos).
///
/// `mirror` inverte horizontalmente (oponente voltado para a esquerda). `flash` desenha todos os
/// pixels opacos em branco (indicação de golpe recebido).
#[allow(clippy::too_many_arguments)]
pub fn fighter(
    s: &mut Surface,
    x: i32,
    y: i32,
    scale: i32,
    frame: Frame,
    palette: &SpritePalette,
    mirror: bool,
    flash: bool,
) {
    let map = assets::fighter_frame(frame);
    blit_map(s, x, y, scale, &map, mirror, |ink| {
        let c = palette.color(ink)?;
        Some(if flash && ink != Ink::Outline {
            Rgba::rgb(0xffffff)
        } else {
            c
        })
    });
}

/// Largura e altura (em pixels lógicos) dos quadros do lutador.
pub fn fighter_size() -> (i32, i32) {
    let m = assets::fighter_frame(Frame::Idle);
    (m.width() as i32, m.height() as i32)
}

fn blit_map(
    s: &mut Surface,
    x: i32,
    y: i32,
    scale: i32,
    map: &CharMap,
    mirror: bool,
    color: impl Fn(Ink) -> Option<Rgba>,
) {
    let w = map.width();
    for my in 0..map.height() {
        for mx in 0..w {
            let ink = map.ink(mx, my);
            let Some(c) = color(ink) else { continue };
            let sx = if mirror { w - 1 - mx } else { mx } as i32;
            s.fill_rect(x + sx * scale, y + my as i32 * scale, scale, scale, c);
        }
    }
}

/// Desenha um ícone 7 × 7 (`#` = cor).
pub fn icon(s: &mut Surface, x: i32, y: i32, scale: i32, which: Icon, color: Rgba) {
    let map = assets::icon(which);
    for (my, row) in map.rows.iter().enumerate() {
        for (mx, ch) in row.bytes().enumerate() {
            if ch == b'#' {
                s.fill_rect(
                    x + mx as i32 * scale,
                    y + my as i32 * scale,
                    scale,
                    scale,
                    color,
                );
            }
        }
    }
}

/// Avanço horizontal de um caractere (glifo + 1 de espaço), em pixels lógicos.
pub const CHAR_ADVANCE: i32 = GLYPH_W as i32 + 1;
/// Altura de uma linha de texto (glifo + contorno), em pixels lógicos.
pub const LINE_HEIGHT: i32 = GLYPH_H as i32 + 2;

/// Largura de um texto em pixels lógicos.
pub fn text_width(text: &str) -> i32 {
    let n = text.chars().count() as i32;
    if n == 0 { 0 } else { n * CHAR_ADVANCE - 1 }
}

/// Desenha texto. Com `outline`, desenha antes um contorno de 1 px lógico em volta de cada pixel
/// aceso (legível sobre fundo claro ou escuro). Retorna a largura desenhada em pixels físicos.
pub fn text(
    s: &mut Surface,
    x: i32,
    y: i32,
    scale: i32,
    content: &str,
    color: Rgba,
    outline: Option<Rgba>,
) -> i32 {
    text_max(s, x, y, scale, content, usize::MAX, color, outline)
}

/// Como [`text`], desenhando no máximo `max_chars` caracteres (sem alocar).
#[allow(clippy::too_many_arguments)]
pub fn text_max(
    s: &mut Surface,
    x: i32,
    y: i32,
    scale: i32,
    content: &str,
    max_chars: usize,
    color: Rgba,
    outline: Option<Rgba>,
) -> i32 {
    if let Some(o) = outline {
        for (dx, dy) in [
            (-1, 0),
            (1, 0),
            (0, -1),
            (0, 1),
            (-1, -1),
            (1, -1),
            (-1, 1),
            (1, 1),
        ] {
            draw_glyphs(
                s,
                x + dx * scale,
                y + dy * scale,
                scale,
                content,
                max_chars,
                o,
            );
        }
    }
    draw_glyphs(s, x, y, scale, content, max_chars, color);
    let n = content.chars().count().min(max_chars) as i32;
    (if n == 0 { 0 } else { n * CHAR_ADVANCE - 1 }) * scale
}

fn draw_glyphs(
    s: &mut Surface,
    x: i32,
    y: i32,
    scale: i32,
    content: &str,
    max_chars: usize,
    color: Rgba,
) {
    let mut cx = x;
    for ch in content.chars().take(max_chars) {
        let g = assets::glyph(ch);
        for (gy, row) in g.iter().enumerate() {
            for (gx, b) in row.bytes().enumerate() {
                if b == b'#' {
                    s.fill_rect(
                        cx + gx as i32 * scale,
                        y + gy as i32 * scale,
                        scale,
                        scale,
                        color,
                    );
                }
            }
        }
        cx += CHAR_ADVANCE * scale;
    }
}

/// Corta o texto para caber em `max_chars` caracteres, terminando com `.` quando cortado.
pub fn ellipsize(content: &str, max_chars: usize) -> String {
    let n = content.chars().count();
    if n <= max_chars {
        return content.to_owned();
    }
    if max_chars == 0 {
        return String::new();
    }
    let mut out: String = content.chars().take(max_chars - 1).collect();
    out.push('.');
    out
}
