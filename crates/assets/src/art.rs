//! Sprites do lutador (16 × 20, voltado para a direita) e ícones das ações (7 × 7).
//!
//! Legenda dos sprites: `k` contorno, `s`/`S` pele, `m`/`M` metal, `t`/`T` cor do time,
//! `w` couro/madeira, `b` cinto, `.` transparente. Ícones: `#` desenho, `.` transparente.

use crate::CharMap;

/// Quadro de animação do lutador.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Frame {
    /// Parado, espada erguida e escudo à frente.
    Idle,
    /// Golpe com a espada estendida.
    Attack,
    /// Escudo erguido.
    Defend,
    /// Caído (nocaute).
    Down,
}

const IDLE: &[&str] = &[
    ".k..kkkk........",
    ".mk.kMmmk.......",
    ".mk.kmmmMk......",
    ".mk.kssksk......",
    ".mk.ksssSk......",
    ".mk..kSSk.......",
    ".kskkttttkk.....",
    "..ksttttTkwk....",
    "...kttttTkwk....",
    "...kbbbbbkwk....",
    "...kttttTkwk....",
    "...kttttTkwk....",
    "...ktt.tTk.k....",
    "...kSs.sSk......",
    "...kss.ssk......",
    "...kss.ssk......",
    "...kSs.sSk......",
    "...kww.wwk......",
    "..kwwk.kwwk.....",
    "..kkk...kkk.....",
];

const ATTACK: &[&str] = &[
    "....kkkk........",
    "...kMmmmk.......",
    "...kmmmmMk......",
    "...kssksk.......",
    "...ksssSk.......",
    "....kSSk........",
    "..kkttttkk......",
    ".kwttttTsskkkkk.",
    ".kwkttttkkmmmmmk",
    ".kwkbbbbbk.kkkk.",
    ".kwkttttTk......",
    "..kkttttTk......",
    "...ktt.tTk......",
    "...kSs.sSk......",
    "...kss..ssk.....",
    "..kss....ssk....",
    "..kSs....sSk....",
    "..kww....wwk....",
    ".kwwk....kwwk...",
    ".kkk......kkk...",
];

const DEFEND: &[&str] = &[
    "................",
    "....kkkk........",
    "...kMmmmk.kk....",
    "...kmmmmMkwwk...",
    "...kssksskwwk...",
    "...ksssSkwwwk...",
    "..kkkSStkwwwk...",
    ".kmkttttkwwwk...",
    "km.kttttTkwwk...",
    "km.kbbbbbkwwk...",
    "k..kttttTkkk....",
    "...kttttTk......",
    "...ktt.tTk......",
    "..kSs..sSk......",
    "..kss...ssk.....",
    ".kss.....ssk....",
    ".kSs.....sSk....",
    ".kww.....wwk....",
    "kwwk.....kwwk...",
    "kkk.......kkk...",
];

const DOWN: &[&str] = &[
    "................",
    "................",
    "................",
    "................",
    "................",
    "................",
    "................",
    "................",
    "................",
    "................",
    "................",
    "................",
    "................",
    "................",
    "................",
    "..kkkk..........",
    ".kMmmk.kkkkkkkk.",
    ".kssskkttttttwwk",
    ".kkSSkktbbtttkkk",
    "..kk..kkkkkkkkk.",
];

/// Mapa de um quadro do lutador.
pub fn fighter_frame(frame: Frame) -> CharMap {
    CharMap {
        rows: match frame {
            Frame::Idle => IDLE,
            Frame::Attack => ATTACK,
            Frame::Defend => DEFEND,
            Frame::Down => DOWN,
        },
    }
}

/// Ícone de ação.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Icon {
    /// Avançar.
    Advance,
    /// Recuar.
    Retreat,
    /// Ataque leve.
    Light,
    /// Ataque médio.
    Medium,
    /// Ataque pesado.
    Heavy,
    /// Defender.
    Defend,
    /// Recuperar.
    Recover,
    /// Provocar.
    Taunt,
    /// Pausar.
    Pause,
    /// Continuar.
    Play,
    /// Painel/menu.
    Menu,
}

impl Icon {
    /// Todos os ícones.
    pub const ALL: [Icon; 11] = [
        Icon::Advance,
        Icon::Retreat,
        Icon::Light,
        Icon::Medium,
        Icon::Heavy,
        Icon::Defend,
        Icon::Recover,
        Icon::Taunt,
        Icon::Pause,
        Icon::Play,
        Icon::Menu,
    ];
}

const ICON_ADVANCE: &[&str] = &[
    "...#...", "...##..", "#######", "#######", "...##..", "...#...", ".......",
];
const ICON_RETREAT: &[&str] = &[
    "...#...", "..##...", "#######", "#######", "..##...", "...#...", ".......",
];
const ICON_LIGHT: &[&str] = &[
    "......#", ".....#.", "....#..", "...#...", ".##....", ".#.....", "#......",
];
const ICON_MEDIUM: &[&str] = &[
    ".....##", "....###", "...###.", "#.###..", ".###...", ".##....", "#.#....",
];
const ICON_HEAVY: &[&str] = &[
    "..####.", ".######", "..####.", "...#...", "...#...", "...#...", "...#...",
];
const ICON_DEFEND: &[&str] = &[
    "#######", "#.....#", "#.###.#", "#.###.#", ".#...#.", "..#.#..", "...#...",
];
const ICON_RECOVER: &[&str] = &[
    "..###..", "..###..", "#######", "#######", "#######", "..###..", "..###..",
];
const ICON_TAUNT: &[&str] = &[
    "...#...", "..###..", "..###..", "..###..", "...#...", ".......", "...#...",
];
const ICON_PAUSE: &[&str] = &[
    ".##.##.", ".##.##.", ".##.##.", ".##.##.", ".##.##.", ".##.##.", ".......",
];
const ICON_PLAY: &[&str] = &[
    ".#.....", ".##....", ".###...", ".####..", ".###...", ".##....", ".#.....",
];
const ICON_MENU: &[&str] = &[
    ".......", "#######", ".......", "#######", ".......", "#######", ".......",
];

/// Mapa de um ícone (7 × 7, `#` = desenho).
pub fn icon(i: Icon) -> CharMap {
    CharMap {
        rows: match i {
            Icon::Advance => ICON_ADVANCE,
            Icon::Retreat => ICON_RETREAT,
            Icon::Light => ICON_LIGHT,
            Icon::Medium => ICON_MEDIUM,
            Icon::Heavy => ICON_HEAVY,
            Icon::Defend => ICON_DEFEND,
            Icon::Recover => ICON_RECOVER,
            Icon::Taunt => ICON_TAUNT,
            Icon::Pause => ICON_PAUSE,
            Icon::Play => ICON_PLAY,
            Icon::Menu => ICON_MENU,
        },
    }
}
