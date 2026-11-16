//! Peças do lutador (boneco em camadas) e a montagem de cada pose.
//!
//! O lutador é desenhado numa tela de [`CANVAS`] × [`CANVAS`] pixels lógicos, voltado para a direita,
//! com os pés na última linha. Em vez de um desenho inteiro por pose, cada pose posiciona peças
//! pequenas (pernas, tronco, cabeça, braços) e, sobre elas, as camadas trocáveis:
//!
//! - **aparência** (do jogador): tom de pele, olhos, cabelo, cor do cabelo e barba. Pele, olhos e
//!   cor do cabelo são só cores da paleta; cabelo e barba são desenhos sobrepostos à cabeça;
//! - **equipamento** (fixo, igual para todos): arma, escudo e as peças da armadura (elmo,
//!   peitoral e grevas).
//!
//! Para criar ou alterar uma peça, edite a grade de texto correspondente e gere as imagens com
//! `cargo run -p render --example preview`. Legenda dos caracteres em [`crate::Ink::from_char`].

use crate::{Armor, Beard, CharMap, Frame, Hair, Look, Shield, Weapon};

/// Lado da tela quadrada do lutador, em pixels lógicos.
pub const CANVAS: i32 = 32;
/// Coluna da tela que fica no centro da casa da arena (eixo do corpo).
pub const CENTER_X: i32 = 12;
/// Número máximo de peças em uma montagem.
pub const MAX_PIECES: usize = 12;

/// Rotação de uma peça.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Turn {
    /// Como desenhada (armas apontam para cima).
    Up,
    /// Girada 90° no sentido horário (armas apontam para a frente).
    Forward,
}

/// Peça posicionada na tela do lutador.
#[derive(Debug, Clone, Copy)]
pub struct Piece {
    /// Desenho.
    pub map: CharMap,
    /// Coluna do canto superior esquerdo (já considerando a rotação).
    pub x: i32,
    /// Linha do canto superior esquerdo (já considerando a rotação).
    pub y: i32,
    /// Rotação.
    pub turn: Turn,
}

/// Montagem de um quadro: peças na ordem de desenho (as últimas ficam por cima).
#[derive(Debug, Clone, Copy)]
pub struct Rig {
    /// Peças.
    pub pieces: [Option<Piece>; MAX_PIECES],
    /// Verdadeiro se o lutador está caído: a tela inteira é girada 90° (cabeça para trás).
    pub lying: bool,
}

// ---------------------------------------------------------------------------------------------
// Corpo
// ---------------------------------------------------------------------------------------------

const HEAD: &[&str] = &[
    "..kkkkkk..",
    ".kssssssk.",
    "kssssssssk",
    "kSsskkskkk",
    "kSsseiseik",
    "kSsseiseik",
    "kSssssssSk",
    "kSSssnnsk.",
    ".kSSsssk..",
    "..kkkkk...",
];

const TORSO: &[&str] = &[
    "..kkkkkk..",
    ".kssssssk.",
    "kSssssssSk",
    "kSsssSssSk",
    "kSssssssSk",
    ".kSsssSsk.",
    ".kSssssSk.",
    ".kSssssSk.",
    ".kkkkkkkk.",
];

const LEGS_IDLE: &[&str] = &[
    ".kbbbbbbbbk.",
    ".kttttttttk.",
    ".kttTttttTk.",
    ".ktTtkktTtk.",
    ".kkkk..kkkk.",
    "..kSsk.kSsk.",
    "..kGgk.kGgk.",
    "..kGgk.kGgk.",
    "..kGgk.kGgk.",
    "..kBbk.kBbk.",
    "..kBbbkkBbbk",
    "..kkkkkkkkkk",
];

const LEGS_ATTACK: &[&str] = &[
    "....kbbbbbbbbk...",
    "....kttttttttk...",
    "...kttTttttTttk..",
    "..kttTkkkkkktTtk.",
    "..kkkk......kkkk.",
    ".kSsk.......kSsk.",
    ".kGgk.......kGgk.",
    "kGgk........kGgk.",
    "kGgk........kGgk.",
    "kBbk........kBbk.",
    "kBbbk.......kBbbk",
    "kkkkk.......kkkkk",
];

const LEGS_DEFEND: &[&str] = &[
    "..kbbbbbbbbk...",
    "..kttttttttk...",
    ".kttTttttTttk..",
    ".ktTtkkkktTtk..",
    ".kkkk....kkkk..",
    ".kSsk....kSsk..",
    ".kGgk....kGgk..",
    "kGgk......kGgk.",
    "kGgk......kGgk.",
    "kBbk......kBbk.",
    "kBbbk.....kBbbk",
    "kkkkk.....kkkkk",
];

