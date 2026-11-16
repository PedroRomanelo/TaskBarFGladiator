//! Arte original do jogo (sprites, ícones e fonte), definida como mapas de caracteres.
//!
//! Cada imagem é uma grade de texto em que cada caractere indica uma entrada da paleta. Isso evita
//! dependência de decodificador de imagem, mantém o executável pequeno e deixa a arte revisável em
//! diff. Licença da arte: CC-BY-4.0 (ver `assets-src/LICENSE`).
//!
//! O lutador é um boneco em camadas (ver [`parts`]): o corpo é montado com peças por pose e recebe
//! por cima a aparência escolhida pelo jogador ([`Look`]: pele, olhos, cabelo, barba) e o
//! equipamento fixo (arma, escudo e peças da armadura).
#![forbid(unsafe_code)]

mod art;
mod font;
pub mod parts;

pub use art::{Icon, icon};
pub use font::{GLYPH_H, GLYPH_W, glyph};
pub use parts::{CANVAS, CENTER_X, Piece, Rig, Turn, rig};

/// Versão do crate.
pub const CRATE_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Quadro de animação do lutador.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Frame {
    /// Parado, em guarda.
    Idle,
    /// Golpe com a arma estendida.
    Attack,
    /// Escudo (ou braço) erguido.
    Defend,
    /// Caído (nocaute).
    Down,
}

impl Frame {
    /// Todos os quadros.
    pub const ALL: [Frame; 4] = [Frame::Idle, Frame::Attack, Frame::Defend, Frame::Down];
}

/// Papel de cada pixel de um sprite. A cor final é escolhida pelo renderizador a partir da
/// aparência do lutador, do time e do tema, o que permite trocar pele, cabelo e cores sem duplicar
/// a arte.
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
    /// Cabelo e barba.
    Hair,
    /// Cabelo e barba em sombra.
    HairShade,
    /// Branco do olho.
    EyeWhite,
    /// Íris do olho.
    Iris,
    /// Boca.
    Mouth,
    /// Cor do time (saiote, crista do elmo, faixas do escudo).
    Team,
    /// Cor do time em sombra.
    TeamShade,
    /// Aço.
    Metal,
    /// Aço em sombra.
    MetalShade,
    /// Brilho de metal.
    Shine,
    /// Bronze.
    Bronze,
    /// Bronze em sombra.
    BronzeShade,
    /// Madeira.
    Wood,
    /// Madeira em sombra.
    WoodShade,
    /// Couro.
    Leather,
    /// Couro em sombra.
    LeatherShade,
    /// Tecido cru (túnica, corda do arco).
    Cloth,
    /// Tecido cru em sombra.
    ClothShade,
    /// Canela: pele, ou greva quando a armadura tem grevas.
    Greave,
    /// Canela em sombra.
    GreaveShade,
}

