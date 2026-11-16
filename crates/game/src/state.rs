//! Estado do jogo e a função de transição [`step`].
//!
//! `step` é a única forma de alterar o estado. Não lê relógio, não acessa disco e não conhece
//! animação: recebe uma entrada e devolve eventos para a interface.

use crate::ai;
use crate::combat::{Action, FightEvent, FightState, OpponentRef, Outcome, Side};
use crate::content::{Archetype, Content, ItemId, ItemKind};
use crate::fighter::{Attr, Attributes, Equipment, Fighter};
use crate::progression;
use crate::rng::Pcg32;
use serde::{Deserialize, Serialize};

/// Versão do esquema do estado salvo.
pub const SCHEMA_VERSION: u32 = 1;

/// Fluxo de sequência do PCG32 reservado ao estado global (sementes de luta).
const STATE_STREAM: u64 = 0x57a7_e001;

/// Modo de jogo.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind", content = "strategy")]
pub enum Mode {
    /// O jogador escolhe cada ação.
    Manual,
    /// A IA joga pelo jogador com a estratégia indicada.
    Auto(Archetype),
}

/// Resumo de uma luta encerrada.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FightSummary {
    /// Oponente.
    pub opponent: OpponentRef,
    /// Nome do oponente.
    pub opponent_name: String,
    /// Resultado.
    pub outcome: Outcome,
    /// XP ganho.
    pub xp: u32,
    /// Ouro ganho.
    pub gold: u32,
    /// Luta de treino.
    pub rematch: bool,
}

/// Tela atual.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind", content = "data")]
pub enum Screen {
    /// Fora de luta.
    Hub,
    /// Luta em andamento.
    Fight(Box<FightState>),
    /// Luta encerrada, aguardando o jogador continuar.
    PostFight(FightSummary),
}

/// Estatísticas da carreira.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct CareerStats {
    /// Lutas disputadas.
    pub fights: u32,
    /// Vitórias.
    pub wins: u32,
    /// Derrotas.
    pub losses: u32,
    /// Itens comprados.
    pub items_bought: u32,
    /// Derrotas seguidas atuais.
    pub loss_streak: u32,
    /// Maior sequência de derrotas.
    pub max_loss_streak: u32,
}

/// Aparência do lutador do jogador. É só visual: não entra em nenhuma regra. Cada campo é um
/// índice nas tabelas de arte (tons de pele, cortes de cabelo, cores, barbas); quem desenha trata
/// índices fora das tabelas dando a volta, então o jogo não precisa conhecer os tamanhos delas.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Appearance {
    /// Tom de pele.
    pub skin: u8,
    /// Corte de cabelo.
    pub hair: u8,
    /// Cor do cabelo e da barba.
    pub hair_color: u8,
    /// Cor dos olhos.
    pub eyes: u8,
    /// Barba.
    pub beard: u8,
}

impl Appearance {
    /// Verdadeiro se é a aparência padrão (não precisa ir para o save).
    pub fn is_default(&self) -> bool {
        *self == Self::default()
    }
}

impl Default for Appearance {
    fn default() -> Self {
        Self {
            skin: 2,
            hair: 1,
            hair_color: 1,
            eyes: 0,
            beard: 0,
        }
    }
}

/// Estado completo do jogo (é o que vai para o save).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameState {
    /// Versão do esquema.
    pub version: u32,
    /// Lutador do jogador.
    pub player: Fighter,
    /// Aparência do lutador do jogador. Só é gravada quando difere da padrão: o save de quem
    /// nunca personalizou continua idêntico ao formato anterior (e os hashes dos replays, que
    /// cobrem o estado serializado, não mudam). Saves antigos, sem o campo, usam a padrão.
    #[serde(default, skip_serializing_if = "Appearance::is_default")]
    pub appearance: Appearance,
    /// XP acumulado no nível atual.
    pub xp: u32,
    /// Pontos de atributo não distribuídos.
    pub unspent_points: u16,
    /// Ouro.
    pub gold: u32,
    /// Itens possuídos (inclui os equipados).
    pub inventory: Vec<ItemId>,
    /// Quantos oponentes de cada liga já foram vencidos (índice do próximo desafio).
    pub league_progress: Vec<u16>,
    /// Modo de jogo.
    pub mode: Mode,
    /// Tela atual.
    pub screen: Screen,
    /// Gerador das sementes de luta.
    pub rng: Pcg32,
    /// Estatísticas.
    pub stats: CareerStats,
}