// Braço da arma (o de trás) e braço do escudo (o da frente), por pose.
const ARM_BACK_IDLE: &[&str] = &["..kk.", ".kssk", "kssk.", "kSsk.", "kssk.", ".kk.."];
const ARM_BACK_ATTACK: &[&str] = &[".kkkkkkkk.", "kSsssssssk", "kSSSsssssk", ".kkkkkkkk."];
const ARM_BACK_DEFEND: &[&str] = &[".kk..", "kssk.", "kSsk.", "kSssk", ".kssk", "..kk."];
const ARM_FRONT_IDLE: &[&str] = &["..kk.", ".kssk", "kSssk", "kSsk.", ".kk.."];
const ARM_FRONT_ATTACK: &[&str] = &[".kk.", "kssk", "kSsk", "kSsk", ".kk."];
const ARM_FRONT_DEFEND: &[&str] = &["...kk.", "..kssk", ".kSsk.", "kSsk..", ".kk..."];

// ---------------------------------------------------------------------------------------------
// Aparência: cabelos e barbas. Grade de 12 × 13 alinhada à cabeça com 1 coluna de folga de cada
// lado e 3 linhas acima (a cabeça ocupa as colunas 1 a 10 e as linhas 3 a 12).
// ---------------------------------------------------------------------------------------------

const HAIR_SHORT: &[&str] = &[
    "............",
    "............",
    "...kkkkkk...",
    "..khhhhhhk..",
    ".khhhhhhhhk.",
    ".khhHhhhk...",
    ".khHh.......",
    ".khH........",
    ".khh........",
    "..kk........",
    "............",
    "............",
    "............",
];

const HAIR_LONG: &[&str] = &[
    "............",
    "............",
    "...kkkkkk...",
    "..khhhhhhk..",
    ".khhhhhhhhk.",
    ".khhHhhhk...",
    "khhHh.......",
    "khhH........",
    "khhh........",
    "khHh........",
    "khHhk.......",
    "khhhk.......",
    ".kkk........",
];

const HAIR_MOHAWK: &[&str] = &[
    "...kkkkkk...",
    "..khhHhhhk..",
    ".khhHhhHhhk.",
    ".khkkkkkkk..",
    ".khk........",
    ".khk........",
    "..k.........",
    "............",
    "............",
    "............",
    "............",
    "............",
    "............",
];

const HAIR_TOPKNOT: &[&str] = &[
    "..kkk.......",
    ".khhhk......",
    ".khHhkkkk...",
    "..khhhhhhk..",
    ".khhhhhhhhk.",
    ".khHhhhh....",
    ".khh........",
    ".kh.........",
    "..k.........",
    "............",
    "............",
    "............",
    "............",
];

const HAIR_CURLY: &[&str] = &[
    "............",
    "..kk.kk.kk..",
    ".khhkhhkhhk.",
    "khhhhhhhhhhk",
    "khHhhHhhHhhk",
    "khhhHhhhhk..",
    "khHhh.......",
    "khhH........",
    "khHh........",
    ".khh........",
    "..kk........",
    "............",
    "............",
];

const BEARD_STUBBLE: &[&str] = &[
    "............",
    "............",
    "............",
    "............",
    "............",
    "............",
    "............",
    "............",
    "............",
    "..S.....S...",
    "...S.S..S...",
    "....S.S.....",
    "............",
];

const BEARD_MUSTACHE: &[&str] = &[
    "............",
    "............",
    "............",
    "............",
    "............",
    "............",
    "............",
    "............",
    "............",
    ".....hhhh...",
    ".....h..h...",
    "............",
    "............",
];

const BEARD_FULL: &[&str] = &[
    "............",
    "............",
    "............",
    "............",
    "............",
    "............",
    "............",
    "............",
    "............",
    "..h...hhh...",
    "..hHh...hk..",
    "..khHhhhk...",
    "...kkkkk....",
];

const BEARD_LONG: &[&str] = &[
    "............",
    "............",
    "............",
    "............",
    "............",
    "............",
    "............",
    "............",
    "............",
    "..h...hhh...",
    "..hHh...hk..",
    "..khHhhhhk..",
    "...khHhhk...",
    "...khHhhk...",
    "....khhk....",
    ".....kk.....",
];

