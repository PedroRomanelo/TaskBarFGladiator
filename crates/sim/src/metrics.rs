//! Cálculo das métricas da seção 3 de `docs/12-balanceamento-de-referencia.md`.

use crate::Scale;
use crate::career::{self, CareerResult};
use crate::fights::{self, BUILDS, ScenarioResult};
use game::{Action, Archetype, Content, FightState, OpponentRef, Side};
use std::collections::BTreeMap;
use std::fmt::Write as _;

/// Segundos de animação por rodada no modo automático (para a meta P6).
pub const SECONDS_PER_ROUND: f64 = 1.5;

fn sum<'a>(rs: impl Iterator<Item = &'a ScenarioResult>) -> ScenarioResult {
    let mut t = ScenarioResult::default();
    for r in rs {
        t.fights += r.fights;
        t.wins += r.wins;
        t.turns_sum += r.turns_sum;
        t.turns_sq_sum += r.turns_sq_sum;
        t.exhausted += r.exhausted;
        t.turn_limit += r.turn_limit;
        for i in 0..8 {
            t.opp_actions[i] += r.opp_actions[i];
            t.player_actions[i] += r.player_actions[i];
        }
    }
    t
}

fn mean_turns(r: &ScenarioResult) -> f64 {
    if r.fights == 0 {
        0.0
    } else {
        r.turns_sum as f64 / f64::from(r.fights)
    }
}

fn std_turns(r: &ScenarioResult) -> f64 {
    if r.fights == 0 {
        return 0.0;
    }
    let n = f64::from(r.fights);
    let m = r.turns_sum as f64 / n;
    (r.turns_sq_sum as f64 / n - m * m).max(0.0).sqrt()
}

fn pct(part: u32, total: u32) -> f64 {
    if total == 0 {
        0.0
    } else {
        f64::from(part) * 100.0 / f64::from(total)
    }
}

fn median(mut v: Vec<f64>) -> f64 {
    if v.is_empty() {
        return 0.0;
    }
    v.sort_by(f64::total_cmp);
    v[v.len() / 2]
}

/// Calcula todas as métricas.
pub fn compute(content: &Content, scale: Scale) -> BTreeMap<String, f64> {
    let results = fights::run_matrix(content, scale.fights);
    let careers = career::run_many(content, scale.careers);
    let mut m = BTreeMap::new();

    let by_diff = |d: i8| sum(results.iter().filter(|r| r.diff == Some(d)));
    let d0 = by_diff(0);
    m.insert("C1_turnos_medios".into(), mean_turns(&d0));
    m.insert("C2_vitoria_mesmo_nivel_pct".into(), pct(d0.wins, d0.fights));
    for (id, d) in [
        ("C3_vitoria_1_abaixo_pct", -1i8),
        ("C4_vitoria_3_abaixo_pct", -3),
        ("C5_vitoria_1_acima_pct", 1),
        ("C6_vitoria_3_acima_pct", 3),
        ("C7_vitoria_5_acima_pct", 5),
    ] {
        let r = by_diff(d);
        m.insert(id.into(), pct(r.wins, r.fights));
    }
    let near = sum(results
        .iter()
        .filter(|r| r.diff.is_some_and(|d| (-1..=1).contains(&d))));
    m.insert("C8_exaustao_pct".into(), pct(near.exhausted, near.fights));
    m.insert(
        "C9_limite_turnos_pct".into(),
        pct(near.turn_limit, near.fights),
    );

    let aggressive = sum(results
        .iter()
        .filter(|r| r.opp_archetype == Some(Archetype::Aggressive)));
    let attacks: u64 = [Action::Light, Action::Medium, Action::Heavy]
        .iter()
        .map(|a| aggressive.opp_actions[a.index()])
        .sum();
    let heavy = aggressive.opp_actions[Action::Heavy.index()];
    m.insert(
        "C10_pesado_entre_ataques_agressivo_pct".into(),
        if attacks == 0 {
            0.0
        } else {
            heavy as f64 * 100.0 / attacks as f64
        },
    );
    let mut max_share = 0.0f64;
    for a in fights::OPPONENT_ARCHETYPES {
        let r = sum(results.iter().filter(|r| r.opp_archetype == Some(a)));
        let total: u64 = r.opp_actions.iter().sum();
        if let Some(&mx) = r.opp_actions.iter().max()
            && total > 0
        {
            max_share = max_share.max(mx as f64 * 100.0 / total as f64);
        }
    }
    m.insert("C11_acao_mais_usada_pct".into(), max_share);
    m.insert("C12_desvio_turnos".into(), std_turns(&d0));

    // Builds.
    let n = BUILDS.len();
    let mut win = vec![vec![0.0f64; n]; n];
    for r in &results {
        if let Some((a, b)) = r.mirror {
            win[a][b] = pct(r.wins, r.fights);
        }
    }
    let specialized = 1..n;
    let mut b1 = 0.0f64;
    let mut b2 = 100.0f64;
    for a in specialized.clone() {
        let others: Vec<f64> = specialized
            .clone()
            .filter(|&b| b != a)
            .map(|b| win[a][b])
            .collect();
        let min = others.iter().copied().fold(f64::INFINITY, f64::min);
        let max = others.iter().copied().fold(0.0, f64::max);
        b1 = b1.max(min);
        b2 = b2.min(max);
    }
    m.insert("B1_build_dominante_min_vitoria_pct".into(), b1);
    m.insert("B2_build_fraca_max_vitoria_pct".into(), b2);
    let tank = sum(results.iter().filter(|r| r.tank));
    m.insert(
        "B3_tank_exaustao_pct".into(),
        pct(tank.exhausted, tank.fights),
    );
    m.insert(
        "B4_chance_acertar_agil_pct".into(),
        agility_hit_chance(content),
    );

    careers_metrics(&careers, content, &mut m);
    m
}

