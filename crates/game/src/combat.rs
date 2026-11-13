//! Combate por turnos em arena de uma dimensão.
//!
//! Regras em `docs/11-design-do-jogo.md`, seções 2 a 4 e 9. Todas as constantes vêm de
//! [`crate::content::Rules`]. Toda a aritmética é inteira (com intermediários de 64 bits) para que a
//! mesma semente produza o mesmo resultado em qualquer máquina.

use crate::content::{Archetype, AttackRules, Content};
use crate::fighter::{Fighter, Stats};
use crate::rng::Pcg32;
use serde::{Deserialize, Serialize};

/// Lado da arena.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Side {
    /// Jogador (lado esquerdo).
    Player = 0,
    /// Oponente (lado direito).
    Opponent = 1,
}

impl Side {
    /// Índice no vetor de combatentes.
    pub const fn index(self) -> usize {
        self as usize
    }

    /// O outro lado.
    pub const fn other(self) -> Side {
        match self {
            Side::Player => Side::Opponent,
            Side::Opponent => Side::Player,
        }
    }
}

/// Ação de um turno.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Action {
    /// Move uma casa em direção ao oponente.
    Advance = 0,
    /// Move uma casa para longe do oponente.
    Retreat = 1,
    /// Ataque leve.
    Light = 2,
    /// Ataque médio.
    Medium = 3,
    /// Ataque pesado.
    Heavy = 4,
    /// Postura defensiva.
    Defend = 5,
    /// Recupera fôlego, ficando vulnerável.
    Recover = 6,
    /// Provoca: melhora o próximo ataque e reduz a própria absorção.
    Taunt = 7,
}

impl Action {
    /// Todas as ações.
    pub const ALL: [Action; 8] = [
        Action::Advance,
        Action::Retreat,
        Action::Light,
        Action::Medium,
        Action::Heavy,
        Action::Defend,
        Action::Recover,
        Action::Taunt,
    ];

    /// Verdadeiro para ataques.
    pub const fn is_attack(self) -> bool {
        matches!(self, Action::Light | Action::Medium | Action::Heavy)
    }

    /// Índice da ação.
    pub const fn index(self) -> usize {
        self as usize
    }
}

/// Postura ativa.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Stance {
    /// Nenhuma.
    None,
    /// Defendendo: protege contra a próxima ação do oponente.
    Defend,
    /// Recuperando: vulnerável até a própria próxima ação.
    Recover,
}

/// Estado de um combatente durante a luta.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Combatant {
    /// Nome.
    pub name: String,
    /// Nível.
    pub level: u8,
    /// Atributos derivados.
    pub stats: Stats,
    /// Arquétipo de IA (usado quando a IA controla este lado).
    pub archetype: Archetype,
    /// HP atual.
    pub hp: i32,
    /// Fôlego atual.
    pub stamina: i32,
    /// Fôlego máximo atual (cai durante a exaustão).
    pub stamina_cap: i32,
    /// Casa na arena.
    pub pos: u8,
    /// Postura.
    pub stance: Stance,
    /// Próximo ataque recebe o bônus da provocação.
    pub taunt_buff: bool,
    /// Absorção reduzida até a própria próxima ação (efeito colateral da provocação).
    pub taunt_exposed: bool,
    /// Dano total causado nesta luta.
    pub damage_dealt: i32,
    /// Contagem de ações executadas, por tipo (para o simulador).
    pub actions_used: [u16; 8],
}

impl Combatant {
    fn new(fighter: &Fighter, archetype: Archetype, pos: u8, content: &Content) -> Self {
        let stats = fighter.stats(content);
        Self {
            name: fighter.name.clone(),
            level: fighter.level,
            stats,
            archetype,
            hp: stats.hp_max,
            stamina: stats.stamina_max,
            stamina_cap: stats.stamina_max,
            pos,
            stance: Stance::None,
            taunt_buff: false,
            taunt_exposed: false,
            damage_dealt: 0,
            actions_used: [0; 8],
        }
    }