// ---------------------------------------------------------------------------------------------
// Equipamento fixo: elmos (mesma grade de 12 × 13 dos cabelos), peitorais (grade do tronco com 1
// coluna de folga de cada lado), armas (apontando para cima) e escudos.
// ---------------------------------------------------------------------------------------------

const HELM_LEATHER: &[&str] = &[
    "............",
    "............",
    "...kkkkkk...",
    "..kbbbbbbk..",
    ".kbbbbbbbbk.",
    ".kBBBBBBBBk.",
    ".kbBk.......",
    ".kbk........",
    ".kbk........",
    "..k.........",
    "............",
    "............",
    "............",
];

const HELM_IRON: &[&str] = &[
    "............",
    "............",
    "...kkkkkk...",
    "..kmmmmmmk..",
    ".kmmlmmmmmk.",
    ".kmmmmmmmmk.",
    ".kMMMMMMMMMk",
    ".kmMk..M....",
    ".kmMk..M....",
    ".kmMk.......",
    "..kk........",
    "............",
    "............",
];

const HELM_BRONZE: &[&str] = &[
    "...kkkkkkk..",
    "..ktTttTttk.",
    "..kkkkkkkkk.",
    "..koooooook.",
    ".koolooooook",
    ".koooooooook",
    ".kOOOOOOOOOk",
    ".koOk..O....",
    ".koOk..O....",
    ".koOOk......",
    "..koOk......",
    "...kk.......",
    "............",
];

const CHEST_TUNIC: &[&str] = &[
    "...kkkkkk...",
    "..kcccccck..",
    ".kCccccccCk.",
    ".kCcccccCCk.",
    ".kCccccccCk.",
    "..kCccccCk..",
    "..kCccccCk..",
    "..kCccccCk..",
    "..kkkkkkkk..",
];

const CHEST_LEATHER: &[&str] = &[
    "...kkkkkk...",
    ".kkbbbbbbkk.",
    "kbBbbbbbbBbk",
    "kkBbbBBbbBkk",
    ".kBbbbbbbBk.",
    "..kBbBBbBk..",
    "..kBbbbbBk..",
    "..kBBBBBBk..",
    "..kkkkkkkk..",
];

const CHEST_MAIL: &[&str] = &[
    "...kkkkkk...",
    ".kkmMmMmMkk.",
    "kmMmMmMmMmMk",
    "kkmMmMmMmMkk",
    ".kMmMmMmMmk.",
    "..kMmMmMmk..",
    "..kmMmMmMk..",
    "..kMmMmMmk..",
    "..kkkkkkkk..",
];

const CHEST_PLATES: &[&str] = &[
    ".kkkkkkkkkk.",
    "koolkooklook",
    "kOookoolookk",
    ".kkooloooOk.",
    ".kOoooooOOk.",
    "..kOoOOoOk..",
    "..kOooooOk..",
    "..kOOOOOOk..",
    "..kkkkkkkk..",
];

/// Empunhadura de uma arma desenhada apontando para cima: o pixel que fica na mão do lutador.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Grip {
    /// Coluna do cabo.
    pub x: i32,
    /// Linha segurada em guarda (arma para cima).
    pub carry: i32,
    /// Linha segurada no golpe (arma para a frente). Armas longas são seguradas mais perto da
    /// ponta para caberem na tela. `None`: a arma continua em pé no golpe (arco).
    pub thrust: Option<i32>,
}

impl Grip {
    const fn new(x: i32, carry: i32, thrust: i32) -> Self {
        Self {
            x,
            carry,
            thrust: Some(thrust),
        }
    }

    const fn same(x: i32, y: i32) -> Self {
        Self::new(x, y, y)
    }

    const fn upright(x: i32, y: i32) -> Self {
        Self {
            x,
            carry: y,
            thrust: None,
        }
    }
}

// Armas: desenhadas apontando para cima.
const KNIFE: &[&str] = &[
    "..k.", ".kmk", ".kmk", ".kmk", ".kMk", "kook", ".kbk", ".kbk", "..k.",
];
const KNIFE_GRIP: Grip = Grip::same(2, 6);

const SHORTSWORD: &[&str] = &[
    "..kk..", ".kmlk.", ".kmlk.", ".kmlk.", ".kmlk.", ".kmlk.", ".kmlk.", ".kmlk.", ".kMmk.",
    "kooook", ".kbbk.", ".kbbk.", ".kook.", "..kk..",
];
const SHORTSWORD_GRIP: Grip = Grip::same(2, 10);

