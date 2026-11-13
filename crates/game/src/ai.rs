//! Inteligência artificial por pontuação de utilidade.
//!
//! Para cada ação válida calcula uma pontuação a partir de fatores (dano esperado, risco recebido,
//! fôlego restante, distância preferida, oportunidade) ponderados pelo arquétipo, soma um ruído
//! pequeno e escolhe a maior. Função pura dado o estado do gerador. Ver `docs/11`, seção 8.

use crate::combat::{Action, FightState, Side, Stance};
use crate::content::{Archetype, ArchetypeRules, Content};

/// Pesos do arquétipo.
pub fn archetype_rules(content: &Content, a: Archetype) -> &ArchetypeRules {
    let r = &content.rules.archetypes;
    match a {
        Archetype::Aggressive => &r.aggressive,
        Archetype::Defensive => &r.defensive,
        Archetype::Opportunist => &r.opportunist,
        Archetype::Balanced => &r.balanced,
    }
}

/// Escolhe a ação de `side` usando o arquétipo do combatente e o gerador da luta.
pub fn choose_action(content: &Content, fight: &mut FightState, side: Side) -> Action {
    let archetype = fight.fighter(side).archetype;
    choose_with(content, fight, side, archetype)
}

/// Escolhe a ação de `side` com um arquétipo específico (estratégia do modo automático).
pub fn choose_with(
    content: &Content,
    fight: &mut FightState,
    side: Side,
    archetype: Archetype,
) -> Action {
    let w = archetype_rules(content, archetype).clone();
    let noise_max = content.rules.ai.noise_max;
    let mut best = Action::Defend;
    let mut best_score = i64::MIN;
    for action in Action::ALL {
        if !fight.is_valid(content, side, action) {
            continue;
        }
        let base = score(content, fight, side, action, &w);
        let noise = i64::from(fight.rng.below(noise_max.saturating_add(1)));
        let s = base.saturating_add(noise);
        if s > best_score {
            best_score = s;
            best = action;
        }
    }
    best
}

fn score(
    content: &Content,
    fight: &FightState,
    side: Side,
    action: Action,
    w: &ArchetypeRules,
) -> i64 {
    let me = fight.fighter(side);
    let other = fight.fighter(side.other());
    let dist = i64::from(fight.distance());
    let cost = i64::from(fight.stamina_cost(content, side, action));
    let sta_max = i64::from(me.stats.stamina_max.max(1));
    let sta_after_pm = ((i64::from(me.stamina) - cost).max(0) * 1000) / sta_max;

    // Distância após a ação.
    let new_dist = match action {
        Action::Advance => dist - 1,
        Action::Retreat => dist + 1,
        _ => dist,
    };

    // Dano esperado (por mil de chance × dano médio, com crítico médio).
    let expected = if action.is_attack() {
        let hit = i64::from(fight.hit_chance(content, side, action));
        let dmg = fight.expected_hit_damage(content, side, action);
        let crit_bonus = match action {
            Action::Light => content.rules.actions.light.crit_bonus_pm,
            Action::Medium => content.rules.actions.medium.crit_bonus_pm,
            Action::Heavy => content.rules.actions.heavy.crit_bonus_pm,
            _ => 0,
        };
        let crit = i64::from(me.stats.crit_pm + crit_bonus);
        let crit_mult = i64::from(content.rules.derived.crit_multiplier_pm);
        // dano médio com crítico = dmg × (1 + crit × (mult − 1))
        let avg = dmg * (1_000_000 + crit * (crit_mult - 1000)) / 1_000_000;
        hit * avg / 1000
    } else {
        0
    };

    // Risco: melhor ataque que o oponente poderia fazer a partir da nova distância.
    let threat = best_threat(content, fight, side, new_dist);
    let stance_mult = match action {
        Action::Defend => i64::from(content.rules.actions.defend.damage_taken_pm),
        Action::Recover => i64::from(content.rules.actions.recover.damage_taken_pm),
        Action::Taunt => 1000 + (1000 - i64::from(content.rules.actions.taunt.self_absorb_pm)),
        _ => 1000,
    };
    let risk = threat * stance_mult / 1000;

    // Potencial: melhor dano esperado que eu poderei causar a partir da nova distância no próximo
    // turno, descontado (vale menos que causar dano agora). Sem isso a IA nunca entra no alcance.
    let potential = if action.is_attack() {
        0
    } else {
        best_potential(content, fight, side, new_dist, cost) * content.rules.ai.potential_pm / 1000
    };

    // Distância preferida.
    let pref = i64::from(w.preferred_distance);
    let dist_score = -(new_dist - pref).abs();

    // Oportunidade: oponente vulnerável.
    let other_sta_pm = i64::from(other.stamina) * 1000 / i64::from(other.stats.stamina_max.max(1));
    let vulnerable = other.stance == Stance::Recover || other_sta_pm < 250 || other.hp_pm() < 250;
    let opportunity = if vulnerable && action.is_attack() {
        1
    } else {
        0
    };

    // Escalas: dano e risco em pontos de HP; fôlego restante em décimos do máximo (0 a 10).
    let mut s = w.w_damage * (expected + potential) - w.w_risk * risk
        + w.w_stamina * sta_after_pm / 100
        + w.w_distance * dist_score
        + w.w_opportunity * opportunity;

    // Ajustes específicos por ação.
    let my_sta_pm = i64::from(me.stamina) * 1000 / sta_max;
    match action {
        Action::Heavy if my_sta_pm < w.heavy_min_stamina_pm => s -= 10_000,
        Action::Recover => {
            // Só vale a pena com fôlego faltando; com fôlego cheio é sempre pior que Defender.
            let missing = 1000 - my_sta_pm;
            s += content.rules.ai.recover_value_per_missing_pm * missing / 10 - 20;
        }
        Action::Taunt => {
            if me.taunt_buff {
                s -= 10_000;
            } else {
                let safe = dist >= 3 || other_sta_pm < 250;
                s += if safe {
                    content.rules.ai.taunt_value * w.w_opportunity / 10
                } else {
                    0
                };
            }
        }
        _ => {}
    }
    s
}