/// Entrada do jogador (ou do temporizador, no modo automático).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Input {
    /// Inicia uma luta contra um oponente.
    StartFight(OpponentRef),
    /// Ação de combate no modo manual.
    FightAction(Action),
    /// Resolve uma rodada no modo automático.
    AutoRound,
    /// Sai da tela de resultado.
    Continue,
    /// Muda o modo de jogo.
    SetMode(Mode),
    /// Distribui um ponto de atributo.
    AllocatePoint(Attr),
    /// Compra um item.
    Buy(ItemId),
    /// Vende um item.
    Sell(ItemId),
    /// Equipa um item possuído.
    Equip(ItemId),
    /// Desequipa um slot.
    Unequip(ItemKind),
    /// Muda a aparência do lutador do jogador (permitido em qualquer tela: é só visual).
    SetAppearance(Appearance),
}

/// Motivo de uma entrada recusada.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Rejection {
    /// Entrada não permitida na tela atual.
    WrongScreen,
    /// Oponente ou liga ainda bloqueados.
    Locked,
    /// Oponente inexistente.
    UnknownOpponent,
    /// Item inexistente.
    UnknownItem,
    /// Item não possuído.
    NotOwned,
    /// Ouro insuficiente.
    NotEnoughGold,
    /// Inventário cheio.
    InventoryFull,
    /// Sem pontos para distribuir.
    NoPoints,
    /// Atributo no máximo.
    AttributeAtMax,
    /// Ação de combate inválida agora.
    InvalidAction,
    /// Entrada não se aplica ao modo atual.
    WrongMode,
}

/// Evento de jogo.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameEvent {
    /// Entrada recusada (estado inalterado).
    Rejected(Rejection),
    /// Luta iniciada.
    FightStarted(OpponentRef),
    /// Evento de combate.
    Fight(FightEvent),
    /// Recompensa recebida.
    Reward {
        /// XP.
        xp: u32,
        /// Ouro.
        gold: u32,
    },
    /// Subiu de nível.
    LevelUp(u8),
    /// Liga concluída (campeão vencido).
    LeagueCompleted(u16),
    /// Item comprado.
    Bought(ItemId),
    /// Item vendido.
    Sold(ItemId),
    /// Item equipado.
    Equipped(ItemId),
    /// Slot desequipado.
    Unequipped(ItemKind),
    /// Ponto distribuído.
    PointAllocated(Attr),
    /// Modo alterado.
    ModeChanged(Mode),
    /// Aparência alterada.
    AppearanceChanged(Appearance),
}

/// Quando o aplicativo deve salvar.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SaveHint {
    /// Nada mudou que precise ser salvo.
    None,
    /// Salvar em breve (pode agrupar).
    Soon,
    /// Salvar agora.
    Now,
}

/// Resultado de [`step`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StepResult {
    /// Eventos produzidos.
    pub events: Vec<GameEvent>,
    /// Indicação de gravação.
    pub save: SaveHint,
}