const SPEAR: &[&str] = &[
    ".k.", "kmk", "kmk", "kMk", "kok", "kwk", "kwk", "kwk", "kwk", "kwk", "kwk", "kwk", "kwk",
    "kwk", "kwk", "kwk", "kwk", "kwk", "kwk", "kwk", "kWk", "kWk", "kWk", ".k.",
];
const SPEAR_GRIP: Grip = Grip::new(1, 15, 9);

const AXE: &[&str] = &[
    "...k...", ".kkwkk.", "kmmwmmk", "kmlwlmk", "kmmwmmk", "kMMwMMk", ".kkwkk.", "..kwk..",
    "..kwk..", "..kwk..", "..kwk..", "..kwk..", "..kwk..", "..kWk..", "...k...",
];
const AXE_GRIP: Grip = Grip::new(3, 11, 9);

const BOW: &[&str] = &[
    "..kk..", ".kwwk.", "kwkck.", "kwk.c.", "kwk.c.", "kwk.c.", "kwk.c.", "kbk.c.", "kbk.c.",
    "kwk.c.", "kwk.c.", "kwk.c.", "kwk.c.", "kwkck.", ".kwwk.", "..kk..",
];
const BOW_GRIP: Grip = Grip::upright(1, 8);

const STAFF: &[&str] = &[
    ".kkk.", "kooik", "koiok", "kwook", ".kwk.", ".kwk.", ".kwk.", ".kwk.", ".kwk.", ".kwk.",
    ".kwk.", ".kwk.", ".kwk.", ".kwk.", ".kwk.", ".kwk.", ".kwk.", ".kwk.", ".kwk.", ".kWk.",
    ".kWk.", ".kWk.", "..k..",
];
const STAFF_GRIP: Grip = Grip::new(2, 13, 9);

const BUCKLER: &[&str] = &[".kkkk.", "kwwwwk", "kwmmWk", "kwmMWk", "kwWWWk", ".kkkk."];

const ROUND_SHIELD: &[&str] = &[
    "..kkkk..", ".kmmmmk.", "kmwwwwMk", "kmwwwWMk", "kmwmmWMk", "kmwmMWMk", "kmwwWWMk", "kmWWWWMk",
    ".kMMMMk.", "..kkkk..",
];

const OVAL_SHIELD: &[&str] = &[
    "..kkkk..", ".kooook.", "kobbbbOk", "kobbbBOk", "kobbbBOk", "koboobOk", "kobooBOk", "kobbbBOk",
    "kobbBBOk", "kobBBBOk", ".kOOOOk.", "..kkkk..",
];

const TOWER_SHIELD: &[&str] = &[
    ".kkkkkk.", "koooooOk", "kottttOk", "kottttOk", "kottTtOk", "kooooOOk", "kottTtOk", "kottTtOk",
    "kolooOOk", "kottTtOk", "kottTtOk", "kooooOOk", "kotTTtOk", "kotTTtOk", "kOOOOOOk", ".kkkkkk.",
];

// ---------------------------------------------------------------------------------------------
// Montagem
// ---------------------------------------------------------------------------------------------

/// Posições das peças em uma pose (tela do lutador).
struct Pose {
    legs: (&'static [&'static str], i32, i32),
    torso: (i32, i32),
    head: (i32, i32),
    arm_back: (&'static [&'static str], i32, i32),
    arm_front: (&'static [&'static str], i32, i32),
    /// Mão da arma e rotação da arma.
    weapon_hand: (i32, i32, Turn),
    /// Centro do escudo.
    shield_center: (i32, i32),
    /// Arma desenhada por cima do corpo (golpe) ou por trás (guarda).
    weapon_in_front: bool,
}

const POSE_IDLE: Pose = Pose {
    legs: (LEGS_IDLE, 6, 20),
    torso: (7, 12),
    head: (7, 3),
    arm_back: (ARM_BACK_IDLE, 3, 13),
    arm_front: (ARM_FRONT_IDLE, 14, 14),
    weapon_hand: (4, 16, Turn::Up),
    shield_center: (19, 17),
    weapon_in_front: false,
};

const POSE_ATTACK: Pose = Pose {
    legs: (LEGS_ATTACK, 4, 20),
    torso: (9, 12),
    head: (10, 3),
    arm_back: (ARM_BACK_ATTACK, 13, 13),
    arm_front: (ARM_FRONT_ATTACK, 8, 13),
    weapon_hand: (21, 14, Turn::Forward),
    shield_center: (8, 17),
    weapon_in_front: true,
};