    /// HP restante em ‰ do máximo.
    pub fn hp_pm(&self) -> i64 {
        i64::from(self.hp) * 1000 / i64::from(self.stats.hp_max.max(1))
    }
}

/// Motivo do fim da luta.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EndReason {
    /// HP de um lado chegou a 0.
    Knockout,
    /// Limite de turnos atingido.
    TurnLimit,
}

/// Resultado da luta.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Outcome {
    /// Vencedor.
    pub winner: Side,
    /// Motivo.
    pub reason: EndReason,
    /// Turnos completos.
    pub turns: u16,
}

/// Evento produzido pela resolução de um turno (consumido por animação, textos e simulador).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FightEvent {
    /// Quem agiu primeiro na rodada.
    Initiative(Side),
    /// Ação escolhida tornou-se inválida no momento da execução e foi trocada por Defender.
    Converted {
        /// Lado.
        side: Side,
        /// Ação original.
        from: Action,
    },
    /// Ação executada.
    Acted {
        /// Lado.
        side: Side,
        /// Ação.
        action: Action,
    },
    /// Movimento.
    Moved {
        /// Lado.
        side: Side,
        /// Nova casa.
        to: u8,
    },
    /// Acerto.
    Hit {
        /// Atacante.
        attacker: Side,
        /// Dano aplicado.
        damage: i32,
        /// Crítico.
        critical: bool,
    },
    /// Erro.
    Miss {
        /// Atacante.
        attacker: Side,
    },
    /// Fôlego recuperado pela ação Recuperar.
    Recovered {
        /// Lado.
        side: Side,
        /// Quantidade.
        amount: i32,
    },
    /// Fim da luta.
    Ended(Outcome),
}

/// Referência ao oponente na liga.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct OpponentRef {
    /// Índice da liga.
    pub league: u16,
    /// Índice do oponente na liga.
    pub index: u16,
}

/// Estado completo de uma luta.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FightState {
    /// Oponente enfrentado.
    pub opponent: OpponentRef,
    /// Luta de treino (oponente já vencido).
    pub rematch: bool,
    /// Semente usada para criar o gerador desta luta.
    pub seed: u64,
    /// Turnos completos.
    pub turn: u16,
    /// Combatentes: `[jogador, oponente]`.
    pub fighters: [Combatant; 2],
    /// Gerador da luta.
    pub rng: Pcg32,
    /// Resultado, quando a luta terminou.
    pub outcome: Option<Outcome>,
}

/// Fluxo de sequência do PCG32 reservado para lutas.
const FIGHT_STREAM: u64 = 0x5eed_f167;