impl GameState {
    /// Cria um personagem novo.
    pub fn new(content: &Content, name: &str, seed: u64) -> Self {
        let r = &content.rules;
        let start_weapon = content.starting_weapon();
        Self {
            version: SCHEMA_VERSION,
            player: Fighter {
                name: name.chars().take(24).collect(),
                level: 1,
                attrs: Attributes([r.attributes.start_value; crate::content::ATTR_COUNT]),
                equipment: Equipment {
                    weapon: start_weapon,
                    armor: None,
                    shield: None,
                },
            },
            appearance: Appearance::default(),
            xp: 0,
            unspent_points: r.attributes.start_free_points,
            gold: r.progression.start_gold,
            inventory: start_weapon.into_iter().collect(),
            league_progress: vec![0; content.leagues.len()],
            mode: Mode::Manual,
            screen: Screen::Hub,
            rng: Pcg32::new(seed, STATE_STREAM),
            stats: CareerStats::default(),
        }
    }

    /// Verdadeiro se a liga está liberada (a primeira sempre; as demais após concluir a anterior).
    pub fn league_unlocked(&self, content: &Content, league: usize) -> bool {
        if league == 0 {
            return true;
        }
        let prev = league - 1;
        match (content.leagues.get(prev), self.league_progress.get(prev)) {
            (Some(def), Some(&p)) => usize::from(p) >= def.opponents.len(),
            _ => false,
        }
    }

    /// Verdadeiro se o oponente pode ser desafiado.
    pub fn can_challenge(&self, content: &Content, r: OpponentRef) -> bool {
        let (l, i) = (usize::from(r.league), usize::from(r.index));
        content.opponent(l, i).is_some()
            && self.league_unlocked(content, l)
            && self
                .league_progress
                .get(l)
                .is_some_and(|&p| i <= usize::from(p))
    }

    /// Arquétipo que controla o jogador quando a IA joga por ele.
    pub fn player_archetype(&self) -> Archetype {
        match self.mode {
            Mode::Manual => Archetype::Balanced,
            Mode::Auto(a) => a,
        }
    }
}

/// Aplica uma entrada ao estado.
pub fn step(content: &Content, state: &mut GameState, input: Input) -> StepResult {
    let mut events = Vec::new();
    let save = match input {
        Input::StartFight(r) => start_fight(content, state, r, &mut events),
        Input::FightAction(a) => fight_round(content, state, Some(a), &mut events),
        Input::AutoRound => fight_round(content, state, None, &mut events),
        Input::Continue => {
            if matches!(state.screen, Screen::PostFight(_)) {
                state.screen = Screen::Hub;
                SaveHint::Soon
            } else {
                reject(&mut events, Rejection::WrongScreen)
            }
        }
        Input::SetMode(m) => {
            state.mode = m;
            let archetype = state.player_archetype();
            if let Screen::Fight(f) = &mut state.screen {
                f.fighters[Side::Player.index()].archetype = archetype;
            }
            events.push(GameEvent::ModeChanged(m));
            SaveHint::Soon
        }
        Input::AllocatePoint(attr) => allocate(content, state, attr, &mut events),
        Input::Buy(id) => buy(content, state, id, &mut events),
        Input::Sell(id) => sell(content, state, id, &mut events),
        Input::Equip(id) => equip(content, state, id, &mut events),
        Input::Unequip(kind) => {
            if !matches!(state.screen, Screen::Hub) {
                reject(&mut events, Rejection::WrongScreen)
            } else {
                *state.player.equipment.slot_mut(kind) = None;
                events.push(GameEvent::Unequipped(kind));
                SaveHint::Soon
            }
        }
        Input::SetAppearance(a) => {
            state.appearance = a;
            events.push(GameEvent::AppearanceChanged(a));
            SaveHint::Soon
        }
    };
    StepResult { events, save }
}

fn reject(events: &mut Vec<GameEvent>, r: Rejection) -> SaveHint {
    events.push(GameEvent::Rejected(r));
    SaveHint::None
}