/// Chance (%) de um lutador equilibrado acertar um ataque médio em um lutador com a build de
/// Agilidade, ambos no nível 15.
fn agility_hit_chance(content: &Content) -> f64 {
    let attacker = fights::make_fighter(content, BUILDS[0].1, 15);
    let defender = fights::make_fighter(content, BUILDS[3].1, 15);
    let mut f = FightState::new(
        content,
        &attacker,
        Archetype::Balanced,
        &defender,
        Archetype::Balanced,
        OpponentRef {
            league: 0,
            index: 0,
        },
        false,
        1,
    );
    f.fighters[1].pos = f.fighters[0].pos + 1;
    f64::from(f.hit_chance(content, Side::Player, Action::Medium)) / 10.0
}

fn careers_metrics(careers: &[CareerResult], content: &Content, m: &mut BTreeMap<String, f64>) {
    let completed: Vec<&CareerResult> = careers.iter().filter(|c| c.completed).collect();
    m.insert(
        "P0_carreiras_concluidas_pct".into(),
        pct(completed.len() as u32, careers.len() as u32),
    );
    m.insert(
        "P1_nivel_ao_vencer_campeao".into(),
        median(completed.iter().map(|c| f64::from(c.level)).collect()),
    );
    m.insert(
        "P2_lutas_ate_campeao".into(),
        median(completed.iter().map(|c| f64::from(c.fights)).collect()),
    );
    m.insert(
        "P3_vitorias_por_item".into(),
        median(
            completed
                .iter()
                .map(|c| f64::from(c.wins) / f64::from(c.items_bought.max(1)))
                .collect(),
        ),
    );
    m.insert(
        "P4_maior_seq_derrotas".into(),
        median(
            completed
                .iter()
                .map(|c| f64::from(c.max_loss_streak))
                .collect(),
        ),
    );
    let max_price = content
        .items
        .iter()
        .map(|i| i.price)
        .max()
        .unwrap_or(1)
        .max(1);
    m.insert(
        "P5_ouro_sem_uso_em_itens_caros".into(),
        median(
            completed
                .iter()
                .map(|c| f64::from(c.unused_gold) / f64::from(max_price))
                .collect(),
        ),
    );
    m.insert(
        "P6_horas_ate_campeao".into(),
        median(
            completed
                .iter()
                .map(|c| c.rounds as f64 * SECONDS_PER_ROUND / 3600.0)
                .collect(),
        ),
    );
}

/// Tabela Markdown; com `baseline`, inclui a coluna de comparação.
pub fn to_table(now: &BTreeMap<String, f64>, baseline: Option<&BTreeMap<String, f64>>) -> String {
    let mut s = String::new();
    match baseline {
        None => {
            s.push_str("| Métrica | Valor |\n|---|---|\n");
            for (k, v) in now {
                let _ = writeln!(s, "| {k} | {v:.3} |");
            }
        }
        Some(b) => {
            s.push_str("| Métrica | Baseline | Atual | Diferença |\n|---|---|---|---|\n");
            for (k, v) in now {
                let old = b.get(k).copied();
                let diff = old
                    .map(|o| format!("{:+.3}", v - o))
                    .unwrap_or_else(|| "nova".into());
                let old = old.map(|o| format!("{o:.3}")).unwrap_or_else(|| "—".into());
                let _ = writeln!(s, "| {k} | {old} | {v:.3} | {diff} |");
            }
        }
    }
    s
}

/// JSON das métricas (valores arredondados a 6 casas para estabilidade textual).
pub fn to_json(m: &BTreeMap<String, f64>) -> String {
    let rounded: BTreeMap<&String, f64> = m
        .iter()
        .map(|(k, v)| (k, (v * 1e6).round() / 1e6))
        .collect();
    serde_json::to_string_pretty(&rounded).unwrap_or_default() + "\n"
}
