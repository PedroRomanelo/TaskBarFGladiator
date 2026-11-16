//! Gera PNGs de pré-visualização: `cargo run -p render --example preview -- <pasta>`.
//!
//! - `folha_*.png`: folhas de sprites (poses, aparências e equipamentos) para revisar a arte;
//! - `palco_*.png`: o palco em cada etapa do jogo;
//! - `painel_*.png`: o painel.

use assets::{Armor, Beard, CANVAS, Frame, Hair, Shield, Weapon};
use render::draw::{self, SpritePalette, TeamColors};
use render::scene::{PANEL_HEIGHT, PANEL_WIDTH, STAGE_HEIGHT, STAGE_WIDTH};
use render::{
    ButtonState, FighterView, FloatText, Floater, HitId, Look, PanelRow, PanelView, Phase, Rgba,
    StageView, Surface, Theme, Tone, draw_panel, draw_stage, png,
};

const RED: TeamColors = TeamColors {
    main: Rgba::rgb(0xc8402f),
    shade: Rgba::rgb(0x862419),
};

fn hero() -> Look {
    Look {
        skin: 2,
        hair: Hair::Short,
        hair_color: 1,
        eyes: 1,
        beard: Beard::Stubble,
        weapon: Weapon::Shortsword,
        armor: Armor::Leather,
        shield: Shield::Round,
    }
}

fn rival() -> Look {
    Look {
        skin: 4,
        hair: Hair::Mohawk,
        hair_color: 0,
        eyes: 0,
        beard: Beard::Full,
        weapon: Weapon::Axe,
        armor: Armor::Tunic,
        shield: Shield::None,
    }
}

fn fighter(name: &str, pos: i32, frame: Frame, hp: i32, look: Look) -> FighterView {
    FighterView {
        name: name.into(),
        level: 7,
        hp,
        hp_max: 88,
        stamina: 40,
        stamina_max: 60,
        pos_pm: pos * 1000,
        frame,
        flash: false,
        look,
        float: None,
    }
}

/// Folha de sprites: uma linha por aparência, uma coluna por quadro.
fn sheet(rows: &[Look], scale: i32) -> Surface {
    let cell = (CANVAS + 4) * scale;
    let mut s = Surface::new(
        (cell * Frame::ALL.len() as i32) as usize,
        (cell * rows.len() as i32) as usize,
    );
    for (r, look) in rows.iter().enumerate() {
        let palette = SpritePalette::new(look, RED);
        for (c, frame) in Frame::ALL.iter().enumerate() {
            draw::fighter(
                &mut s,
                c as i32 * cell + 2 * scale,
                r as i32 * cell + 2 * scale,
                scale,
                *frame,
                look,
                &palette,
                false,
                false,
            );
        }
    }
    s
}