fn start_fight(
    content: &Content,
    state: &mut GameState,
    r: OpponentRef,
    events: &mut Vec<GameEvent>,
) -> SaveHint {
    if !matches!(state.screen, Screen::Hub) {
        return reject(events, Rejection::WrongScreen);
    }
    let Some(def) = content.opponent(usize::from(r.league), usize::from(r.index)) else {
        return reject(events, Rejection::UnknownOpponent);
    };
    if !state.can_challenge(content, r) {
        return reject(events, Rejection::Locked);
    }
    let rematch = state
        .league_progress
        .get(usize::from(r.league))
        .is_some_and(|&p| r.index < p);
    let opponent = Fighter::from_opponent(content, def);
    // A semente é sorteada e gravada no início da luta: reabrir o jogo reproduz o mesmo resultado.
    let seed = state.rng.next_u64();
    let fight = FightState::new(
        content,
        &state.player,
        state.player_archetype(),
        &opponent,
        def.archetype,
        r,
        rematch,
        seed,
    );
    state.screen = Screen::Fight(Box::new(fight));
    events.push(GameEvent::FightStarted(r));
    SaveHint::Now
}

fn fight_round(
    content: &Content,
    state: &mut GameState,
    manual: Option<Action>,
    events: &mut Vec<GameEvent>,
) -> SaveHint {
    let mode = state.mode;
    let Screen::Fight(fight) = &mut state.screen else {
        return reject(events, Rejection::WrongScreen);
    };
    let player_action = match (manual, mode) {
        (Some(a), Mode::Manual) => {
            if !fight.is_valid(content, Side::Player, a) {
                return reject(events, Rejection::InvalidAction);
            }
            a
        }
        (None, Mode::Auto(strategy)) => ai::choose_with(content, fight, Side::Player, strategy),
        _ => return reject(events, Rejection::WrongMode),
    };
    let opponent_action = ai::choose_action(content, fight, Side::Opponent);
    let mut fight_events = Vec::new();
    fight.play_round(content, player_action, opponent_action, &mut fight_events);
    events.extend(fight_events.into_iter().map(GameEvent::Fight));
    let Some(outcome) = fight.outcome else {
        return SaveHint::None;
    };
    let fight = fight.clone();
    finish_fight(content, state, &fight, outcome, events);
    SaveHint::Now
}

fn finish_fight(
    content: &Content,
    state: &mut GameState,
    fight: &FightState,
    outcome: Outcome,
    events: &mut Vec<GameEvent>,
) {
    let won = outcome.winner == Side::Player;
    let opp = fight.fighter(Side::Opponent);
    let reward = progression::reward(content, state.player.level, opp.level, won, fight.rematch);
    state.gold = state.gold.saturating_add(reward.gold);
    events.push(GameEvent::Reward {
        xp: reward.xp,
        gold: reward.gold,
    });
    gain_xp(content, state, reward.xp, events);

    let s = &mut state.stats;
    s.fights = s.fights.saturating_add(1);
    if won {
        s.wins = s.wins.saturating_add(1);
        s.loss_streak = 0;
    } else {
        s.losses = s.losses.saturating_add(1);
        s.loss_streak = s.loss_streak.saturating_add(1);
        s.max_loss_streak = s.max_loss_streak.max(s.loss_streak);
    }

    let r = fight.opponent;
    if won && !fight.rematch {
        let l = usize::from(r.league);
        if let Some(p) = state.league_progress.get_mut(l)
            && *p == r.index
        {
            *p = p.saturating_add(1);
            let len = content.leagues.get(l).map_or(0, |d| d.opponents.len());
            if usize::from(*p) >= len {
                events.push(GameEvent::LeagueCompleted(r.league));
            }
        }
    }
    state.screen = Screen::PostFight(FightSummary {
        opponent: r,
        opponent_name: opp.name.clone(),
        outcome,
        xp: reward.xp,
        gold: reward.gold,
        rematch: fight.rematch,
    });
}