/// Maior dano esperado que `side` poderá causar a partir de `dist`, depois de gastar `cost`.
fn best_potential(content: &Content, fight: &FightState, side: Side, dist: i64, cost: i64) -> i64 {
    let a = &content.rules.actions;
    let me = fight.fighter(side);
    let stamina_next = i64::from(me.stamina) - cost + i64::from(me.stats.regen);
    let mut best = 0i64;
    for (action, rules) in [
        (Action::Light, &a.light),
        (Action::Medium, &a.medium),
        (Action::Heavy, &a.heavy),
    ] {
        if dist < i64::from(rules.range_min) || dist > i64::from(rules.range_max) {
            continue;
        }
        if stamina_next < i64::from(fight.stamina_cost(content, side, action)) {
            continue;
        }
        let dmg = fight.expected_hit_damage(content, side, action);
        let base = fight.hit_chance(content, side, action);
        // hit_chance usa a distância atual; corrige a penalidade de distância para `dist`.
        let cur = i64::from(fight.distance()).max(1);
        let adj = i64::from(rules.far_penalty_pm) * (cur - dist.max(1));
        let hit = (i64::from(base) + adj).clamp(
            i64::from(content.rules.hit.min_pm),
            i64::from(content.rules.hit.max_pm),
        );
        best = best.max(hit * dmg / 1000);
    }
    best
}

/// Maior dano esperado que o oponente de `side` consegue causar a partir de `dist`.
fn best_threat(content: &Content, fight: &FightState, side: Side, dist: i64) -> i64 {
    let attacker = side.other();
    let a = &content.rules.actions;
    let mut best = 0i64;
    for (action, rules) in [
        (Action::Light, &a.light),
        (Action::Medium, &a.medium),
        (Action::Heavy, &a.heavy),
    ] {
        if dist < i64::from(rules.range_min) || dist > i64::from(rules.range_max) {
            continue;
        }
        let cost = fight.stamina_cost(content, attacker, action);
        if fight.fighter(attacker).stamina < cost {
            continue;
        }
        let dmg = fight.expected_hit_damage(content, attacker, action);
        let hit = i64::from(fight.hit_chance(content, attacker, action));
        best = best.max(hit * dmg / 1000);
    }
    best
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::combat::OpponentRef;
    use crate::fighter::{Attributes, Equipment, Fighter};

    fn content() -> Content {
        match Content::embedded() {
            Ok(c) => c,
            Err(e) => panic!("{e}"),
        }
    }

    fn fighter() -> Fighter {
        Fighter {
            name: "t".into(),
            level: 1,
            attrs: Attributes([7, 7, 7, 7, 7, 5]),
            equipment: Equipment::default(),
        }
    }

    #[test]
    fn ai_always_returns_valid_action() {
        let c = content();
        for seed in 0..200u64 {
            let mut f = FightState::new(
                &c,
                &fighter(),
                Archetype::Defensive,
                &fighter(),
                Archetype::Aggressive,
                OpponentRef {
                    league: 0,
                    index: 0,
                },
                false,
                seed,
            );
            let mut ev = Vec::new();
            while f.outcome.is_none() {
                let p = choose_action(&c, &mut f, Side::Player);
                assert!(f.is_valid(&c, Side::Player, p));
                let o = choose_action(&c, &mut f, Side::Opponent);
                assert!(f.is_valid(&c, Side::Opponent, o));
                f.play_round(&c, p, o, &mut ev);
            }
        }
    }
}
