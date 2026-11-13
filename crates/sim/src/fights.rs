//! Matriz de lutas isoladas (docs/12, seção 4.1).

use game::combat::EndReason;
use game::content::ATTR_COUNT;
use game::{
    Action, Archetype, Content, Equipment, FightState, Fighter, ItemKind, OpponentRef, Side,
};
use std::fmt::Write as _;

/// Builds avaliadas: equilibrada + uma especializada por atributo.
pub const BUILDS: [(&str, [u16; ATTR_COUNT]); 7] = [
    ("equilibrada", [1, 1, 1, 1, 1, 1]),
    ("forca", [5, 1, 1, 1, 1, 1]),
    ("tecnica", [1, 5, 1, 1, 1, 1]),
    ("agilidade", [1, 1, 5, 1, 1, 1]),
    ("defesa", [1, 1, 1, 5, 1, 1]),
    ("vitalidade", [1, 1, 1, 1, 5, 1]),
    ("folego", [1, 1, 1, 1, 1, 5]),
];

/// Build especializada de Defesa + Vitalidade (meta B3).
pub const TANK_BUILD: [u16; ATTR_COUNT] = [1, 1, 1, 3, 3, 1];

/// Níveis-base dos cenários de diferença de nível.
pub const BASE_LEVELS: [u8; 3] = [4, 9, 15];

/// Arquétipos de oponente.
pub const OPPONENT_ARCHETYPES: [Archetype; 3] = [
    Archetype::Aggressive,
    Archetype::Defensive,
    Archetype::Opportunist,
];

/// Build típica de cada arquétipo de oponente.
pub fn archetype_build(a: Archetype) -> [u16; ATTR_COUNT] {
    match a {
        Archetype::Aggressive => [3, 1, 1, 1, 2, 2],
        Archetype::Defensive => [1, 1, 1, 3, 3, 2],
        Archetype::Opportunist => [1, 3, 3, 1, 1, 2],
        Archetype::Balanced => [1, 1, 1, 1, 1, 1],
    }
}

/// Equipamento esperado para um nível: o tier cresce a cada 4 níveis; a armadura acompanha com um
/// tier de atraso e o escudo com dois.
pub fn equipment_for_level(content: &Content, level: u8) -> Equipment {
    let tier = usize::from(level / 4);
    let pick = |kind: ItemKind, lag: usize| -> Option<game::ItemId> {
        let mut items: Vec<_> = content.items.iter().filter(|i| i.kind == kind).collect();
        items.sort_by_key(|i| (i.price, i.id));
        let t = tier.checked_sub(lag)?;
        items.get(t.min(items.len().checked_sub(1)?)).map(|i| i.id)
    };
    Equipment {
        weapon: pick(ItemKind::Weapon, 0),
        armor: pick(ItemKind::Armor, 1),
        shield: pick(ItemKind::Shield, 2),
    }
}

/// Cria um lutador de simulação.
pub fn make_fighter(content: &Content, build: [u16; ATTR_COUNT], level: u8) -> Fighter {
    Fighter {
        name: "sim".into(),
        level,
        attrs: Fighter::distribute(content, build, level),
        equipment: equipment_for_level(content, level),
    }
}

/// Tipo de cenário.
#[derive(Debug, Clone, Copy)]
pub enum Scenario {
    /// Jogador equilibrado contra oponente de um arquétipo, com diferença de nível.
    LevelDiff {
        /// Nível do jogador.
        base: u8,
        /// Nível do oponente − nível do jogador.
        diff: i8,
        /// Arquétipo do oponente.
        archetype: Archetype,
    },
    /// Build contra build, mesmo nível, ambos com IA equilibrada.
    Mirror {
        /// Índice da build do jogador em [`BUILDS`].
        a: usize,
        /// Índice da build do oponente em [`BUILDS`].
        b: usize,
    },
    /// Build Defesa + Vitalidade contra equilibrada (meta B3).
    Tank,
}

