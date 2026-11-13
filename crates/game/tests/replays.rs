//! Replays gravados: lutas reais que precisam se repetir exatamente (docs/12, seção 5.2).
//!
//! Para regenerar após uma mudança deliberada de balanceamento:
//! `UPDATE_REPLAYS=1 cargo test -p game --test replays`

use game::content::fnv1a64;
use game::{Archetype, Attr, Content, GameState, Input, Mode, OpponentRef, Screen, step};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct Replay {
    name: String,
    seed: u64,
    strategy: Archetype,
    /// Atributo que recebe todos os pontos livres (`None` = distribuição equilibrada).
    focus: Option<Attr>,
    opponent: u16,
    /// Resultado esperado.
    winner: String,
    turns: u16,
    state_hash: String,
}

fn content() -> Content {
    match Content::embedded() {
        Ok(c) => c,
        Err(e) => panic!("{e}"),
    }
}

fn path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/replays/replays.json")
}

/// Executa um replay e devolve (vencedor, turnos, hash do estado final).
fn run(
    c: &Content,
    seed: u64,
    strategy: Archetype,
    focus: Option<Attr>,
    opponent: u16,
) -> (String, u16, String) {
    let mut s = GameState::new(c, "Replay", seed);
    // Personagem de teste no nível do oponente, com equipamento inicial.
    let level = c.opponent(0, usize::from(opponent)).map_or(1, |o| o.level);
    s.player.level = level;
    s.unspent_points = c.rules.attributes.start_free_points
        + c.rules.attributes.points_per_level * u16::from(level - 1);
    s.league_progress[0] = opponent;
    let mut k = 0usize;
    while s.unspent_points > 0 {
        let attr = focus.unwrap_or(Attr::ALL[k % Attr::ALL.len()]);
        step(c, &mut s, Input::AllocatePoint(attr));
        k += 1;
        if k > 10_000 {
            break;
        }
    }
    step(c, &mut s, Input::SetMode(Mode::Auto(strategy)));
    step(
        c,
        &mut s,
        Input::StartFight(OpponentRef {
            league: 0,
            index: opponent,
        }),
    );
    while matches!(s.screen, Screen::Fight(_)) {
        step(c, &mut s, Input::AutoRound);
    }
    let (winner, turns) = match &s.screen {
        Screen::PostFight(sum) => (format!("{:?}", sum.outcome.winner), sum.outcome.turns),
        other => panic!("estado inesperado: {other:?}"),
    };
    let json = serde_json::to_vec(&s).unwrap_or_default();
    (winner, turns, format!("{:016x}", fnv1a64(&[&json])))
}

fn cases() -> Vec<(String, u64, Archetype, Option<Attr>, u16)> {
    let strategies = [
        Archetype::Balanced,
        Archetype::Aggressive,
        Archetype::Defensive,
        Archetype::Opportunist,
    ];
    let focuses = [
        None,
        Some(Attr::Strength),
        Some(Attr::Agility),
        Some(Attr::Vitality),
        Some(Attr::Defense),
    ];
    let mut v = Vec::new();
    for i in 0..20u64 {
        let strategy = strategies[(i % 4) as usize];
        let focus = focuses[(i % 5) as usize];
        let opponent = (i % 9) as u16;
        v.push((
            format!("replay_{i:02}"),
            1000 + i,
            strategy,
            focus,
            opponent,
        ));
    }
    v
}

#[test]
fn replays_reproduce_exactly() {
    let c = content();
    let computed: Vec<Replay> = cases()
        .into_iter()
        .map(|(name, seed, strategy, focus, opponent)| {
            let (winner, turns, state_hash) = run(&c, seed, strategy, focus, opponent);
            Replay {
                name,
                seed,
                strategy,
                focus,
                opponent,
                winner,
                turns,
                state_hash,
            }
        })
        .collect();
    if std::env::var_os("UPDATE_REPLAYS").is_some() {
        let text = serde_json::to_string_pretty(&computed).unwrap_or_default() + "\n";
        if let Err(e) = std::fs::write(path(), text) {
            panic!("não foi possível gravar os replays: {e}");
        }
        return;
    }
    let saved: Vec<Replay> = match std::fs::read_to_string(path()) {
        Ok(t) => match serde_json::from_str(&t) {
            Ok(v) => v,
            Err(e) => panic!("replays.json inválido: {e}"),
        },
        Err(e) => panic!("replays.json ausente ({e}); rode com UPDATE_REPLAYS=1"),
    };
    assert_eq!(
        saved, computed,
        "replay diferente: houve mudança de balanceamento. Siga docs/12, seção 6."
    );
}

#[test]
fn replay_is_deterministic_within_run() {
    let c = content();
    let a = run(&c, 7, Archetype::Aggressive, None, 3);
    let b = run(&c, 7, Archetype::Aggressive, None, 3);
    assert_eq!(a, b);
}
