//! Primitivas: lutador em camadas, ícones, discos e texto com contorno, com escala inteira.

use crate::surface::{Rgba, Surface};
use assets::{Armor, CANVAS, Frame, GLYPH_H, GLYPH_W, Icon, Ink, Look, Turn};

/// Cores de um time.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TeamColors {
    /// Cor principal.
    pub main: Rgba,
    /// Sombra.
    pub shade: Rgba,
}

/// Paleta de um lutador: cores da aparência (pele, cabelo, olhos), do time e dos materiais.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpritePalette {
    skin: (Rgba, Rgba),
    hair: (Rgba, Rgba),
    iris: Rgba,
    team: TeamColors,
    greave: (Rgba, Rgba),
}

const OUTLINE: Rgba = Rgba::rgb(0x1a1412);
const EYE_WHITE: Rgba = Rgba::rgb(0xf6f2ea);
const MOUTH: Rgba = Rgba::rgb(0x7a3b2e);
const METAL: (Rgba, Rgba) = (Rgba::rgb(0xb9bec8), Rgba::rgb(0x737a88));
const SHINE: Rgba = Rgba::rgb(0xf4f6fa);
const BRONZE: (Rgba, Rgba) = (Rgba::rgb(0xd9a441), Rgba::rgb(0x9a6a22));
const WOOD: (Rgba, Rgba) = (Rgba::rgb(0x9a6b3a), Rgba::rgb(0x6b4623));
const LEATHER: (Rgba, Rgba) = (Rgba::rgb(0x7a4a26), Rgba::rgb(0x4f2e16));
const CLOTH: (Rgba, Rgba) = (Rgba::rgb(0xe6dcc2), Rgba::rgb(0xb5a888));

fn pair((main, shade): (u32, u32)) -> (Rgba, Rgba) {
    (Rgba::rgb(main), Rgba::rgb(shade))
}

impl SpritePalette {
    /// Paleta para uma aparência e um time. Índices de cor fora das tabelas dão a volta.
    pub fn new(look: &Look, team: TeamColors) -> Self {
        let skin = pair(assets::SKIN_TONES[usize::from(look.skin) % assets::SKIN_TONES.len()]);
        let hair =
            pair(assets::HAIR_COLORS[usize::from(look.hair_color) % assets::HAIR_COLORS.len()]);
        let iris = Rgba::rgb(assets::EYE_COLORS[usize::from(look.eyes) % assets::EYE_COLORS.len()]);
        let greave = match look.armor {
            Armor::None | Armor::Tunic => skin,
            Armor::Leather => LEATHER,
            Armor::Mail => METAL,
            Armor::Plates => BRONZE,
        };
        Self {
            skin,
            hair,
            iris,
            team,
            greave,
        }
    }

    fn color(&self, ink: Ink) -> Option<Rgba> {
        Some(match ink {
            Ink::None => return None,
            Ink::Outline => OUTLINE,
            Ink::Skin => self.skin.0,
            Ink::SkinShade => self.skin.1,
            Ink::Hair => self.hair.0,
            Ink::HairShade => self.hair.1,
            Ink::EyeWhite => EYE_WHITE,
            Ink::Iris => self.iris,
            Ink::Mouth => MOUTH,
            Ink::Team => self.team.main,
            Ink::TeamShade => self.team.shade,
            Ink::Metal => METAL.0,
            Ink::MetalShade => METAL.1,
            Ink::Shine => SHINE,
            Ink::Bronze => BRONZE.0,
            Ink::BronzeShade => BRONZE.1,
            Ink::Wood => WOOD.0,
            Ink::WoodShade => WOOD.1,
            Ink::Leather => LEATHER.0,
            Ink::LeatherShade => LEATHER.1,
            Ink::Cloth => CLOTH.0,
            Ink::ClothShade => CLOTH.1,
            Ink::Greave => self.greave.0,
            Ink::GreaveShade => self.greave.1,
        })
    }
}

/// Quantas linhas o lutador caído desce para encostar as costas no chão.
const LYING_DROP: i32 = 6;

/// Desenha um quadro de lutador com o canto superior esquerdo da tela de [`CANVAS`] ×
/// [`CANVAS`] em `(x, y)` (pixels físicos).
///
/// As peças vêm de [`assets::rig`], na ordem de desenho. `mirror` inverte horizontalmente
/// (oponente voltado para a esquerda). `flash` desenha todos os pixels opacos em branco (indicação
/// de golpe recebido).
#[allow(clippy::too_many_arguments)]
pub fn fighter(
    s: &mut Surface,
    x: i32,
    y: i32,
    scale: i32,
    frame: Frame,
    look: &Look,
    palette: &SpritePalette,
    mirror: bool,
    flash: bool,
) {
    let rig = assets::rig(frame, look);
    for piece in rig.pieces.iter().flatten() {
        let (w, h) = (piece.map.width() as i32, piece.map.height() as i32);
        for my in 0..h {
            for mx in 0..w {
                let ink = piece.map.ink(mx as usize, my as usize);
                let Some(color) = palette.color(ink) else {
                    continue;
                };
                // Posição na tela do lutador, com a rotação da peça.
                let (mut cx, mut cy) = match piece.turn {
                    Turn::Up => (piece.x + mx, piece.y + my),
                    Turn::Forward => (piece.x + (h - 1 - my), piece.y + mx),
                };
                if rig.lying {
                    // Tela inteira girada 90° no sentido anti-horário: cabeça para trás.
                    (cx, cy) = (cy, CANVAS - 1 - cx + LYING_DROP);
                }
                if cx < 0 || cy < 0 || cx >= CANVAS || cy >= CANVAS {
                    continue;
                }
                if mirror {
                    cx = CANVAS - 1 - cx;
                }
                let c = if flash && ink != Ink::Outline {
                    Rgba::rgb(0xffffff)
                } else {
                    color
                };
                s.fill_rect(x + cx * scale, y + cy * scale, scale, scale, c);
            }
        }
    }
}

/// Disco de diâmetro `d` (pixels lógicos) com o canto superior esquerdo em `(x, y)` (pixels
/// físicos), desenhado na grade lógica para combinar com a pixel art.
pub fn disc(s: &mut Surface, x: i32, y: i32, d: i32, scale: i32, color: Rgba) {
    for j in 0..d {
        for i in 0..d {
            let (dx, dy) = (2 * i + 1 - d, 2 * j + 1 - d);
            if dx * dx + dy * dy <= d * d {
                s.fill_rect(x + i * scale, y + j * scale, scale, scale, color);
            }
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