const POSE_DEFEND: Pose = Pose {
    legs: (LEGS_DEFEND, 4, 20),
    torso: (6, 13),
    head: (6, 4),
    arm_back: (ARM_BACK_DEFEND, 2, 13),
    arm_front: (ARM_FRONT_DEFEND, 13, 12),
    weapon_hand: (3, 15, Turn::Up),
    shield_center: (19, 13),
    weapon_in_front: false,
};

fn hair_map(h: Hair) -> Option<&'static [&'static str]> {
    match h {
        Hair::Bald => None,
        Hair::Short => Some(HAIR_SHORT),
        Hair::Long => Some(HAIR_LONG),
        Hair::Mohawk => Some(HAIR_MOHAWK),
        Hair::Topknot => Some(HAIR_TOPKNOT),
        Hair::Curly => Some(HAIR_CURLY),
    }
}

fn beard_map(b: Beard) -> Option<&'static [&'static str]> {
    match b {
        Beard::None => None,
        Beard::Stubble => Some(BEARD_STUBBLE),
        Beard::Mustache => Some(BEARD_MUSTACHE),
        Beard::Full => Some(BEARD_FULL),
        Beard::Long => Some(BEARD_LONG),
    }
}

fn helm_map(a: Armor) -> Option<&'static [&'static str]> {
    match a {
        Armor::None | Armor::Tunic => None,
        Armor::Leather => Some(HELM_LEATHER),
        Armor::Mail => Some(HELM_IRON),
        Armor::Plates => Some(HELM_BRONZE),
    }
}

fn chest_map(a: Armor) -> Option<&'static [&'static str]> {
    match a {
        Armor::None => None,
        Armor::Tunic => Some(CHEST_TUNIC),
        Armor::Leather => Some(CHEST_LEATHER),
        Armor::Mail => Some(CHEST_MAIL),
        Armor::Plates => Some(CHEST_PLATES),
    }
}

/// Desenho de uma arma (apontando para cima) e sua empunhadura.
pub fn weapon_map(w: Weapon) -> Option<(CharMap, Grip)> {
    let (rows, grip) = match w {
        Weapon::None => return None,
        Weapon::Knife => (KNIFE, KNIFE_GRIP),
        Weapon::Shortsword => (SHORTSWORD, SHORTSWORD_GRIP),
        Weapon::Spear => (SPEAR, SPEAR_GRIP),
        Weapon::Axe => (AXE, AXE_GRIP),
        Weapon::Bow => (BOW, BOW_GRIP),
        Weapon::Staff => (STAFF, STAFF_GRIP),
    };
    Some((CharMap { rows }, grip))
}

/// Desenho de um escudo.
pub fn shield_map(s: Shield) -> Option<CharMap> {
    let rows = match s {
        Shield::None => return None,
        Shield::Buckler => BUCKLER,
        Shield::Round => ROUND_SHIELD,
        Shield::Oval => OVAL_SHIELD,
        Shield::Tower => TOWER_SHIELD,
    };
    Some(CharMap { rows })
}

fn piece(rows: &'static [&'static str], x: i32, y: i32) -> Option<Piece> {
    Some(Piece {
        map: CharMap { rows },
        x,
        y,
        turn: Turn::Up,
    })
}

fn weapon_piece(w: Weapon, hand: (i32, i32, Turn)) -> Option<Piece> {
    let (map, grip) = weapon_map(w)?;
    let (hx, hy, wanted) = hand;
    let h = map.height() as i32;
    // Girar 90° no sentido horário leva o pixel (x, y) para (h − 1 − y, x).
    let (ax, ay, turn) = match (wanted, grip.thrust) {
        (Turn::Forward, Some(gy)) => (h - 1 - gy, grip.x, Turn::Forward),
        _ => (grip.x, grip.carry, Turn::Up),
    };
    Some(Piece {
        map,
        x: hx - ax,
        y: hy - ay,
        turn,
    })
}

fn shield_piece(s: Shield, center: (i32, i32)) -> Option<Piece> {
    let map = shield_map(s)?;
    Some(Piece {
        map,
        x: center.0 - map.width() as i32 / 2,
        y: center.1 - map.height() as i32 / 2,
        turn: Turn::Up,
    })
}

