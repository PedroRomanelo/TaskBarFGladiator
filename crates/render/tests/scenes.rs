//! Golden images: hash dos pixels de cada etapa do palco em várias escalas e temas, mais invariantes
//! de layout (nada sai do palco, o chão encosta na base, todo botão é clicável).
//!
//! Regenerar as referências após mudança visual deliberada:
//! `UPDATE_GOLDEN=1 cargo test -p render --test scenes`

mod common;

use render::scene::{STAGE_HEIGHT, STAGE_WIDTH};
use render::{HitId, Phase, Surface, draw_stage};
use std::collections::BTreeMap;
use std::path::PathBuf;

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

fn surface(scale: i32) -> Surface {
    Surface::new(
        (STAGE_WIDTH * scale) as usize,
        (STAGE_HEIGHT * scale) as usize,
    )
}

#[test]
fn golden_images() {
    let mut computed = BTreeMap::new();
    let mut hits = Vec::new();
    for (name, view, hover) in common::scenes() {
        for scale in [1, 2, 3, 5] {
            for (theme, t) in common::themes() {
                let mut s = surface(scale);
                draw_stage(&mut s, &view, theme, hover, &mut hits);
                computed.insert(format!("{name}_x{scale}_{t}"), hash(s.pixels()));
            }
        }
    }
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/golden/stage.json");
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

#[test]
fn stage_rests_on_its_bottom_edge_and_keeps_the_air_clear() {
    let mut hits = Vec::new();
    for (name, view, hover) in common::scenes() {
        for (theme, t) in common::themes() {
            let scale = 3;
            let mut s = surface(scale);
            draw_stage(&mut s, &view, theme, hover, &mut hits);
            let (w, h) = (s.width() as i32, s.height() as i32);
            // A última linha é a faixa de status, opaca de ponta a ponta: é ela que encosta na
            // barra de tarefas e serve de alça para arrastar.
            for x in 0..w {
                let a = s.get(x, h - 1).unwrap_or(0) >> 24;
                assert!(a >= 200, "{name}/{t}: base transparente em x={x}");
            }
            // O palco não tem fundo: a maior parte da área acima do chão é transparente, para não
            // esconder (nem bloquear cliques em) o que está atrás.
            let air_rows = 48 * scale;
            let clear = (0..air_rows)
                .flat_map(|y| (0..w).map(move |x| (x, y)))
                .filter(|&(x, y)| s.get(x, y) == Some(0))
                .count() as i32;
            assert!(
                clear * 100 / (w * air_rows) >= 55,
                "{name}/{t}: só {}% do ar está livre",
                clear * 100 / (w * air_rows)
            );
        }
    }
}

#[test]
fn every_hit_region_is_inside_the_stage_and_drawn() {
    let mut hits = Vec::new();
    for (name, view, hover) in common::scenes() {
        let (theme, _) = common::themes()[0];
        for scale in [1, 2, 4] {
            let mut s = surface(scale);
            draw_stage(&mut s, &view, theme, hover, &mut hits);
            let (w, h) = (s.width() as i32, s.height() as i32);
            for hit in &hits {
                assert!(
                    hit.x >= 0 && hit.y >= 0 && hit.x + hit.w <= w && hit.y + hit.h <= h,
                    "{name} x{scale}: região fora do palco: {hit:?}"
                );
                // Pixels transparentes deixam o clique passar para a janela de trás: o centro de
                // todo botão precisa estar desenhado.
                let center = s.get(hit.x + hit.w / 2, hit.y + hit.h / 2).unwrap_or(0);
                assert!(
                    center >> 24 > 0,
                    "{name} x{scale}: botão sem pixel: {hit:?}"
                );
            }
            // Cada etapa tem o seu botão principal; a luta manual tem as oito ações.
            let has = |id: HitId| hits.iter().any(|x| x.id == id);
            match &view.phase {
                Phase::Hub { .. } => assert!(has(HitId::Fight) && has(HitId::OpenPanel), "{name}"),
                Phase::Result { .. } => assert!(has(HitId::Continue), "{name}"),
                Phase::Fight {
                    buttons: Some(_), ..
                } => {
                    for i in 0..8 {
                        assert!(has(HitId::Action(i)), "{name}: ação {i} sem botão");
                    }
                }
                Phase::Fight { buttons: None, .. } => assert!(has(HitId::TogglePause), "{name}"),
            }
        }
    }
}

#[test]
fn action_buttons_never_overlap_each_other_or_the_fighters_axis() {
    // Em qualquer par de posições válidas, os botões não se sobrepõem e não cobrem o eixo dos
    // lutadores.
    let mut hits = Vec::new();
    let (theme, _) = common::themes()[0];
    let (_, base, _) = common::scenes().swap_remove(1);
    for p in 0..9 {
        for o in (p + 1)..10 {
            let mut view = base.clone();
            view.player.pos_pm = p * 1000;
            view.opponent.pos_pm = o * 1000;
            let mut s = surface(1);
            draw_stage(&mut s, &view, theme, None, &mut hits);
            let actions: Vec<_> = hits
                .iter()
                .filter(|h| matches!(h.id, HitId::Action(_)))
                .collect();
            assert_eq!(actions.len(), 8);
            for (i, a) in actions.iter().enumerate() {
                assert!(
                    a.x >= 0 && a.x + a.w <= STAGE_WIDTH,
                    "jogador {p}, oponente {o}: botão fora: {a:?}"
                );
                for b in &actions[i + 1..] {
                    let apart = a.x + a.w <= b.x
                        || b.x + b.w <= a.x
                        || a.y + a.h <= b.y
                        || b.y + b.h <= a.y;
                    assert!(apart, "jogador {p}, oponente {o}: {a:?} sobre {b:?}");
                }
                for cell in [p, o] {
                    let axis = 15 + cell * 10 + 5;
                    assert!(
                        axis < a.x || axis >= a.x + a.w,
                        "jogador {p}, oponente {o}: botão sobre o lutador da casa {cell}"
                    );
                }
            }
        }
    }
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