impl Ink {
    /// Converte um caractere do mapa em papel. Caracteres desconhecidos são transparentes.
    ///
    /// | Letra | Papel | Letra | Papel |
    /// |---|---|---|---|
    /// | `k` | contorno | `.` | transparente |
    /// | `s` / `S` | pele / sombra | `h` / `H` | cabelo / sombra |
    /// | `e` / `i` | branco do olho / íris | `n` | boca |
    /// | `t` / `T` | cor do time / sombra | `c` / `C` | tecido cru / sombra |
    /// | `m` / `M` | aço / sombra | `l` | brilho de metal |
    /// | `o` / `O` | bronze / sombra | `w` / `W` | madeira / sombra |
    /// | `b` / `B` | couro / sombra | `g` / `G` | canela (pele ou greva) / sombra |
    pub const fn from_char(c: u8) -> Ink {
        match c {
            b'k' => Ink::Outline,
            b's' => Ink::Skin,
            b'S' => Ink::SkinShade,
            b'h' => Ink::Hair,
            b'H' => Ink::HairShade,
            b'e' => Ink::EyeWhite,
            b'i' => Ink::Iris,
            b'n' => Ink::Mouth,
            b't' => Ink::Team,
            b'T' => Ink::TeamShade,
            b'm' => Ink::Metal,
            b'M' => Ink::MetalShade,
            b'l' => Ink::Shine,
            b'o' => Ink::Bronze,
            b'O' => Ink::BronzeShade,
            b'w' => Ink::Wood,
            b'W' => Ink::WoodShade,
            b'b' => Ink::Leather,
            b'B' => Ink::LeatherShade,
            b'c' => Ink::Cloth,
            b'C' => Ink::ClothShade,
            b'g' => Ink::Greave,
            b'G' => Ink::GreaveShade,
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

/// Tons de pele: `(cor, sombra)` em `0xRRGGBB`.
pub const SKIN_TONES: [(u32, u32); 6] = [
    (0xf2d3b3, 0xd3a583),
    (0xe3b78d, 0xbd8a62),
    (0xcf9a6b, 0xa5714a),
    (0xb07a4e, 0x875a36),
    (0x8a5a38, 0x653f25),
    (0x5e3d28, 0x432a1a),
];

/// Cores de cabelo e barba: `(cor, sombra)` em `0xRRGGBB`.
pub const HAIR_COLORS: [(u32, u32); 7] = [
    (0x2b2420, 0x15110f),
    (0x5a3a22, 0x3b2515),
    (0x8a5a2c, 0x60401e),
    (0xd9b25a, 0xa98538),
    (0xb5452a, 0x82301c),
    (0x9a9a9e, 0x6c6c72),
    (0xe9e6dc, 0xb9b5a8),
];

/// Cores de olhos em `0xRRGGBB`.
pub const EYE_COLORS: [u32; 5] = [0x4a2f1b, 0x2f6fb5, 0x3f8f4a, 0x6f7f8c, 0xb5802a];

/// Corte de cabelo.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Hair {
    /// Careca.
    Bald,
    /// Curto.
    #[default]
    Short,
    /// Comprido.
    Long,
    /// Moicano.
    Mohawk,
    /// Coque.
    Topknot,
    /// Cacheado.
    Curly,
}

impl Hair {
    /// Todos os cortes, na ordem dos índices salvos.
    pub const ALL: [Hair; 6] = [
        Hair::Bald,
        Hair::Short,
        Hair::Long,
        Hair::Mohawk,
        Hair::Topknot,
        Hair::Curly,
    ];

    /// Corte de um índice salvo (índices fora da lista dão a volta).
    pub fn from_index(i: u8) -> Hair {
        Self::ALL[usize::from(i) % Self::ALL.len()]
    }
}

/// Barba.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Beard {
    /// Sem barba.
    #[default]
    None,
    /// Rala.
    Stubble,
    /// Bigode.
    Mustache,
    /// Cheia.
    Full,
    /// Longa.
    Long,
}

impl Beard {
    /// Todas as barbas, na ordem dos índices salvos.
    pub const ALL: [Beard; 5] = [
        Beard::None,
        Beard::Stubble,
        Beard::Mustache,
        Beard::Full,
        Beard::Long,
    ];

    /// Barba de um índice salvo (índices fora da lista dão a volta).
    pub fn from_index(i: u8) -> Beard {
        Self::ALL[usize::from(i) % Self::ALL.len()]
    }
}

/// Desenho de arma (fixo: igual para qualquer lutador).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Weapon {
    /// Mãos vazias.
    #[default]
    None,
    /// Faca.
    Knife,
    /// Espada curta.
    Shortsword,
    /// Lança.
    Spear,
    /// Machado.
    Axe,
    /// Arco.
    Bow,
    /// Bastão.
    Staff,
}

impl Weapon {
    /// Todas as armas.
    pub const ALL: [Weapon; 7] = [
        Weapon::None,
        Weapon::Knife,
        Weapon::Shortsword,
        Weapon::Spear,
        Weapon::Axe,
        Weapon::Bow,
        Weapon::Staff,
    ];
}