/// Resultado agregado de um cenário.
#[derive(Debug, Clone, Default)]
pub struct ScenarioResult {
    /// Rótulo do cenário.
    pub label: String,
    /// Lutas.
    pub fights: u32,
    /// Vitórias do jogador.
    pub wins: u32,
    /// Soma de turnos.
    pub turns_sum: u64,
    /// Soma dos quadrados dos turnos.
    pub turns_sq_sum: u64,
    /// Lutas que chegaram à exaustão.
    pub exhausted: u32,
    /// Lutas decididas no limite de turnos.
    pub turn_limit: u32,
    /// Ações do oponente por tipo.
    pub opp_actions: [u64; 8],
    /// Ações do jogador por tipo.
    pub player_actions: [u64; 8],
    /// Arquétipo do oponente (quando aplicável).
    pub opp_archetype: Option<Archetype>,
    /// Diferença de nível (quando aplicável).
    pub diff: Option<i8>,
    /// Builds (quando aplicável).
    pub mirror: Option<(usize, usize)>,
    /// Cenário Tank.
    pub tank: bool,
}

/// Lista de cenários, em ordem fixa.
pub fn scenarios() -> Vec<Scenario> {
    let mut v = Vec::new();
    for base in BASE_LEVELS {
        for diff in -5i8..=5 {
            for archetype in OPPONENT_ARCHETYPES {
                v.push(Scenario::LevelDiff {
                    base,
                    diff,
                    archetype,
                });
            }
        }
    }
    for a in 0..BUILDS.len() {
        for b in 0..BUILDS.len() {
            v.push(Scenario::Mirror { a, b });
        }
    }
    v.push(Scenario::Tank);
    v
}

/// Joga uma luta inteira com IA nos dois lados. Retorna a luta encerrada.
pub fn play(content: &Content, mut fight: FightState) -> FightState {
    let mut events = Vec::new();
    while fight.outcome.is_none() {
        let p = game::ai::choose_action(content, &mut fight, Side::Player);
        let o = game::ai::choose_action(content, &mut fight, Side::Opponent);
        events.clear();
        fight.play_round(content, p, o, &mut events);
    }
    fight
}

fn run_scenario(content: &Content, s: Scenario, n: u32) -> ScenarioResult {
    let max_level = content.rules.progression.max_level;
    let (label, player, p_arch, opp, o_arch) = match s {
        Scenario::LevelDiff {
            base,
            diff,
            archetype,
        } => {
            let ol = (i16::from(base) + i16::from(diff)).clamp(1, i16::from(max_level)) as u8;
            (
                format!("nivel{base:02}_dif{diff:+}_{archetype:?}"),
                make_fighter(content, BUILDS[0].1, base),
                Archetype::Balanced,
                make_fighter(content, archetype_build(archetype), ol),
                archetype,
            )
        }
        Scenario::Mirror { a, b } => (
            format!("build_{}_vs_{}", BUILDS[a].0, BUILDS[b].0),
            make_fighter(content, BUILDS[a].1, 9),
            Archetype::Balanced,
            make_fighter(content, BUILDS[b].1, 9),
            Archetype::Balanced,
        ),
        Scenario::Tank => (
            "tank_vs_equilibrada".into(),
            make_fighter(content, TANK_BUILD, 9),
            Archetype::Defensive,
            make_fighter(content, BUILDS[0].1, 9),
            Archetype::Balanced,
        ),
    };
    let mut r = ScenarioResult {
        label,
        ..Default::default()
    };
    match s {
        Scenario::LevelDiff {
            diff, archetype, ..
        } => {
            r.diff = Some(diff);
            r.opp_archetype = Some(archetype);
        }
        Scenario::Mirror { a, b } => r.mirror = Some((a, b)),
        Scenario::Tank => r.tank = true,
    }
    let exhaustion = content.rules.fight_end.exhaustion_turn;
    for seed in 1..=u64::from(n) {
        let fight = FightState::new(
            content,
            &player,
            p_arch,
            &opp,
            o_arch,
            OpponentRef {
                league: 0,
                index: 0,
            },
            false,
            seed,
        );
        let done = play(content, fight);
        let Some(out) = done.outcome else { continue };
        r.fights += 1;
        if out.winner == Side::Player {
            r.wins += 1;
        }
        let t = u64::from(out.turns);
        r.turns_sum += t;
        r.turns_sq_sum += t * t;
        if out.turns >= exhaustion {
            r.exhausted += 1;
        }
        if out.reason == EndReason::TurnLimit {
            r.turn_limit += 1;
        }
        for (i, a) in Action::ALL.iter().enumerate() {
            r.opp_actions[i] += u64::from(done.fighters[1].actions_used[a.index()]);
            r.player_actions[i] += u64::from(done.fighters[0].actions_used[a.index()]);
        }
    }
    r
}

