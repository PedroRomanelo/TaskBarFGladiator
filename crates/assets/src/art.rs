//! Ícones das ações e da interface (7 × 7). `#` = desenho, `.` = transparente.

use crate::CharMap;

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
    "......#", ".....#.", "....#..", "...#...", "..#....", ".#.....", "#......",
];
const ICON_MEDIUM: &[&str] = &[
    "...#..#", "..#..#.", ".#..#..", "#..#...", "..#....", ".#.....", "#......",
];
const ICON_HEAVY: &[&str] = &[
    "...#..#", "..#..#.", ".#..#..", "#..#..#", "..#..#.", ".#..#..", "#..#...",
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
