//! Arte original do jogo (sprites, ícones e fonte), definida como mapas de caracteres.
//!
//! Cada imagem é uma grade de texto em que cada caractere indica uma entrada da paleta. Isso evita
//! dependência de decodificador de imagem, mantém o executável pequeno e deixa a arte revisável em
//! diff. Licença da arte: CC-BY-4.0 (ver `assets-src/LICENSE`).
#![forbid(unsafe_code)]

mod art;
mod font;

pub use art::{Frame, Icon, fighter_frame, icon};
pub use font::{GLYPH_H, GLYPH_W, glyph};

/// Versão do crate.
pub const CRATE_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Papel de cada pixel de um sprite de lutador. A cor final é escolhida pelo renderizador, o que
/// permite trocar a cor do time sem duplicar a arte.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ink {
    /// Transparente.
    None,
    /// Contorno.
    Outline,
    /// Pele.
    Skin,
    /// Pele em sombra.
    SkinShade,
    /// Metal claro.
    Metal,
    /// Metal escuro.
    MetalShade,
    /// Cor do time.
    Team,
    /// Cor do time em sombra.
    TeamShade,
    /// Couro/madeira.
    Leather,
    /// Cinto.
    Belt,
}

impl Ink {
    /// Converte um caractere do mapa em papel. Caracteres desconhecidos são transparentes.
    pub const fn from_char(c: u8) -> Ink {
        match c {
            b'k' => Ink::Outline,
            b's' => Ink::Skin,
            b'S' => Ink::SkinShade,
            b'm' => Ink::Metal,
            b'M' => Ink::MetalShade,
            b't' => Ink::Team,
            b'T' => Ink::TeamShade,
            b'w' => Ink::Leather,
            b'b' => Ink::Belt,
            _ => Ink::None,
        }
    }
}

/// Imagem como grade de caracteres de largura fixa.
#[derive(Debug, Clone, Copy)]
pub struct CharMap {
    /// Linhas (todas com o mesmo comprimento).
    pub rows: &'static [&'static str],
}

impl CharMap {
    /// Largura em pixels.
    pub fn width(&self) -> usize {
        self.rows.first().map_or(0, |r| r.len())
    }

    /// Altura em pixels.
    pub fn height(&self) -> usize {
        self.rows.len()
    }

    /// Papel do pixel `(x, y)`.
    pub fn ink(&self, x: usize, y: usize) -> Ink {
        self.rows
            .get(y)
            .and_then(|r| r.as_bytes().get(x))
            .map_or(Ink::None, |&c| Ink::from_char(c))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_fighter_frames_are_rectangular() {
        for f in [Frame::Idle, Frame::Attack, Frame::Defend, Frame::Down] {
            let m = fighter_frame(f);
            let w = m.width();
            assert!(w > 0);
            for (i, r) in m.rows.iter().enumerate() {
                assert_eq!(r.len(), w, "frame {f:?} linha {i}");
            }
        }
    }

    #[test]
    fn all_icons_are_rectangular() {
        for i in Icon::ALL {
            let m = icon(i);
            let w = m.width();
            for r in m.rows {
                assert_eq!(r.len(), w, "{i:?}");
            }
        }
    }
}