/// Roda todos os cenários em paralelo. O resultado não depende da ordem das threads.
pub fn run_matrix(content: &Content, n: u32) -> Vec<ScenarioResult> {
    let list = scenarios();
    let threads = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1)
        .clamp(1, 16);
    let mut results: Vec<Option<ScenarioResult>> = vec![None; list.len()];
    std::thread::scope(|scope| {
        let chunks: Vec<_> = results.chunks_mut(list.len().div_ceil(threads)).collect();
        let mut start = 0;
        for chunk in chunks {
            let base = start;
            start += chunk.len();
            let list = &list;
            scope.spawn(move || {
                for (k, slot) in chunk.iter_mut().enumerate() {
                    *slot = Some(run_scenario(content, list[base + k], n));
                }
            });
        }
    });
    results.into_iter().flatten().collect()
}

/// CSV determinístico.
pub fn to_csv(results: &[ScenarioResult]) -> String {
    let mut s = String::from(
        "cenario,lutas,vitorias,turnos_soma,turnos_quad_soma,exaustao,limite,acoes_oponente,acoes_jogador\n",
    );
    for r in results {
        let join = |a: &[u64; 8]| a.iter().map(u64::to_string).collect::<Vec<_>>().join(" ");
        let _ = writeln!(
            s,
            "{},{},{},{},{},{},{},{},{}",
            r.label,
            r.fights,
            r.wins,
            r.turns_sum,
            r.turns_sq_sum,
            r.exhausted,
            r.turn_limit,
            join(&r.opp_actions),
            join(&r.player_actions)
        );
    }
    s
}

/// Luta equilibrada × agressivo no nível `level`, turno a turno (para depuração de balanceamento).
pub fn trace(content: &Content, seed: u64, level: u8) -> String {
    let player = make_fighter(content, BUILDS[0].1, level);
    let opp = make_fighter(content, archetype_build(Archetype::Aggressive), level);
    let mut fight = FightState::new(
        content,
        &player,
        Archetype::Balanced,
        &opp,
        Archetype::Aggressive,
        OpponentRef {
            league: 0,
            index: 0,
        },
        false,
        seed,
    );
    let mut s = String::new();
    let st = |f: &FightState, side: Side| {
        let c = f.fighter(side);
        format!(
            "hp {}/{} fol {}/{} pos {}",
            c.hp, c.stats.hp_max, c.stamina, c.stamina_cap, c.pos
        )
    };
    let _ = writeln!(s, "jogador {:?}", fight.fighters[0].stats);
    let _ = writeln!(s, "oponente {:?}", fight.fighters[1].stats);
    let mut events = Vec::new();
    while fight.outcome.is_none() {
        let p = game::ai::choose_action(content, &mut fight, Side::Player);
        let o = game::ai::choose_action(content, &mut fight, Side::Opponent);
        events.clear();
        fight.play_round(content, p, o, &mut events);
        let _ = writeln!(
            s,
            "t{:02} J:{:?} O:{:?} | J[{}] O[{}] | {:?}",
            fight.turn,
            p,
            o,
            st(&fight, Side::Player),
            st(&fight, Side::Opponent),
            events
                .iter()
                .filter(|e| matches!(
                    e,
                    game::FightEvent::Hit { .. } | game::FightEvent::Miss { .. }
                ))
                .collect::<Vec<_>>()
        );
    }
    let _ = writeln!(s, "{:?}", fight.outcome);
    s
}