fn gain_xp(content: &Content, state: &mut GameState, xp: u32, events: &mut Vec<GameEvent>) {
    let p = &content.rules.progression;
    state.xp = state.xp.saturating_add(xp);
    loop {
        if state.player.level >= p.max_level {
            state.xp = 0;
            break;
        }
        let need = progression::xp_to_next(content, state.player.level);
        if state.xp < need {
            break;
        }
        state.xp -= need;
        state.player.level += 1;
        state.unspent_points = state
            .unspent_points
            .saturating_add(content.rules.attributes.points_per_level);
        events.push(GameEvent::LevelUp(state.player.level));
    }
}

fn allocate(
    content: &Content,
    state: &mut GameState,
    attr: Attr,
    events: &mut Vec<GameEvent>,
) -> SaveHint {
    if matches!(state.screen, Screen::Fight(_)) {
        return reject(events, Rejection::WrongScreen);
    }
    if state.unspent_points == 0 {
        return reject(events, Rejection::NoPoints);
    }
    let v = state.player.attrs.get(attr);
    if v >= content.rules.attributes.max {
        return reject(events, Rejection::AttributeAtMax);
    }
    state.player.attrs.set(attr, v + 1);
    state.unspent_points -= 1;
    events.push(GameEvent::PointAllocated(attr));
    SaveHint::Soon
}

fn buy(
    content: &Content,
    state: &mut GameState,
    id: ItemId,
    events: &mut Vec<GameEvent>,
) -> SaveHint {
    if !matches!(state.screen, Screen::Hub) {
        return reject(events, Rejection::WrongScreen);
    }
    let Some(item) = content.item(id) else {
        return reject(events, Rejection::UnknownItem);
    };
    if state.inventory.len() >= usize::from(content.rules.progression.inventory_max) {
        return reject(events, Rejection::InventoryFull);
    }
    if state.gold < item.price {
        return reject(events, Rejection::NotEnoughGold);
    }
    state.gold -= item.price;
    state.inventory.push(id);
    state.stats.items_bought = state.stats.items_bought.saturating_add(1);
    events.push(GameEvent::Bought(id));
    SaveHint::Soon
}

fn sell(
    content: &Content,
    state: &mut GameState,
    id: ItemId,
    events: &mut Vec<GameEvent>,
) -> SaveHint {
    if !matches!(state.screen, Screen::Hub) {
        return reject(events, Rejection::WrongScreen);
    }
    let Some(item) = content.item(id) else {
        return reject(events, Rejection::UnknownItem);
    };
    let Some(pos) = state.inventory.iter().position(|&i| i == id) else {
        return reject(events, Rejection::NotOwned);
    };
    state.inventory.remove(pos);
    // Se não restar outra cópia, o slot que usava o item fica vazio.
    if !state.inventory.contains(&id) {
        let slot = state.player.equipment.slot_mut(item.kind);
        if *slot == Some(id) {
            *slot = None;
        }
    }
    let value = item.price.saturating_mul(content.rules.progression.sell_pm) / 1000;
    state.gold = state.gold.saturating_add(value);
    events.push(GameEvent::Sold(id));
    SaveHint::Soon
}

fn equip(
    content: &Content,
    state: &mut GameState,
    id: ItemId,
    events: &mut Vec<GameEvent>,
) -> SaveHint {
    if !matches!(state.screen, Screen::Hub) {
        return reject(events, Rejection::WrongScreen);
    }
    let Some(item) = content.item(id) else {
        return reject(events, Rejection::UnknownItem);
    };
    if !state.inventory.contains(&id) {
        return reject(events, Rejection::NotOwned);
    }
    *state.player.equipment.slot_mut(item.kind) = Some(id);
    events.push(GameEvent::Equipped(id));
    SaveHint::Soon
}

#[cfg(test)]
mod tests {
    use super::*;

    fn content() -> Content {
        match Content::embedded() {
            Ok(c) => c,
            Err(e) => panic!("{e}"),
        }
    }

    fn first_opponent() -> OpponentRef {
        OpponentRef {
            league: 0,
            index: 0,
        }
    }