fn main() -> std::io::Result<()> {
    let dir = std::env::args().nth(1).unwrap_or_else(|| "preview".into());
    std::fs::create_dir_all(&dir)?;
    let bg = Some(0x6f8fa8);

    // Folhas de sprites.
    let base = Look::default();
    let appearance: Vec<Look> = Hair::ALL
        .iter()
        .enumerate()
        .map(|(i, &hair)| Look {
            skin: i as u8,
            hair,
            hair_color: i as u8,
            eyes: i as u8,
            beard: Beard::from_index(i as u8),
            ..base
        })
        .collect();
    std::fs::write(
        format!("{dir}/folha_aparencia.png"),
        png::encode(&sheet(&appearance, 6), bg),
    )?;
    let gear: Vec<Look> = (0..Weapon::ALL.len())
        .map(|i| Look {
            skin: 1,
            hair: Hair::Long,
            hair_color: 2,
            weapon: Weapon::ALL[i],
            armor: Armor::ALL[i % Armor::ALL.len()],
            shield: Shield::ALL[i % Shield::ALL.len()],
            ..base
        })
        .collect();
    std::fs::write(
        format!("{dir}/folha_equipamento.png"),
        png::encode(&sheet(&gear, 6), bg),
    )?;

    // Palco em cada etapa.
    let mut hits = Vec::new();
    let dark = Theme {
        light: false,
        high_contrast: false,
    };
    let light = Theme {
        light: true,
        high_contrast: false,
    };
    let mut hit = fighter("TARO", 4, Frame::Defend, 34, rival());
    hit.float = Some(Floater {
        text: FloatText::Damage {
            amount: 12,
            critical: false,
        },
        rise: 3,
    });
    let views = [
        (
            "1_preparar",
            StageView {
                player: fighter("ARIA", 2, Frame::Idle, 88, hero()),
                opponent: fighter("TARO", 7, Frame::Idle, 88, rival()),
                cells: 10,
                caption: "PRÓXIMO: TARO, O PESCADOR".into(),
                tone: Tone::Normal,
                hint: "NV 1 - CLIQUE EM LUTAR".into(),
                phase: Phase::Hub {
                    gold: 1234,
                    unspent: 3,
                    level_label: "NV".into(),
                    points_label: "PTS".into(),
                    fight_label: "LUTAR".into(),
                },
            },
            None,
        ),
        (
            "2_sua_vez",
            StageView {
                player: fighter("ARIA", 2, Frame::Idle, 61, hero()),
                opponent: fighter("TARO", 7, Frame::Idle, 70, rival()),
                cells: 10,
                caption: "RODADA 1: ESCOLHA SUA AÇÃO".into(),
                tone: Tone::Prompt,
                hint: String::new(),
                phase: Phase::Fight {
                    buttons: Some([
                        ButtonState::Enabled,
                        ButtonState::Enabled,
                        ButtonState::Disabled,
                        ButtonState::Disabled,
                        ButtonState::Disabled,
                        ButtonState::Enabled,
                        ButtonState::Enabled,
                        ButtonState::Enabled,
                    ]),
                    paused: false,
                },
            },
            Some(HitId::Action(0)),
        ),
        (
            "3_golpe",
            StageView {
                player: fighter("ARIA", 3, Frame::Attack, 61, hero()),
                opponent: hit,
                cells: 10,
                caption: "ARIA: GOLPE MÉDIO - 12 DE DANO".into(),
                tone: Tone::Good,
                hint: String::new(),
                phase: Phase::Fight {
                    buttons: Some([ButtonState::Disabled; 8]),
                    paused: false,
                },
            },
            None,
        ),
        (
            "4_auto",
            StageView {
                player: fighter("ARIA", 5, Frame::Defend, 30, hero()),
                opponent: fighter("TARO", 6, Frame::Attack, 52, rival()),
                cells: 10,
                caption: "TARO: GOLPE PESADO".into(),
                tone: Tone::Bad,
                hint: String::new(),
                phase: Phase::Fight {
                    buttons: None,
                    paused: false,
                },
            },
            None,
        ),
        (
            "5_resultado",
            StageView {
                player: fighter("ARIA", 4, Frame::Idle, 20, hero()),
                opponent: fighter("TARO", 6, Frame::Down, 0, rival()),
                cells: 10,
                caption: "VITÓRIA!".into(),
                tone: Tone::Good,
                hint: String::new(),
                phase: Phase::Result {
                    detail: "+92 XP  +57 OURO".into(),
                    ok_label: "CONTINUAR".into(),
                },
            },
            None,
        ),
    ];
    for (name, view, hover) in &views {
        for (sc, theme, tname) in [(3, dark, "escuro"), (3, light, "claro"), (2, dark, "x2")] {
            let mut s = Surface::new((STAGE_WIDTH * sc) as usize, (STAGE_HEIGHT * sc) as usize);
            draw_stage(&mut s, view, theme, *hover, &mut hits);
            std::fs::write(
                format!("{dir}/palco_{name}_{tname}.png"),
                png::encode(&s, bg),
            )?;
        }
    }

    let panel = PanelView {
        tabs: vec![
            "FICHA".into(),
            "LOJA".into(),
            "LIGA".into(),
            "VISUAL".into(),
            "CONFIG".into(),
        ],
        active_tab: 3,
        header: "APARÊNCIA DO GLADIADOR".into(),
        rows: [
            ("PELE", "3/6"),
            ("CABELO", "CURTO"),
            ("COR DO CABELO", "2/7"),
            ("OLHOS", "2/5"),
            ("BARBA", "RALA"),
        ]
        .into_iter()
        .map(|(text, value)| PanelRow {
            text: text.into(),
            value: value.into(),
            button: Some(("TROCAR".into(), true)),
            highlight: false,
        })
        .collect(),
        footer: "ESC OU CLIQUE FORA PARA FECHAR".into(),
        preview: Some(hero()),
    };
    let mut s = Surface::new((PANEL_WIDTH * 2) as usize, (PANEL_HEIGHT * 2) as usize);
    draw_panel(&mut s, &panel, &mut hits);
    std::fs::write(format!("{dir}/painel_visual.png"), png::encode(&s, None))?;
    println!("PNGs gravados em {dir}");
    Ok(())
}
