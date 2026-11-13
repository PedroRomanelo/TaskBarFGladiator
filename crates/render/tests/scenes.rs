//! Golden images: hash dos pixels de cada cena em várias alturas e temas.
//!
//! Regenerar as referências após mudança visual deliberada:
//! `UPDATE_GOLDEN=1 cargo test -p render --test scenes`

use assets::Frame;
use render::scene::STRIP_WIDTH;
use render::{ButtonState, FighterView, StripView, Surface, Theme, draw_strip};
use std::collections::BTreeMap;
use std::path::PathBuf;

fn fighter(pos: i32, frame: Frame, hp: i32, flash: bool) -> FighterView {
    FighterView {
        name: "TESTE".into(),
        level: 5,
        hp,
        hp_max: 80,
        stamina: 30,
        stamina_max: 60,
        pos_pm: pos * 1000,
        frame,
        flash,
    }
}

fn scenes() -> Vec<(&'static str, StripView)> {
    vec![
        (
            "luta_manual",
            StripView::Fight {
                player: fighter(3, Frame::Attack, 50, false),
                opponent: fighter(4, Frame::Defend, 20, true),
                cells: 10,
                buttons: Some([ButtonState::Enabled; 8]),
                paused: false,
                banner: Some("CRÍTICO!".into()),
            },
        ),
        (
            "luta_auto",
            StripView::Fight {
                player: fighter(2, Frame::Idle, 80, false),
                opponent: fighter(7, Frame::Down, 0, false),
                cells: 10,
                buttons: None,
                paused: true,
                banner: None,
            },
        ),
        (
            "hub",
            StripView::Hub {
                name: "Ação Ç".into(),
                level: 12,
                gold: 98765,
                unspent: 3,
                next_label: "PRÓXIMO: TESTE".into(),
                fight_label: "LUTAR".into(),
            },
        ),
        (
            "resultado",
            StripView::Result {
                title: "DERROTA".into(),
                detail: "+10 XP +5 OURO".into(),
                won: false,
                ok_label: "OK".into(),
            },
        ),
    ]
}

fn hash(pixels: &[u32]) -> String {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for p in pixels {
        for b in p.to_le_bytes() {
            h ^= u64::from(b);
            h = h.wrapping_mul(0x0000_0100_0000_01b3);
        }
    }
    format!("{h:016x}")
}

#[test]
fn golden_images() {
    let mut computed = BTreeMap::new();
    let mut hits = Vec::new();
    for (name, view) in scenes() {
        for h in [24, 40, 48, 60, 72] {
            for (theme, t) in [
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
            ] {
                let (sc, _) = render::strip_scale(h);
                let mut s = Surface::new((STRIP_WIDTH * sc) as usize, h as usize);
                draw_strip(&mut s, &view, theme, &mut hits);
                computed.insert(format!("{name}_{h}_{t}"), hash(s.pixels()));
            }
        }
    }
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/golden/strip.json");
    if std::env::var_os("UPDATE_GOLDEN").is_some() {
        let text = serde_like_json(&computed);
        if let Err(e) = std::fs::write(&path, text) {
            panic!("{e}");
        }
        return;
    }
    let saved = match std::fs::read_to_string(&path) {
        Ok(t) => parse_simple_json(&t),
        Err(e) => panic!("{} ausente ({e}); rode com UPDATE_GOLDEN=1", path.display()),
    };
    assert_eq!(saved, computed, "imagem diferente da referência");
}

/// JSON simples `{"chave": "valor"}` sem dependência.
fn serde_like_json(m: &BTreeMap<String, String>) -> String {
    let mut s = String::from("{\n");
    let n = m.len();
    for (i, (k, v)) in m.iter().enumerate() {
        s.push_str(&format!(
            "  \"{k}\": \"{v}\"{}\n",
            if i + 1 < n { "," } else { "" }
        ));
    }
    s.push_str("}\n");
    s
}

fn parse_simple_json(t: &str) -> BTreeMap<String, String> {
    t.lines()
        .filter_map(|l| {
            let l = l.trim().trim_end_matches(',');
            let (k, v) = l.split_once(':')?;
            Some((
                k.trim().trim_matches('"').to_owned(),
                v.trim().trim_matches('"').to_owned(),
            ))
        })
        .collect()
}