/// Conjunto de armadura (fixo). Cada conjunto define as peças visíveis: peitoral, elmo e grevas.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Armor {
    /// Sem armadura.
    #[default]
    None,
    /// Túnica acolchoada: só peitoral de tecido.
    Tunic,
    /// Couro fervido: peitoral, gorro e grevas de couro.
    Leather,
    /// Malha de anéis: cota, elmo de ferro e grevas de aço.
    Mail,
    /// Placas de bronze: couraça, elmo com crista e grevas de bronze.
    Plates,
}

impl Armor {
    /// Todas as armaduras.
    pub const ALL: [Armor; 5] = [
        Armor::None,
        Armor::Tunic,
        Armor::Leather,
        Armor::Mail,
        Armor::Plates,
    ];
}

/// Desenho de escudo (fixo).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Shield {
    /// Sem escudo.
    #[default]
    None,
    /// Broquel de madeira.
    Buckler,
    /// Escudo redondo.
    Round,
    /// Escudo oval reforçado.
    Oval,
    /// Escudo de torre.
    Tower,
}

impl Shield {
    /// Todos os escudos.
    pub const ALL: [Shield; 5] = [
        Shield::None,
        Shield::Buckler,
        Shield::Round,
        Shield::Oval,
        Shield::Tower,
    ];
}

/// Tudo o que define o desenho de um lutador: aparência (personalizável) e equipamento (fixo).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Look {
    /// Índice em [`SKIN_TONES`].
    pub skin: u8,
    /// Corte de cabelo.
    pub hair: Hair,
    /// Índice em [`HAIR_COLORS`].
    pub hair_color: u8,
    /// Índice em [`EYE_COLORS`].
    pub eyes: u8,
    /// Barba.
    pub beard: Beard,
    /// Arma.
    pub weapon: Weapon,
    /// Armadura.
    pub armor: Armor,
    /// Escudo.
    pub shield: Shield,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_parts_are_rectangular_and_use_known_letters() {
        for (name, m) in parts::all_parts() {
            let w = m.width();
            assert!(w > 0, "{name}");
            for (y, r) in m.rows.iter().enumerate() {
                assert_eq!(r.len(), w, "peça {name} linha {y}");
                for (x, c) in r.bytes().enumerate() {
                    assert!(
                        c == b'.' || Ink::from_char(c) != Ink::None,
                        "peça {name}: letra {:?} desconhecida em ({x}, {y})",
                        c as char
                    );
                }
            }
        }
    }

    #[test]
    fn every_rig_fits_the_canvas() {
        // Nenhuma peça pode sair da tela do lutador, em nenhuma pose e com nenhum equipamento.
        for frame in Frame::ALL {
            for weapon in Weapon::ALL {
                for shield in Shield::ALL {
                    for armor in Armor::ALL {
                        for hair in Hair::ALL {
                            let look = Look {
                                hair,
                                beard: Beard::Long,
                                weapon,
                                armor,
                                shield,
                                ..Look::default()
                            };
                            for p in rig(frame, &look).pieces.into_iter().flatten() {
                                let (w, h) = match p.turn {
                                    Turn::Up => (p.map.width(), p.map.height()),
                                    Turn::Forward => (p.map.height(), p.map.width()),
                                };
                                assert!(
                                    p.x >= 0
                                        && p.y >= 0
                                        && p.x + w as i32 <= CANVAS
                                        && p.y + h as i32 <= CANVAS,
                                    "{frame:?} {look:?}: peça fora da tela em ({}, {}) {w}x{h}",
                                    p.x,
                                    p.y
                                );
                            }
                        }
                    }
                }
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

    #[test]
    fn saved_indices_wrap_around() {
        assert_eq!(Hair::from_index(0), Hair::Bald);
        assert_eq!(Hair::from_index(Hair::ALL.len() as u8), Hair::Bald);
        assert_eq!(Beard::from_index(200), Beard::ALL[200 % Beard::ALL.len()]);
    }
}
