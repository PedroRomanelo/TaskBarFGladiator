//! Cenas fixas usadas pelos testes de imagem e de alocação: uma por etapa do jogo.

use assets::{Armor, Beard, Frame, Hair, Shield, Weapon};
use render::{
    ButtonState, FighterView, FloatText, Floater, HitId, Look, Phase, StageView, Theme, Tone,
};

fn look(i: u8) -> Look {
    Look {
        skin: i,
        hair: Hair::from_index(i + 1),
        hair_color: i + 2,
        eyes: i,
        beard: Beard::from_index(i + 3),
        weapon: Weapon::ALL[usize::from(i + 2) % Weapon::ALL.len()],
        armor: Armor::ALL[usize::from(i + 2) % Armor::ALL.len()],
        shield: Shield::ALL[usize::from(i + 1) % Shield::ALL.len()],
        ..Look::default()
    }
}

fn fighter(pos: i32, frame: Frame, hp: i32, look: Look) -> FighterView {
    FighterView {
        name: "TESTE".into(),
        level: 5,
        hp,
        hp_max: 80,
        stamina: 30,
        stamina_max: 60,
        pos_pm: pos * 1000,
        frame,
        anim_ms: 0,
        flash: false,
        look,
        float: None,
    }
}

fn stage(
    player: FighterView,
    opponent: FighterView,
    caption: &str,
    tone: Tone,
    phase: Phase,
) -> StageView {
    StageView {
        player,
        opponent,
        cells: 10,
        caption: caption.into(),
        tone,
        hint: String::new(),
        phase,
    }
}

/// Cenas: nome, visão e região sob o mouse.
pub fn scenes() -> Vec<(&'static str, StageView, Option<HitId>)> {
    let mut hub = stage(
        fighter(2, Frame::Idle, 80, look(0)),
        fighter(7, Frame::Idle, 80, look(1)),
        "PRÓXIMO: AÇÃO Ç",
        Tone::Normal,
        Phase::Hub {
            gold: 98765,
            unspent: 3,
            level_label: "NV".into(),
            points_label: "PTS".into(),
            fight_label: "LUTAR".into(),
        },
    );
    hub.hint = "NV 12 - CLIQUE EM LUTAR".into();
    let mut struck = fighter(4, Frame::Defend, 20, look(3));
    struck.flash = true;
    struck.float = Some(Floater {
        text: FloatText::Damage {
            amount: 12,
            critical: true,
        },
        rise: 4,
    });
    let mut tired = fighter(2, Frame::Idle, 80, look(4));
    tired.float = Some(Floater {
        text: FloatText::Stamina(9),
        rise: 1,
    });
    let mut missed = fighter(7, Frame::Down, 0, look(5));
    missed.float = Some(Floater {
        text: FloatText::Label("ERROU"),
        rise: 6,
    });
    vec![
        ("preparar", hub, Some(HitId::Fight)),
        (
            "luta_sua_vez",
            stage(
                fighter(2, Frame::Idle, 50, look(2)),
                fighter(7, Frame::Idle, 80, look(3)),
                "RODADA 1: ESCOLHA SUA AÇÃO",
                Tone::Prompt,
                Phase::Fight {
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
            ),
            Some(HitId::Action(0)),
        ),
        (
            "luta_golpe",
            stage(
                fighter(3, Frame::Attack, 50, look(2)),
                struck,
                "TESTE: CRÍTICO! 12 DE DANO",
                Tone::Good,
                Phase::Fight {
                    buttons: Some([ButtonState::Disabled; 8]),
                    paused: false,
                },
            ),
            None,
        ),
        (
            "luta_auto",
            stage(
                tired,
                missed,
                "PAUSADO",
                Tone::Prompt,
                Phase::Fight {
                    buttons: None,
                    paused: true,
                },
            ),
            None,
        ),
        (
            "resultado",
            stage(
                fighter(4, Frame::Down, 0, look(0)),
                fighter(5, Frame::Idle, 30, look(1)),
                "DERROTA",
                Tone::Bad,
                Phase::Result {
                    detail: "+10 XP  +5 OURO".into(),
                    ok_label: "CONTINUAR".into(),
                },
            ),
            None,
        ),
    ]
}

/// Temas: tema e nome.
pub fn themes() -> [(Theme, &'static str); 3] {
    [
        (
            Theme {
                light: false,
                high_contrast: false,
            },
            "escuro",
        ),
        (
            Theme {
                light: true,
                high_contrast: false,
            },
            "claro",
        ),
        (
            Theme {
                light: false,
                high_contrast: true,
            },
            "contraste",
        ),
    ]
}