/// Monta as peças de um quadro para uma aparência e um equipamento.
pub fn rig(frame: Frame, look: &Look) -> Rig {
    let (pose, lying) = match frame {
        Frame::Idle | Frame::Walk | Frame::Hurt => (&POSE_IDLE, false),
        Frame::Attack | Frame::AttackMedium | Frame::AttackHeavy => (&POSE_ATTACK, false),
        Frame::Defend | Frame::Block => (&POSE_DEFEND, false),
        Frame::Down => (&POSE_IDLE, true),
    };
    // Caído: arma e escudo ficam fora do desenho (soltos na queda).
    let weapon = if lying {
        None
    } else {
        weapon_piece(look.weapon, pose.weapon_hand)
    };
    let shield = if lying {
        None
    } else {
        shield_piece(look.shield, pose.shield_center)
    };
    let (hx, hy) = pose.head;
    let (tx, ty) = pose.torso;
    let (weapon_back, weapon_front) = if pose.weapon_in_front {
        (None, weapon)
    } else {
        (weapon, None)
    };
    // Com elmo, o cabelo comprido continua aparecendo por baixo; os demais ficam cobertos.
    let helm = helm_map(look.armor);
    let hair = match (helm, look.hair) {
        (Some(_), Hair::Long) | (None, _) => hair_map(look.hair),
        (Some(_), _) => None,
    };
    let (arm_first, arm_last) = if pose.weapon_in_front {
        // No golpe o braço da arma cruza o corpo por cima.
        (pose.arm_front, pose.arm_back)
    } else {
        (pose.arm_back, pose.arm_front)
    };
    Rig {
        pieces: [
            weapon_back,
            piece(arm_first.0, arm_first.1, arm_first.2),
            piece(pose.legs.0, pose.legs.1, pose.legs.2),
            piece(TORSO, tx, ty),
            chest_map(look.armor).and_then(|m| piece(m, tx - 1, ty)),
            piece(HEAD, hx, hy),
            hair.and_then(|m| piece(m, hx - 1, hy - 3)),
            beard_map(look.beard).and_then(|m| piece(m, hx - 1, hy - 3)),
            helm.and_then(|m| piece(m, hx - 1, hy - 3)),
            weapon_front,
            piece(arm_last.0, arm_last.1, arm_last.2),
            shield,
        ],
        lying,
    }
}

/// Todos os desenhos de peças, com nome (para os testes e para a folha de sprites).
pub fn all_parts() -> Vec<(&'static str, CharMap)> {
    let named: &[(&str, &'static [&'static str])] = &[
        ("cabeca", HEAD),
        ("tronco", TORSO),
        ("pernas_parado", LEGS_IDLE),
        ("pernas_golpe", LEGS_ATTACK),
        ("pernas_defesa", LEGS_DEFEND),
        ("braco_arma_parado", ARM_BACK_IDLE),
        ("braco_arma_golpe", ARM_BACK_ATTACK),
        ("braco_arma_defesa", ARM_BACK_DEFEND),
        ("braco_escudo_parado", ARM_FRONT_IDLE),
        ("braco_escudo_golpe", ARM_FRONT_ATTACK),
        ("braco_escudo_defesa", ARM_FRONT_DEFEND),
        ("cabelo_curto", HAIR_SHORT),
        ("cabelo_longo", HAIR_LONG),
        ("cabelo_moicano", HAIR_MOHAWK),
        ("cabelo_coque", HAIR_TOPKNOT),
        ("cabelo_cacheado", HAIR_CURLY),
        ("barba_rala", BEARD_STUBBLE),
        ("barba_bigode", BEARD_MUSTACHE),
        ("barba_cheia", BEARD_FULL),
        ("barba_longa", BEARD_LONG),
        ("elmo_couro", HELM_LEATHER),
        ("elmo_ferro", HELM_IRON),
        ("elmo_bronze", HELM_BRONZE),
        ("peitoral_tunica", CHEST_TUNIC),
        ("peitoral_couro", CHEST_LEATHER),
        ("peitoral_malha", CHEST_MAIL),
        ("peitoral_placas", CHEST_PLATES),
        ("arma_faca", KNIFE),
        ("arma_espada_curta", SHORTSWORD),
        ("arma_lanca", SPEAR),
        ("arma_machado", AXE),
        ("arma_arco", BOW),
        ("arma_bastao", STAFF),
        ("escudo_broquel", BUCKLER),
        ("escudo_redondo", ROUND_SHIELD),
        ("escudo_oval", OVAL_SHIELD),
        ("escudo_torre", TOWER_SHIELD),
    ];
    named
        .iter()
        .map(|&(name, rows)| (name, CharMap { rows }))
        .collect()
}