    #[test]
    fn new_character_has_start_weapon_and_points() {
        let c = content();
        let s = GameState::new(&c, "Teste", 1);
        assert_eq!(s.player.equipment.weapon, c.starting_weapon());
        assert_eq!(s.unspent_points, c.rules.attributes.start_free_points);
    }

    #[test]
    fn locked_opponent_is_rejected() {
        let c = content();
        let mut s = GameState::new(&c, "T", 1);
        let r = step(
            &c,
            &mut s,
            Input::StartFight(OpponentRef {
                league: 0,
                index: 3,
            }),
        );
        assert_eq!(r.events, vec![GameEvent::Rejected(Rejection::Locked)]);
        assert_eq!(s.screen, Screen::Hub);
    }

    #[test]
    fn full_auto_fight_reaches_post_fight_and_rewards() {
        let c = content();
        let mut s = GameState::new(&c, "T", 1);
        step(&c, &mut s, Input::SetMode(Mode::Auto(Archetype::Balanced)));
        step(&c, &mut s, Input::StartFight(first_opponent()));
        let mut rounds = 0;
        while matches!(s.screen, Screen::Fight(_)) {
            step(&c, &mut s, Input::AutoRound);
            rounds += 1;
            assert!(rounds <= u32::from(c.rules.fight_end.max_turns));
        }
        assert!(matches!(s.screen, Screen::PostFight(_)));
        assert_eq!(s.stats.fights, 1);
        step(&c, &mut s, Input::Continue);
        assert_eq!(s.screen, Screen::Hub);
    }

    #[test]
    fn manual_input_rejected_in_auto_mode_and_vice_versa() {
        let c = content();
        let mut s = GameState::new(&c, "T", 1);
        step(&c, &mut s, Input::StartFight(first_opponent()));
        let r = step(&c, &mut s, Input::AutoRound);
        assert_eq!(r.events, vec![GameEvent::Rejected(Rejection::WrongMode)]);
        step(
            &c,
            &mut s,
            Input::SetMode(Mode::Auto(Archetype::Aggressive)),
        );
        let r = step(&c, &mut s, Input::FightAction(Action::Defend));
        assert_eq!(r.events, vec![GameEvent::Rejected(Rejection::WrongMode)]);
    }

    #[test]
    fn shop_buy_sell_equip() {
        let c = content();
        let mut s = GameState::new(&c, "T", 1);
        let r = step(&c, &mut s, Input::Buy(2));
        assert_eq!(
            r.events,
            vec![GameEvent::Rejected(Rejection::NotEnoughGold)]
        );
        s.gold = 100;
        step(&c, &mut s, Input::Buy(2));
        assert_eq!(s.gold, 40);
        step(&c, &mut s, Input::Equip(2));
        assert_eq!(s.player.equipment.weapon, Some(2));
        step(&c, &mut s, Input::Sell(2));
        assert_eq!(s.player.equipment.weapon, None);
        assert_eq!(s.gold, 70);
    }

    #[test]
    fn level_up_grants_points() {
        let c = content();
        let mut s = GameState::new(&c, "T", 1);
        let mut ev = Vec::new();
        gain_xp(&c, &mut s, 10_000, &mut ev);
        assert!(s.player.level > 1);
        let expected = c.rules.attributes.start_free_points
            + c.rules.attributes.points_per_level * u16::from(s.player.level - 1);
        assert_eq!(s.unspent_points, expected);
    }

    #[test]
    fn same_seed_same_career_step() {
        let c = content();
        let run = || {
            let mut s = GameState::new(&c, "T", 99);
            step(
                &c,
                &mut s,
                Input::SetMode(Mode::Auto(Archetype::Opportunist)),
            );
            step(&c, &mut s, Input::StartFight(first_opponent()));
            while matches!(s.screen, Screen::Fight(_)) {
                step(&c, &mut s, Input::AutoRound);
            }
            s
        };
        assert_eq!(run(), run());
    }
}