impl FightState {
    /// Cria uma luta.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        content: &Content,
        player: &Fighter,
        player_archetype: Archetype,
        opponent: &Fighter,
        opponent_archetype: Archetype,
        opponent_ref: OpponentRef,
        rematch: bool,
        seed: u64,
    ) -> Self {
        let a = &content.rules.arena;
        Self {
            opponent: opponent_ref,
            rematch,
            seed,
            turn: 0,
            fighters: [
                Combatant::new(player, player_archetype, a.player_start, content),
                Combatant::new(opponent, opponent_archetype, a.opponent_start, content),
            ],
            rng: Pcg32::new(seed, FIGHT_STREAM),
            outcome: None,
        }
    }

    /// Combatente de um lado.
    pub fn fighter(&self, side: Side) -> &Combatant {
        &self.fighters[side.index()]
    }

    /// Distância entre os combatentes.
    pub fn distance(&self) -> u8 {
        self.fighters[0].pos.abs_diff(self.fighters[1].pos)
    }

    fn attack_rules(content: &Content, action: Action) -> Option<&AttackRules> {
        let a = &content.rules.actions;
        match action {
            Action::Light => Some(&a.light),
            Action::Medium => Some(&a.medium),
            Action::Heavy => Some(&a.heavy),
            _ => None,
        }
    }

    /// Custo de fôlego de uma ação para um lado.
    pub fn stamina_cost(&self, content: &Content, side: Side, action: Action) -> i32 {
        let a = &content.rules.actions;
        let c = self.fighter(side);
        match action {
            Action::Advance => a.advance.stamina,
            Action::Retreat => a.retreat.stamina,
            Action::Light | Action::Medium | Action::Heavy => {
                Self::attack_rules(content, action).map_or(0, |r| r.stamina) + c.stats.weight
            }
            Action::Defend | Action::Recover => 0,
            Action::Taunt => a.taunt.stamina,
        }
    }

    /// Verifica se uma ação é válida agora.
    pub fn is_valid(&self, content: &Content, side: Side, action: Action) -> bool {
        if self.outcome.is_some() {
            return false;
        }
        let me = self.fighter(side);
        if me.stamina < self.stamina_cost(content, side, action) {
            return false;
        }
        let dist = self.distance();
        let cells = content.rules.arena.cells;
        match action {
            Action::Advance => dist > 1,
            Action::Retreat => match side {
                Side::Player => me.pos > 0,
                Side::Opponent => me.pos + 1 < cells,
            },
            Action::Light | Action::Medium | Action::Heavy => Self::attack_rules(content, action)
                .is_some_and(|r| dist >= r.range_min && dist <= r.range_max),
            Action::Defend | Action::Recover => true,
            Action::Taunt => true,
        }
    }

    /// Ações válidas agora (Defender e Recuperar estão sempre presentes).
    pub fn valid_actions(&self, content: &Content, side: Side) -> Vec<Action> {
        Action::ALL
            .iter()
            .copied()
            .filter(|&a| self.is_valid(content, side, a))
            .collect()
    }

    /// Chance de acerto (‰) de um ataque de `side` com a distância atual.
    pub fn hit_chance(&self, content: &Content, side: Side, action: Action) -> i32 {
        let Some(r) = Self::attack_rules(content, action) else {
            return 0;
        };
        let h = &content.rules.hit;
        let me = self.fighter(side);
        let other = self.fighter(side.other());
        let dist = i32::from(self.distance()).max(1);
        let acc = r.accuracy_pm + me.stats.accuracy_pm - r.far_penalty_pm * (dist - 1)
            + (me.stats.technique - other.stats.agility) * h.per_point_pm;
        acc.clamp(h.min_pm, h.max_pm)
    }

    /// Dano médio esperado de um acerto (sem crítico e sem variação), já com a redução do defensor.
    pub fn expected_hit_damage(&self, content: &Content, side: Side, action: Action) -> i64 {
        let Some(r) = Self::attack_rules(content, action) else {
            return 0;
        };
        let me = self.fighter(side);
        let other = self.fighter(side.other());
        let mut raw = i64::from(me.stats.damage) * i64::from(r.damage_pm) / 1000;
        if me.taunt_buff {
            raw = raw * i64::from(content.rules.actions.taunt.next_damage_pm) / 1000;
        }
        let reduced = self.reduce(content, other, raw);
        reduced.max(i64::from(content.rules.damage.min_damage))
    }

    fn effective_absorb(content: &Content, c: &Combatant) -> i64 {
        let mut absorb = i64::from(c.stats.absorb);
        if c.taunt_exposed {
            absorb = absorb * i64::from(content.rules.actions.taunt.self_absorb_pm) / 1000;
        }
        absorb.max(0)
    }

    fn reduce(&self, content: &Content, defender: &Combatant, raw: i64) -> i64 {
        let scale = i64::from(content.rules.damage.absorb_scale);
        let mut dmg = raw * scale / (scale + Self::effective_absorb(content, defender));
        dmg = match defender.stance {
            Stance::Defend => dmg * i64::from(content.rules.actions.defend.damage_taken_pm) / 1000,
            Stance::Recover => {
                dmg * i64::from(content.rules.actions.recover.damage_taken_pm) / 1000
            }
            Stance::None => dmg,
        };
        dmg
    }

    /// Resolve uma rodada completa com as ações escolhidas pelos dois lados.
    ///
    /// Ordem: iniciativa (Agilidade + dado; empate → maior Agilidade; persistindo → sorteio);
    /// cada ação é resolvida por completo; se a primeira derrubar o oponente, a segunda não
    /// acontece. Uma ação que ficou inválida por causa da ação anterior vira Defender.
    pub fn play_round(
        &mut self,
        content: &Content,
        player_action: Action,
        opponent_action: Action,
        events: &mut Vec<FightEvent>,
    ) {
        if self.outcome.is_some() {
            return;
        }
        let first = self.roll_initiative(content);
        events.push(FightEvent::Initiative(first));
        let order = [first, first.other()];
        for side in order {
            if self.outcome.is_some() {
                break;
            }
            let chosen = match side {
                Side::Player => player_action,
                Side::Opponent => opponent_action,
            };
            self.execute(content, side, chosen, events);
        }
        if self.outcome.is_none() {
            self.end_round(content, events);
        }
    }

    fn roll_initiative(&mut self, content: &Content) -> Side {
        let die = content.rules.derived.initiative_die;
        let p = self.fighters[0].stats.agility + self.rng.range_inclusive(1, die) as i32;
        let o = self.fighters[1].stats.agility + self.rng.range_inclusive(1, die) as i32;
        if p != o {
            return if p > o { Side::Player } else { Side::Opponent };
        }
        let (pa, oa) = (
            self.fighters[0].stats.agility,
            self.fighters[1].stats.agility,
        );
        if pa != oa {
            return if pa > oa {
                Side::Player
            } else {
                Side::Opponent
            };
        }
        if self.rng.below(2) == 0 {
            Side::Player
        } else {
            Side::Opponent
        }
    }

    fn execute(
        &mut self,
        content: &Content,
        side: Side,
        chosen: Action,
        events: &mut Vec<FightEvent>,
    ) {
        let i = side.index();
        // Efeitos que duram "até a própria próxima ação".
        self.fighters[i].stance = Stance::None;
        self.fighters[i].taunt_exposed = false;

        let action = if self.is_valid(content, side, chosen) {
            chosen
        } else {
            events.push(FightEvent::Converted { side, from: chosen });
            Action::Defend
        };
        let cost = self.stamina_cost(content, side, action);
        self.fighters[i].stamina = (self.fighters[i].stamina - cost).max(0);
        self.fighters[i].actions_used[action.index()] =
            self.fighters[i].actions_used[action.index()].saturating_add(1);
        events.push(FightEvent::Acted { side, action });

        match action {
            Action::Advance | Action::Retreat => {
                let toward = action == Action::Advance;
                let me = &mut self.fighters[i];
                let right = matches!(side, Side::Player) == toward;
                me.pos = if right { me.pos + 1 } else { me.pos - 1 };
                events.push(FightEvent::Moved { side, to: me.pos });
            }
            Action::Light | Action::Medium | Action::Heavy => {
                self.resolve_attack(content, side, action, events);
            }
            Action::Defend => self.fighters[i].stance = Stance::Defend,
            Action::Recover => {
                let me = &mut self.fighters[i];
                let gain = me.stats.regen * content.rules.actions.recover.regen_multiplier;
                let before = me.stamina;
                me.stamina = (me.stamina + gain).min(me.stamina_cap);
                me.stance = Stance::Recover;
                events.push(FightEvent::Recovered {
                    side,
                    amount: me.stamina - before,
                });
            }
            Action::Taunt => {
                let me = &mut self.fighters[i];
                me.taunt_buff = true;
                me.taunt_exposed = true;
            }
        }

        // Defender protege contra exatamente uma ação do oponente.
        let o = side.other().index();
        if self.fighters[o].stance == Stance::Defend {
            self.fighters[o].stance = Stance::None;
        }
    }

    fn resolve_attack(
        &mut self,
        content: &Content,
        side: Side,
        action: Action,
        events: &mut Vec<FightEvent>,
    ) {
        let Some(r) = Self::attack_rules(content, action) else {
            return;
        };
        let rules = &content.rules;
        let chance = self.hit_chance(content, side, action);
        let buffed = self.fighters[side.index()].taunt_buff;
        self.fighters[side.index()].taunt_buff = false;
        if !self.rng.chance(chance) {
            events.push(FightEvent::Miss { attacker: side });
            return;
        }
        let my_stats = self.fighter(side).stats;
        let mut raw = i64::from(my_stats.damage) * i64::from(r.damage_pm) / 1000;
        let variance = self
            .rng
            .range_inclusive(rules.damage.variance_min_pm, rules.damage.variance_max_pm);
        raw = raw * i64::from(variance) / 1000;
        let mut crit_pm = my_stats.crit_pm + r.crit_bonus_pm;
        if buffed {
            crit_pm += rules.actions.taunt.next_crit_bonus_pm;
            raw = raw * i64::from(rules.actions.taunt.next_damage_pm) / 1000;
        }
        let critical = self.rng.chance(crit_pm);
        if critical {
            raw = raw * i64::from(rules.derived.crit_multiplier_pm) / 1000;
        }
        let defender = self.fighter(side.other());
        let dmg = self
            .reduce(content, defender, raw)
            .max(i64::from(rules.damage.min_damage));
        let dmg = i32::try_from(dmg).unwrap_or(i32::MAX);
        let o = side.other().index();
        self.fighters[o].hp = self.fighters[o].hp.saturating_sub(dmg).max(0);
        let me = &mut self.fighters[side.index()];
        me.damage_dealt = me.damage_dealt.saturating_add(dmg);
        events.push(FightEvent::Hit {
            attacker: side,
            damage: dmg,
            critical,
        });
        if self.fighters[o].hp == 0 {
            self.finish(side, EndReason::Knockout, events);
        }
    }

    fn end_round(&mut self, content: &Content, events: &mut Vec<FightEvent>) {
        let fe = &content.rules.fight_end;
        self.turn = self.turn.saturating_add(1);
        for c in &mut self.fighters {
            c.stamina = (c.stamina + c.stats.regen).min(c.stamina_cap);
        }
        if self.turn >= fe.exhaustion_turn {
            let steps = i64::from(self.turn - fe.exhaustion_turn + 1);
            for c in &mut self.fighters {
                let base = i64::from(c.stats.stamina_max);
                let lost = base * i64::from(fe.exhaustion_pm_per_turn) * steps / 1000;
                c.stamina_cap = i32::try_from((base - lost).max(0)).unwrap_or(0);
                c.stamina = c.stamina.min(c.stamina_cap);
            }
        }
        if self.turn >= fe.max_turns {
            let (p, o) = (&self.fighters[0], &self.fighters[1]);
            let winner = match p.hp_pm().cmp(&o.hp_pm()) {
                std::cmp::Ordering::Greater => Side::Player,
                std::cmp::Ordering::Less => Side::Opponent,
                std::cmp::Ordering::Equal => {
                    if o.damage_dealt > p.damage_dealt {
                        Side::Opponent
                    } else {
                        Side::Player
                    }
                }
            };
            self.finish(winner, EndReason::TurnLimit, events);
        }
    }

    fn finish(&mut self, winner: Side, reason: EndReason, events: &mut Vec<FightEvent>) {
        let outcome = Outcome {
            winner,
            reason,
            turns: self.turn,
        };
        self.outcome = Some(outcome);
        events.push(FightEvent::Ended(outcome));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fighter::{Attributes, Equipment};

    fn content() -> Content {
        match Content::embedded() {
            Ok(c) => c,
            Err(e) => panic!("{e}"),
        }
    }

    fn fighter(attrs: [u8; 6]) -> Fighter {
        Fighter {
            name: "t".into(),
            level: 1,
            attrs: Attributes(attrs),
            equipment: Equipment::default(),
        }
    }

    fn fight(c: &Content, seed: u64) -> FightState {
        FightState::new(
            c,
            &fighter([7, 7, 7, 7, 7, 5]),
            Archetype::Balanced,
            &fighter([7, 7, 7, 7, 7, 5]),
            Archetype::Balanced,
            OpponentRef {
                league: 0,
                index: 0,
            },
            false,
            seed,
        )
    }

    #[test]
    fn advance_invalid_when_adjacent_and_retreat_invalid_at_wall() {
        let c = content();
        let mut f = fight(&c, 1);
        f.fighters[0].pos = 4;
        f.fighters[1].pos = 5;
        assert!(!f.is_valid(&c, Side::Player, Action::Advance));
        assert!(!f.is_valid(&c, Side::Opponent, Action::Advance));
        f.fighters[0].pos = 0;
        assert!(!f.is_valid(&c, Side::Player, Action::Retreat));
        f.fighters[1].pos = c.rules.arena.cells - 1;
        assert!(!f.is_valid(&c, Side::Opponent, Action::Retreat));
    }

    #[test]
    fn defend_and_recover_always_valid() {
        let c = content();
        let mut f = fight(&c, 1);
        f.fighters[0].stamina = 0;
        let v = f.valid_actions(&c, Side::Player);
        assert!(v.contains(&Action::Defend));
        assert!(v.contains(&Action::Recover));
    }

    #[test]
    fn invalid_action_at_execution_becomes_defend() {
        let c = content();
        let mut f = fight(&c, 3);
        f.fighters[0].pos = 4;
        f.fighters[1].pos = 5;
        // O oponente recua para a distância 2; o ataque médio do jogador fica fora de alcance se o
        // oponente agir primeiro.
        let mut ev = Vec::new();
        f.fighters[1].stats.agility = 200;
        f.play_round(&c, Action::Medium, Action::Retreat, &mut ev);
        assert!(ev.contains(&FightEvent::Converted {
            side: Side::Player,
            from: Action::Medium
        }));
    }

    #[test]
    fn same_seed_same_fight() {
        let c = content();
        let run = |seed| {
            let mut f = fight(&c, seed);
            let mut ev = Vec::new();
            while f.outcome.is_none() {
                f.play_round(&c, Action::Advance, Action::Medium, &mut ev);
                f.play_round(&c, Action::Medium, Action::Heavy, &mut ev);
            }
            (f, ev)
        };
        assert_eq!(run(77), run(77));
    }

    #[test]
    fn fight_always_ends_by_turn_limit() {
        let c = content();
        let mut f = fight(&c, 5);
        let mut ev = Vec::new();
        while f.outcome.is_none() {
            f.play_round(&c, Action::Defend, Action::Defend, &mut ev);
        }
        let o = f.outcome;
        assert_eq!(o.map(|o| o.reason), Some(EndReason::TurnLimit));
        assert_eq!(o.map(|o| o.turns), Some(c.rules.fight_end.max_turns));
        // Empate total: vence o jogador (desafiante).
        assert_eq!(o.map(|o| o.winner), Some(Side::Player));
    }

    #[test]
    fn defend_reduces_damage_of_next_hit() {
        let c = content();
        let mut f = fight(&c, 1);
        f.fighters[0].pos = 4;
        f.fighters[1].pos = 5;
        let normal = f.expected_hit_damage(&c, Side::Opponent, Action::Medium);
        f.fighters[0].stance = Stance::Defend;
        let defended = f.expected_hit_damage(&c, Side::Opponent, Action::Medium);
        let pm = i64::from(c.rules.actions.defend.damage_taken_pm);
        // Arredondamento inteiro: tolerância de 1 ponto.
        assert!(
            (defended - normal * pm / 1000).abs() <= 1,
            "{defended} vs {normal}"
        );
        assert!(defended <= normal);
    }
}
