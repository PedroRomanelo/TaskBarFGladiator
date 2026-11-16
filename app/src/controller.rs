//! Controlador do aplicativo: liga o estado do jogo, a animação e as visões de renderização
//! (portável, testado sem Windows).
//!
//! A camada Windows chama os métodos `on_*` e aplica os [`Effects`] devolvidos (redesenhar, abrir
//! painel, salvar, sair). Os temporizadores são definidos a partir de [`App::anim_busy`] e
//! [`App::auto_delay_ms`].
//!
//! O palco sempre diz ao jogador em que etapa ele está e o que fazer:
//!
//! | Etapa | Legenda | Botão principal |
//! |---|---|---|
//! | Preparar (fora de luta) | próximo oponente e nível | LUTAR |
//! | Luta, sua vez (manual) | "RODADA n: ESCOLHA SUA AÇÃO", ou a explicação do botão sob o mouse | botões de ação |
//! | Luta, rodada em andamento | quem agiu, o golpe, o dano ou o erro | — |
//! | Resultado | vitória ou derrota, com a recompensa | CONTINUAR |

use crate::anim::{Animator, Caption};
use crate::config::Config;
use crate::i18n::{Key, Lang, t};
use crate::looks;
use game::{
    Action, Archetype, Attr, Content, FightState, GameEvent, GameState, Input, ItemKind, Mode,
    OpponentRef, Rejection, SaveHint, Screen, Side, step,
};
use render::{ButtonState, FighterView, HitId, Look, PanelRow, PanelView, Phase, StageView, Tone};

/// Efeitos pedidos à camada de plataforma.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Effects {
    /// Redesenhar o palco.
    pub redraw_strip: bool,
    /// Redesenhar o painel.
    pub redraw_panel: bool,
    /// Abrir o painel.
    pub open_panel: bool,
    /// Fechar o painel.
    pub close_panel: bool,
    /// Salvar o jogo agora.
    pub save_now: bool,
    /// Salvar o jogo em breve (agrupável).
    pub save_soon: bool,
    /// Salvar a configuração.
    pub save_config: bool,
    /// Reposicionar o palco no padrão.
    pub reset_position: bool,
    /// Recalcular posição e tamanho do palco.
    pub relayout: bool,
    /// Alterar "iniciar com o Windows".
    pub start_with_windows: Option<bool>,
    /// Encerrar o jogo.
    pub quit: bool,
}

impl Effects {
    fn merge(&mut self, o: Effects) {
        self.redraw_strip |= o.redraw_strip;
        self.redraw_panel |= o.redraw_panel;
        self.open_panel |= o.open_panel;
        self.close_panel |= o.close_panel;
        self.save_now |= o.save_now;
        self.save_soon |= o.save_soon;
        self.save_config |= o.save_config;
        self.reset_position |= o.reset_position;
        self.relayout |= o.relayout;
        self.quit |= o.quit;
        if o.start_with_windows.is_some() {
            self.start_with_windows = o.start_with_windows;
        }
    }

    fn from_hint(h: SaveHint) -> Effects {
        Effects {
            save_now: h == SaveHint::Now,
            save_soon: h == SaveHint::Soon,
            ..Default::default()
        }
    }

    fn redraw() -> Effects {
        Effects {
            redraw_strip: true,
            ..Default::default()
        }
    }
}

/// Parte da aparência alterada por uma linha da aba de aparência.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LookPart {
    Skin,
    Hair,
    HairColor,
    Eyes,
    Beard,
}

/// Ação associada a um botão de linha do painel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RowAction {
    Allocate(Attr),
    CycleMode,
    ToggleAutoAllocate,
    Buy(game::ItemId),
    Equip(game::ItemId),
    Challenge(OpponentRef),
    CycleLook(LookPart),
    ToggleLang,
    ToggleReduceMotion,
    ToggleStartWithWindows,
    CycleSize,
    Reposition,
    CyclePace,
    Quit,
}

/// O que a legenda da luta está mostrando. Serve para só refazer o texto quando ele muda (o
/// desenho de cada quadro não aloca).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Cap {
    /// Legenda definida ao montar a visão (preparação e resultado), ou ainda não calculada.
    Other,
    /// Passo da rodada em andamento.
    Anim(Caption),
    /// Vez do jogador.
    Choose { turn: u16 },
    /// Vez do jogador, com o mouse sobre um botão de ação.
    Info { action: Action, turn: u16 },
    /// Modo automático.
    Auto { turn: u16, paused: bool },
}

/// Tempo de exibição do resultado no modo automático (ms).
const RESULT_MS: u32 = 3_000;
/// Índice da aba de configurações (a última).
const TAB_CONFIG: usize = 4;

/// Estado do aplicativo.
#[derive(Debug)]
pub struct App {
    content: Content,
    state: GameState,
    config: Config,
    lang: Lang,
    anim: Option<Animator>,
    paused: bool,
    panel_tab: usize,
    row_actions: Vec<Option<RowAction>>,
    message: Option<Key>,
    stage: StageView,
    cap: Cap,
    hover: Option<HitId>,
    league_just_completed: bool,
}

const ATTRS: [(Attr, Key); 6] = [
    (Attr::Strength, Key::Strength),
    (Attr::Technique, Key::Technique),
    (Attr::Agility, Key::Agility),
    (Attr::Defense, Key::Defense),
    (Attr::Vitality, Key::Vitality),
    (Attr::Stamina, Key::Stamina),
];

const HAIR_NAMES: [Key; 6] = [
    Key::HairBald,
    Key::HairShort,
    Key::HairLong,
    Key::HairMohawk,
    Key::HairTopknot,
    Key::HairCurly,
];

const BEARD_NAMES: [Key; 5] = [
    Key::BeardNone,
    Key::BeardStubble,
    Key::BeardMustache,
    Key::BeardFull,
    Key::BeardLong,
];

fn archetype_key(a: Archetype) -> Key {
    match a {
        Archetype::Aggressive => Key::Aggressive,
        Archetype::Defensive => Key::Defensive,
        Archetype::Opportunist => Key::Opportunist,
        Archetype::Balanced => Key::Balanced,
    }
}

fn rejection_key(r: Rejection) -> Key {
    match r {
        Rejection::NotEnoughGold => Key::NotEnoughGold,
        Rejection::InventoryFull => Key::InventoryFull,
        Rejection::Locked | Rejection::UnknownOpponent => Key::Locked,
        _ => Key::NotInFight,
    }
}

/// Nome de uma ação no botão (infinitivo).
fn action_key(a: Action) -> Key {
    match a {
        Action::Advance => Key::ActAdvance,
        Action::Retreat => Key::ActRetreat,
        Action::Light => Key::ActLight,
        Action::Medium => Key::ActMedium,
        Action::Heavy => Key::ActHeavy,
        Action::Defend => Key::ActDefend,
        Action::Recover => Key::ActRecover,
        Action::Taunt => Key::ActTaunt,
    }
}

/// Ação narrada na legenda ("fulano: avança").
fn narration_key(a: Action) -> Key {
    match a {
        Action::Advance => Key::NarAdvance,
        Action::Retreat => Key::NarRetreat,
        Action::Light => Key::ActLight,
        Action::Medium => Key::ActMedium,
        Action::Heavy => Key::ActHeavy,
        Action::Defend => Key::NarDefend,
        Action::Recover => Key::NarRecover,
        Action::Taunt => Key::NarTaunt,
    }
}

impl App {
    /// Cria o controlador.
    pub fn new(content: Content, state: GameState, config: Config, system_lang: Lang) -> Self {
        let lang = config.language.unwrap_or(system_lang);
        let mut app = Self {
            content,
            state,
            config,
            lang,
            anim: None,
            paused: false,
            panel_tab: 0,
            row_actions: Vec::new(),
            message: None,
            stage: StageView {
                player: blank_fighter(),
                opponent: blank_fighter(),
                cells: 1,
                caption: String::new(),
                tone: Tone::Normal,
                hint: String::new(),
                phase: Phase::Fight {
                    buttons: None,
                    paused: false,
                },
            },
            cap: Cap::Other,
            hover: None,
            league_just_completed: false,
        };
        if let Screen::Fight(f) = &app.state.screen {
            app.anim = Some(Animator::new(f, app.config.reduce_motion));
        }
        app.rebuild_stage_view();
        app
    }

    /// Conteúdo.
    pub fn content(&self) -> &Content {
        &self.content
    }

    /// Estado do jogo.
    pub fn state(&self) -> &GameState {
        &self.state
    }

    /// Configuração.
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Configuração (mutável, para deslocamentos salvos pela camada de plataforma).
    pub fn config_mut(&mut self) -> &mut Config {
        &mut self.config
    }

    /// Idioma.
    pub fn lang(&self) -> Lang {
        self.lang
    }

    /// Pausado (modo automático).
    pub fn paused(&self) -> bool {
        self.paused
    }

    /// Pausa/continua o modo automático.
    pub fn set_paused(&mut self, p: bool) -> Effects {
        self.paused = p;
        self.rebuild_stage_view();
        Effects::redraw()
    }

    /// Verdadeiro enquanto há algo animando (temporizador de quadros ligado).
    pub fn anim_busy(&self) -> bool {
        self.anim.as_ref().is_some_and(Animator::busy)
    }

    /// Verdadeiro enquanto a rodada ainda está sendo mostrada (o jogo espera para seguir).
    fn resolving(&self) -> bool {
        self.anim.as_ref().is_some_and(Animator::resolving)
    }

    /// Atraso até a próxima ação automática, ou `None` se não há ação automática pendente.
    pub fn auto_delay_ms(&self) -> Option<u32> {
        let Mode::Auto(_) = self.state.mode else {
            return None;
        };
        if self.paused {
            return None;
        }
        match &self.state.screen {
            Screen::Fight(_) => Some(self.config.pace.round_ms()),
            Screen::PostFight(_) => Some(RESULT_MS),
            Screen::Hub => Some(self.config.pace.rest_ms()),
        }
    }

    /// Visão atual do palco.
    pub fn stage(&self) -> &StageView {
        &self.stage
    }

    /// Região do palco sob o mouse.
    pub fn hover(&self) -> Option<HitId> {
        self.hover
    }

    /// Informa a região do palco sob o mouse (`None` quando o mouse sai). Realça o botão e, na
    /// vez do jogador, troca a legenda pela explicação da ação.
    pub fn set_hover(&mut self, hover: Option<HitId>) -> Effects {
        if hover == self.hover {
            return Effects::default();
        }
        self.hover = hover;
        self.refresh_caption();
        Effects::redraw()
    }

    /// Avança a animação. Retorna os efeitos (redesenho).
    pub fn on_anim_tick(&mut self, dt_ms: u32) -> Effects {
        let changed = self.anim.as_mut().is_some_and(|a| a.tick(dt_ms));
        // A última rodada terminou de ser mostrada: passa para o resultado.
        if !self.resolving()
            && matches!(self.state.screen, Screen::PostFight(_))
            && matches!(self.stage.phase, Phase::Fight { .. })
        {
            self.rebuild_stage_view();
            return Effects::redraw();
        }
        if changed {
            self.refresh_fight_view();
        }
        Effects {
            redraw_strip: changed,
            ..Default::default()
        }
    }

    /// Ação automática (temporizador do modo automático).
    pub fn on_auto_tick(&mut self) -> Effects {
        if self.resolving() || self.paused {
            return Effects::default();
        }
        let Mode::Auto(_) = self.state.mode else {
            return Effects::default();
        };
        match &self.state.screen {
            Screen::Fight(_) => self.apply(Input::AutoRound),
            Screen::PostFight(_) => self.apply(Input::Continue),
            Screen::Hub => {
                let mut fx = Effects::default();
                if self.config.auto_allocate {
                    fx.merge(self.auto_allocate());
                }
                let target = self.next_challenge();
                fx.merge(self.apply(Input::StartFight(target)));
                fx
            }
        }
    }

    /// Clique no palco.
    pub fn on_strip_hit(&mut self, id: HitId) -> Effects {
        match id {
            HitId::Action(i) => {
                let Some(&action) = Action::ALL.get(usize::from(i)) else {
                    return Effects::default();
                };
                if self.resolving() {
                    // Clicar durante a rodada só adianta a animação; a ação é escolhida depois,
                    // já vendo o resultado.
                    if let Some(a) = &mut self.anim {
                        a.finish();
                    }
                    return self.on_anim_tick(0).merged(Effects::redraw());
                }
                let valid = match &self.state.screen {
                    Screen::Fight(f) => f.is_valid(&self.content, Side::Player, action),
                    _ => false,
                };
                if !valid || self.state.mode != Mode::Manual {
                    // A legenda já explica o motivo enquanto o mouse está sobre o botão.
                    return Effects::default();
                }
                self.apply(Input::FightAction(action))
            }
            HitId::OpenPanel => Effects {
                open_panel: true,
                redraw_panel: true,
                ..Default::default()
            },
            HitId::TogglePause => self.set_paused(!self.paused),
            HitId::Fight => {
                let target = self.next_challenge();
                self.apply(Input::StartFight(target))
            }
            HitId::Continue => self.apply(Input::Continue),
            HitId::Tab(_) | HitId::Row(_) | HitId::ClosePanel => Effects::default(),
        }
    }

    /// Clique no painel.
    pub fn on_panel_hit(&mut self, id: HitId) -> Effects {
        let mut fx = Effects {
            redraw_panel: true,
            ..Default::default()
        };
        match id {
            HitId::Tab(i) => {
                self.panel_tab = usize::from(i).min(TAB_CONFIG);
                self.message = None;
            }
            HitId::ClosePanel => fx.close_panel = true,
            HitId::Row(r) => {
                if let Some(Some(action)) = self.row_actions.get(usize::from(r)).copied() {
                    fx.merge(self.row_action(action));
                }
            }
            _ => {}
        }
        fx
    }

    fn row_action(&mut self, a: RowAction) -> Effects {
        match a {
            RowAction::Allocate(attr) => self.apply(Input::AllocatePoint(attr)),
            RowAction::CycleMode => {
                let next = match self.state.mode {
                    Mode::Manual => Mode::Auto(Archetype::Balanced),
                    Mode::Auto(Archetype::Balanced) => Mode::Auto(Archetype::Aggressive),
                    Mode::Auto(Archetype::Aggressive) => Mode::Auto(Archetype::Defensive),
                    Mode::Auto(Archetype::Defensive) => Mode::Auto(Archetype::Opportunist),
                    Mode::Auto(Archetype::Opportunist) => Mode::Manual,
                };
                self.apply(Input::SetMode(next))
            }
            RowAction::ToggleAutoAllocate => {
                self.config.auto_allocate = !self.config.auto_allocate;
                Effects {
                    save_config: true,
                    ..Default::default()
                }
            }
            RowAction::Buy(id) => {
                let mut fx = self.apply(Input::Buy(id));
                if self.state.inventory.contains(&id) {
                    fx.merge(self.apply(Input::Equip(id)));
                }
                fx
            }
            RowAction::Equip(id) => self.apply(Input::Equip(id)),
            RowAction::Challenge(r) => {
                let mut fx = self.apply(Input::StartFight(r));
                if matches!(self.state.screen, Screen::Fight(_)) {
                    fx.close_panel = true;
                }
                fx
            }
            RowAction::CycleLook(part) => {
                let mut a = self.state.appearance;
                let (field, count) = match part {
                    LookPart::Skin => (&mut a.skin, looks::SKINS),
                    LookPart::Hair => (&mut a.hair, looks::HAIRS),
                    LookPart::HairColor => (&mut a.hair_color, looks::HAIR_COLORS),
                    LookPart::Eyes => (&mut a.eyes, looks::EYES),
                    LookPart::Beard => (&mut a.beard, looks::BEARDS),
                };
                *field = (*field % count + 1) % count;
                self.apply(Input::SetAppearance(a))
            }
            RowAction::ToggleLang => {
                self.lang = self.lang.toggle();
                self.config.language = Some(self.lang);
                self.rebuild_stage_view();
                Effects {
                    save_config: true,
                    redraw_strip: true,
                    ..Default::default()
                }
            }
            RowAction::ToggleReduceMotion => {
                self.config.reduce_motion = !self.config.reduce_motion;
                if let Some(a) = &mut self.anim {
                    a.set_reduce_motion(self.config.reduce_motion);
                }
                Effects {
                    save_config: true,
                    ..Default::default()
                }
            }
            RowAction::ToggleStartWithWindows => {
                self.config.start_with_windows = !self.config.start_with_windows;
                Effects {
                    save_config: true,
                    start_with_windows: Some(self.config.start_with_windows),
                    ..Default::default()
                }
            }
            RowAction::CycleSize => {
                self.config.size = match self.config.size {
                    0 => 1,
                    1 => -1,
                    _ => 0,
                };
                Effects {
                    save_config: true,
                    relayout: true,
                    ..Default::default()
                }
            }
            RowAction::Reposition => Effects {
                reset_position: true,
                save_config: true,
                ..Default::default()
            },
            RowAction::CyclePace => {
                self.config.pace = self.config.pace.next();
                Effects {
                    save_config: true,
                    ..Default::default()
                }
            }
            RowAction::Quit => Effects {
                quit: true,
                save_now: true,
                ..Default::default()
            },
        }
    }

    /// Distribui os pontos livres de forma equilibrada (no atributo de menor valor).
    fn auto_allocate(&mut self) -> Effects {
        let mut fx = Effects::default();
        let mut guard = 0;
        while self.state.unspent_points > 0 && guard < 1000 {
            let attr = Attr::ALL
                .iter()
                .copied()
                .min_by_key(|&a| (self.state.player.attrs.get(a), a.index()))
                .unwrap_or(Attr::Strength);
            let r = step(&self.content, &mut self.state, Input::AllocatePoint(attr));
            if r.events.iter().any(|e| matches!(e, GameEvent::Rejected(_))) {
                break;
            }
            fx.merge(Effects::from_hint(r.save));
            guard += 1;
        }
        fx
    }

    /// Próximo desafio: o próximo oponente da liga liberada mais avançada; treina contra o anterior
    /// quando estiver mais de um nível abaixo dele. Com todas as ligas concluídas, revanche contra o
    /// último campeão.
    pub fn next_challenge(&self) -> OpponentRef {
        let mut league = 0usize;
        for (i, def) in self.content.leagues.iter().enumerate() {
            if !self.state.league_unlocked(&self.content, i) {
                break;
            }
            league = i;
            let p = usize::from(self.state.league_progress.get(i).copied().unwrap_or(0));
            if p < def.opponents.len() {
                break;
            }
        }
        let def = &self.content.leagues[league];
        let progress = usize::from(self.state.league_progress.get(league).copied().unwrap_or(0));
        let last = def.opponents.len().saturating_sub(1);
        let next = progress.min(last);
        let index = if progress > last {
            last
        } else {
            let lvl = def.opponents.get(next).map_or(1, |o| o.level);
            let mode_auto = matches!(self.state.mode, Mode::Auto(_));
            if mode_auto && next > 0 && self.state.player.level.saturating_add(1) < lvl {
                next - 1
            } else {
                next
            }
        };
        OpponentRef {
            league: league as u16,
            index: index as u16,
        }
    }

    /// Aplica uma entrada ao jogo e atualiza animação, visões e mensagens.
    fn apply(&mut self, input: Input) -> Effects {
        let before_fight = matches!(self.state.screen, Screen::Fight(_));
        let r = step(&self.content, &mut self.state, input);
        let mut fx = Effects::from_hint(r.save);
        fx.redraw_strip = true;
        fx.redraw_panel = true;
        self.message = None;
        let mut fight_events = Vec::new();
        for e in &r.events {
            match e {
                GameEvent::Rejected(rej) => self.message = Some(rejection_key(*rej)),
                GameEvent::Fight(fe) => fight_events.push(*fe),
                GameEvent::FightStarted(_) => {
                    if let Screen::Fight(f) = &self.state.screen {
                        self.anim = Some(Animator::new(f, self.config.reduce_motion));
                    }
                }
                GameEvent::LeagueCompleted(_) => self.league_just_completed = true,
                GameEvent::Bought(_) => self.message = None,
                _ => {}
            }
        }
        if !fight_events.is_empty()
            && let Some(a) = &mut self.anim
        {
            // O estado depois da rodada: se a luta acabou, a tela já é PostFight; a animação
            // sincroniza com o último estado de luta conhecido pelo animador.
            if let Screen::Fight(f) = &self.state.screen {
                a.push_round(&fight_events, f);
            } else {
                a.push_round_final(&fight_events);
            }
        }
        if matches!(input, Input::Continue) {
            self.anim = None;
            self.league_just_completed = false;
        }
        if before_fight && matches!(self.state.screen, Screen::Hub) {
            self.anim = None;
        }
        self.rebuild_stage_view();
        fx
    }

    /// Desenho do lutador do jogador.
    fn player_look(&self) -> Look {
        looks::look(
            &self.content,
            self.state.appearance,
            &self.state.player.equipment,
        )
    }

    /// Nome curto e desenho de um oponente da liga.
    fn opponent_card(&self, r: OpponentRef) -> (String, u8, Look) {
        match self
            .content
            .opponent(usize::from(r.league), usize::from(r.index))
        {
            Some(def) => {
                let equipment = game::Equipment {
                    weapon: def.weapon,
                    armor: def.armor,
                    shield: def.shield,
                };
                (
                    looks::short_name(&def.name),
                    def.level,
                    looks::look(
                        &self.content,
                        looks::opponent_appearance(&def.name),
                        &equipment,
                    ),
                )
            }
            None => (String::new(), 1, Look::default()),
        }
    }

    /// Visões dos dois lutadores: da animação, quando há uma luta sendo mostrada; senão, parados
    /// nas casas iniciais.
    fn fighter_views(&self, r: OpponentRef) -> (FighterView, FighterView) {
        let player_name = looks::short_name(&self.state.player.name);
        let (opp_name, opp_level, opp_look) = self.opponent_card(r);
        match &self.anim {
            Some(a) => (
                a.view(Side::Player, player_name, self.player_look()),
                a.view(Side::Opponent, opp_name, opp_look),
            ),
            None => {
                let arena = &self.content.rules.arena;
                (
                    standing_fighter(
                        player_name,
                        self.state.player.level,
                        arena.player_start,
                        self.player_look(),
                    ),
                    standing_fighter(opp_name, opp_level, arena.opponent_start, opp_look),
                )
            }
        }
    }

    fn refresh_fight_view(&mut self) {
        if let (Some(a), Phase::Fight { .. }) = (&self.anim, &self.stage.phase) {
            let miss = t(self.lang, Key::Miss);
            a.fill_view(Side::Player, &mut self.stage.player, miss);
            a.fill_view(Side::Opponent, &mut self.stage.opponent, miss);
        }
        self.refresh_buttons();
        self.refresh_caption();
    }

    fn refresh_buttons(&mut self) {
        let waiting = self.resolving();
        let manual = self.state.mode == Mode::Manual;
        let fight = match &self.state.screen {
            Screen::Fight(f) => Some(f),
            _ => None,
        };
        if let Phase::Fight { buttons, paused } = &mut self.stage.phase {
            *paused = self.paused;
            *buttons = match (manual, fight) {
                (true, Some(f)) => {
                    let mut states = [ButtonState::Disabled; 8];
                    for (i, a) in Action::ALL.iter().enumerate() {
                        if !waiting && f.is_valid(&self.content, Side::Player, *a) {
                            states[i] = ButtonState::Enabled;
                        }
                    }
                    Some(states)
                }
                (true, None) => Some([ButtonState::Disabled; 8]),
                (false, _) => None,
            };
        }
    }

    /// O que a legenda da luta deve mostrar agora.
    fn fight_cap(&self) -> Cap {
        if !matches!(self.stage.phase, Phase::Fight { .. }) {
            return Cap::Other;
        }
        if let Some(c) = self.anim.as_ref().and_then(Animator::caption) {
            return Cap::Anim(c);
        }
        let Screen::Fight(f) = &self.state.screen else {
            return Cap::Other;
        };
        let turn = f.turn.saturating_add(1);
        match (self.state.mode, self.hover) {
            (Mode::Auto(_), _) => Cap::Auto {
                turn,
                paused: self.paused,
            },
            (Mode::Manual, Some(HitId::Action(i))) => {
                match Action::ALL.get(usize::from(i)).copied() {
                    Some(action) => Cap::Info { action, turn },
                    None => Cap::Choose { turn },
                }
            }
            (Mode::Manual, _) => Cap::Choose { turn },
        }
    }

    /// Refaz o texto da legenda da luta, só quando ele muda.
    fn refresh_caption(&mut self) {
        let cap = self.fight_cap();
        if cap == self.cap || cap == Cap::Other {
            return;
        }
        self.cap = cap;
        let (text, tone) = self.caption_text(cap);
        self.stage.caption = text;
        self.stage.tone = tone;
    }

    fn caption_text(&self, cap: Cap) -> (String, Tone) {
        let l = self.lang;
        let name = |side: Side| match side {
            Side::Player => self.stage.player.name.as_str(),
            Side::Opponent => self.stage.opponent.name.as_str(),
        };
        // Bom para o jogador quando quem se dá bem é o lado dele.
        let for_player = |side: Side| {
            if side == Side::Player {
                Tone::Good
            } else {
                Tone::Bad
            }
        };
        match cap {
            Cap::Other => (String::new(), Tone::Normal),
            Cap::Anim(Caption::Acted { side, action }) => (
                format!("{}: {}", name(side), t(l, narration_key(action))),
                Tone::Normal,
            ),
            Cap::Anim(Caption::Hit {
                attacker,
                damage,
                critical,
            }) => {
                let crit = if critical {
                    format!("{} ", t(l, Key::Critical))
                } else {
                    String::new()
                };
                (
                    format!(
                        "{}: {crit}{damage} {}",
                        name(attacker),
                        t(l, Key::DamageWord)
                    ),
                    for_player(attacker),
                )
            }
            Cap::Anim(Caption::Miss { attacker }) => (
                format!("{}: {}", name(attacker), t(l, Key::Missed)),
                for_player(attacker.other()),
            ),
            Cap::Anim(Caption::Recovered { side, amount }) => (
                format!("{}: +{amount} {}", name(side), t(l, Key::Stamina)),
                Tone::Normal,
            ),
            Cap::Anim(Caption::Knockout { loser }) => (
                format!("{} {}", name(loser), t(l, Key::Fell)),
                for_player(loser.other()),
            ),
            Cap::Anim(Caption::TimeUp) => (t(l, Key::TimeUp).to_owned(), Tone::Normal),
            Cap::Choose { turn } => (
                format!("{} {turn}: {}", t(l, Key::Round), t(l, Key::ChooseAction)),
                Tone::Prompt,
            ),
            Cap::Info { action, .. } => match &self.state.screen {
                Screen::Fight(f) => self.action_info(f, action),
                _ => (String::new(), Tone::Normal),
            },
            Cap::Auto { paused: true, .. } => (t(l, Key::Paused).to_owned(), Tone::Prompt),
            Cap::Auto { turn, .. } => (
                format!(
                    "{} {turn} - {}: {}",
                    t(l, Key::Round),
                    t(l, Key::Auto),
                    t(l, archetype_key(self.state.player_archetype()))
                ),
                Tone::Normal,
            ),
        }
    }

    /// Explicação de um botão de ação: custo e chance de acerto, ou por que não pode ser usado.
    fn action_info(&self, f: &FightState, action: Action) -> (String, Tone) {
        let l = self.lang;
        let label = t(l, action_key(action));
        let cost = f.stamina_cost(&self.content, Side::Player, action);
        if !f.is_valid(&self.content, Side::Player, action) {
            let reason = if f.fighter(Side::Player).stamina < cost {
                Key::NoStamina
            } else if action.is_attack() {
                Key::OutOfRange
            } else {
                Key::Unavailable
            };
            return (format!("{label}: {}", t(l, reason)), Tone::Bad);
        }
        let stamina = t(l, Key::Stamina);
        let text = match action {
            Action::Light | Action::Medium | Action::Heavy => {
                let hit = f.hit_chance(&self.content, Side::Player, action) / 10;
                format!("{label}: {cost} {stamina}, {hit}%")
            }
            Action::Advance | Action::Retreat => format!("{label}: {cost} {stamina}"),
            Action::Defend => format!("{label}: {}", t(l, Key::InfoDefend)),
            Action::Recover => format!("{label}: {}", t(l, Key::InfoRecover)),
            Action::Taunt => format!("{label}: {}", t(l, Key::InfoTaunt)),
        };
        (text, Tone::Normal)
    }

    /// Reconstrói a visão do palco a partir do estado (aloca; chamado só em mudanças de etapa).
    pub fn rebuild_stage_view(&mut self) {
        let l = self.lang;
        let cells = self.content.rules.arena.cells;
        let resolving = self.resolving();
        self.cap = Cap::Other;
        self.stage = match &self.state.screen {
            Screen::Fight(f) => self.fight_stage(f.opponent, cells),
            Screen::PostFight(sum) if resolving => self.fight_stage(sum.opponent, cells),
            Screen::PostFight(sum) => {
                let won = sum.outcome.winner == Side::Player;
                let title = if self.league_just_completed {
                    Key::LeagueDone
                } else if won {
                    Key::Victory
                } else {
                    Key::Defeat
                };
                let (mut player, mut opponent) = self.fighter_views(sum.opponent);
                if self.anim.is_none() {
                    // Save carregado direto no resultado: quem perdeu aparece caído.
                    if won {
                        opponent.frame = assets::Frame::Down;
                    } else {
                        player.frame = assets::Frame::Down;
                    }
                }
                StageView {
                    player,
                    opponent,
                    cells,
                    caption: t(l, title).to_owned(),
                    tone: if won { Tone::Good } else { Tone::Bad },
                    hint: String::new(),
                    phase: Phase::Result {
                        detail: format!(
                            "+{} {}  +{} {}",
                            sum.xp,
                            t(l, Key::Xp),
                            sum.gold,
                            t(l, Key::Gold)
                        ),
                        ok_label: t(l, Key::Continue).to_owned(),
                    },
                }
            }
            Screen::Hub => {
                let r = self.next_challenge();
                let (player, opponent) = self.fighter_views(r);
                let (caption, level) = match self
                    .content
                    .opponent(usize::from(r.league), usize::from(r.index))
                {
                    Some(o) => {
                        let rematch = self
                            .state
                            .league_progress
                            .get(usize::from(r.league))
                            .is_some_and(|&p| r.index < p);
                        let prefix = if rematch { Key::Training } else { Key::Next };
                        (format!("{}: {}", t(l, prefix), o.name), o.level)
                    }
                    None => (String::new(), 1),
                };
                let unspent = self.state.unspent_points;
                let what_now = match (unspent, self.state.mode) {
                    (1.., _) => format!("+{unspent} {}", t(l, Key::PointsInMenu)),
                    (0, Mode::Auto(_)) => t(l, Key::Auto).to_owned(),
                    (0, Mode::Manual) => t(l, Key::ClickFight).to_owned(),
                };
                StageView {
                    player,
                    opponent,
                    cells,
                    caption,
                    tone: Tone::Normal,
                    hint: format!("{} {level} - {what_now}", t(l, Key::Level)),
                    phase: Phase::Hub {
                        gold: self.state.gold,
                        unspent,
                        level_label: t(l, Key::Level).to_owned(),
                        points_label: t(l, Key::PointsShort).to_owned(),
                        fight_label: t(l, Key::Fight).to_owned(),
                    },
                }
            }
        };
        self.refresh_fight_view();
    }

    fn fight_stage(&self, r: OpponentRef, cells: u8) -> StageView {
        let (player, opponent) = self.fighter_views(r);
        StageView {
            player,
            opponent,
            cells,
            caption: String::new(),
            tone: Tone::Normal,
            hint: String::new(),
            phase: Phase::Fight {
                buttons: None,
                paused: self.paused,
            },
        }
    }

    /// Visão do painel (aloca; o painel não é animado).
    pub fn panel_view(&mut self) -> PanelView {
        let l = self.lang;
        let in_hub = matches!(self.state.screen, Screen::Hub | Screen::PostFight(_));
        let mut rows = Vec::new();
        let mut actions = Vec::new();
        let mut preview = None;
        let s = &self.state;
        let header;
        match self.panel_tab {
            0 => {
                let need = game::progression::xp_to_next(&self.content, s.player.level);
                header = format!(
                    "{} {}  {} {}/{}  {} {}",
                    t(l, Key::Level),
                    s.player.level,
                    t(l, Key::Xp),
                    s.xp,
                    need,
                    t(l, Key::Points),
                    s.unspent_points
                );
                for (attr, key) in ATTRS {
                    rows.push(PanelRow {
                        text: t(l, key).to_owned(),
                        value: s.player.attrs.get(attr).to_string(),
                        button: Some(("+".into(), s.unspent_points > 0 && in_hub)),
                        highlight: false,
                    });
                    actions.push(Some(RowAction::Allocate(attr)));
                }
                let mode = match s.mode {
                    Mode::Manual => t(l, Key::Manual).to_owned(),
                    Mode::Auto(a) => format!("{}: {}", t(l, Key::Auto), t(l, archetype_key(a))),
                };
                rows.push(PanelRow {
                    text: t(l, Key::Mode).to_owned(),
                    value: mode,
                    button: Some((t(l, Key::Change).to_owned(), true)),
                    highlight: true,
                });
                actions.push(Some(RowAction::CycleMode));
                rows.push(PanelRow {
                    text: t(l, Key::AutoAllocate).to_owned(),
                    value: on_off(l, self.config.auto_allocate),
                    button: Some((t(l, Key::Toggle).to_owned(), true)),
                    highlight: false,
                });
                actions.push(Some(RowAction::ToggleAutoAllocate));
            }
            1 => {
                header = format!("{} {}", t(l, Key::Gold), s.gold);
                for kind in [ItemKind::Weapon, ItemKind::Armor, ItemKind::Shield] {
                    let mut items: Vec<_> = self
                        .content
                        .items
                        .iter()
                        .filter(|i| i.kind == kind)
                        .collect();
                    items.sort_by_key(|i| (i.price, i.id));
                    for item in items {
                        let name = match l {
                            Lang::Pt => item.name.pt.clone(),
                            Lang::En => item.name.en.clone(),
                        };
                        let equipped = s.player.equipment.slot(kind) == Some(item.id);
                        let owned = s.inventory.contains(&item.id);
                        let stat = match kind {
                            ItemKind::Weapon => format!("+{}", item.damage),
                            ItemKind::Armor => format!("+{}", item.armor),
                            ItemKind::Shield => format!("+{}", item.shield),
                        };
                        let (label, enabled, action) = if equipped {
                            (t(l, Key::Equipped), false, None)
                        } else if owned {
                            (t(l, Key::Equip), in_hub, Some(RowAction::Equip(item.id)))
                        } else {
                            (
                                t(l, Key::Buy),
                                in_hub && s.gold >= item.price,
                                Some(RowAction::Buy(item.id)),
                            )
                        };
                        rows.push(PanelRow {
                            text: name,
                            value: format!("{stat} {}$", item.price),
                            button: Some((label.to_owned(), enabled)),
                            highlight: equipped,
                        });
                        actions.push(action);
                    }
                }
            }
            2 => {
                let league = self.next_challenge().league;
                let li = usize::from(league);
                let def = &self.content.leagues[li];
                header = match l {
                    Lang::Pt => def.name.pt.clone(),
                    Lang::En => def.name.en.clone(),
                };
                let progress = s.league_progress.get(li).copied().unwrap_or(0);
                let last = def.opponents.len().saturating_sub(1);
                for (i, o) in def.opponents.iter().enumerate() {
                    let r = OpponentRef {
                        league,
                        index: i as u16,
                    };
                    let status = if (i as u16) < progress {
                        t(l, Key::Beaten)
                    } else if i as u16 == progress {
                        t(l, Key::Next)
                    } else {
                        t(l, Key::Locked)
                    };
                    let champion = if i == last {
                        format!(" ({})", t(l, Key::Champion))
                    } else {
                        String::new()
                    };
                    let can = s.can_challenge(&self.content, r) && in_hub;
                    rows.push(PanelRow {
                        text: format!("{}{}", o.name, champion),
                        value: format!("{} {} {}", t(l, Key::Level), o.level, status),
                        button: Some((t(l, Key::Fight).to_owned(), can)),
                        highlight: i as u16 == progress,
                    });
                    actions.push(Some(RowAction::Challenge(r)));
                }
            }
            3 => {
                header = t(l, Key::LookHeader).to_owned();
                let a = s.appearance;
                let numbered = |v: u8, count: u8| format!("{}/{count}", v % count + 1);
                let entries: [(Key, String, LookPart); 5] = [
                    (Key::Skin, numbered(a.skin, looks::SKINS), LookPart::Skin),
                    (
                        Key::Hair,
                        t(l, HAIR_NAMES[usize::from(a.hair) % HAIR_NAMES.len()]).to_owned(),
                        LookPart::Hair,
                    ),
                    (
                        Key::HairColor,
                        numbered(a.hair_color, looks::HAIR_COLORS),
                        LookPart::HairColor,
                    ),
                    (Key::Eyes, numbered(a.eyes, looks::EYES), LookPart::Eyes),
                    (
                        Key::Beard,
                        t(l, BEARD_NAMES[usize::from(a.beard) % BEARD_NAMES.len()]).to_owned(),
                        LookPart::Beard,
                    ),
                ];
                for (key, value, part) in entries {
                    rows.push(PanelRow {
                        text: t(l, key).to_owned(),
                        value,
                        button: Some((t(l, Key::Change).to_owned(), true)),
                        highlight: false,
                    });
                    actions.push(Some(RowAction::CycleLook(part)));
                }
                preview = Some(self.player_look());
            }
            _ => {
                header = format!("v{}", env!("CARGO_PKG_VERSION"));
                let pace = match self.config.pace {
                    crate::config::Pace::Slow => t(l, Key::PaceSlow),
                    crate::config::Pace::Normal => t(l, Key::PaceNormal),
                    crate::config::Pace::Fast => t(l, Key::PaceFast),
                };
                let size = match self.config.size {
                    ..=-1 => t(l, Key::SizeSmall),
                    0 => t(l, Key::PaceNormal),
                    1.. => t(l, Key::SizeLarge),
                };
                let entries: [(Key, String, Key, RowAction); 7] = [
                    (
                        Key::Language,
                        match l {
                            Lang::Pt => "PT-BR".into(),
                            Lang::En => "EN".into(),
                        },
                        Key::Toggle,
                        RowAction::ToggleLang,
                    ),
                    (
                        Key::Pace,
                        pace.to_owned(),
                        Key::Change,
                        RowAction::CyclePace,
                    ),
                    (
                        Key::ReduceMotion,
                        on_off(l, self.config.reduce_motion),
                        Key::Toggle,
                        RowAction::ToggleReduceMotion,
                    ),
                    (
                        Key::StartWithWindows,
                        on_off(l, self.config.start_with_windows),
                        Key::Toggle,
                        RowAction::ToggleStartWithWindows,
                    ),
                    (
                        Key::Size,
                        size.to_owned(),
                        Key::Change,
                        RowAction::CycleSize,
                    ),
                    (
                        Key::Reposition,
                        String::new(),
                        Key::Ok,
                        RowAction::Reposition,
                    ),
                    (Key::Quit, String::new(), Key::Ok, RowAction::Quit),
                ];
                for (k, value, btn, action) in entries {
                    rows.push(PanelRow {
                        text: t(l, k).to_owned(),
                        value,
                        button: Some((t(l, btn).to_owned(), true)),
                        highlight: false,
                    });
                    actions.push(Some(action));
                }
            }
        }
        self.row_actions = actions;
        PanelView {
            tabs: vec![
                t(l, Key::TabSheet).to_owned(),
                t(l, Key::TabShop).to_owned(),
                t(l, Key::TabLeague).to_owned(),
                t(l, Key::TabLook).to_owned(),
                t(l, Key::TabConfig).to_owned(),
            ],
            active_tab: self.panel_tab,
            header,
            rows,
            footer: self
                .message
                .map_or_else(|| t(l, Key::ClickOutside), |k| t(l, k))
                .to_owned(),
            preview,
        }
    }
}

impl Effects {
    fn merged(mut self, o: Effects) -> Effects {
        self.merge(o);
        self
    }
}

fn on_off(l: Lang, v: bool) -> String {
    t(l, if v { Key::On } else { Key::Off }).to_owned()
}

fn blank_fighter() -> FighterView {
    standing_fighter(String::new(), 1, 0, Look::default())
}

/// Lutador parado em uma casa, fora de luta (sem barras de vida para mostrar).
fn standing_fighter(name: String, level: u8, cell: u8, look: Look) -> FighterView {
    FighterView {
        name,
        level,
        hp: 1,
        hp_max: 1,
        stamina: 1,
        stamina_max: 1,
        pos_pm: i32::from(cell) * 1000,
        frame: assets::Frame::Idle,
        flash: false,
        look,
        float: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use render::HitId;

    fn app() -> App {
        let c = match Content::embedded() {
            Ok(c) => c,
            Err(e) => panic!("{e}"),
        };
        let s = GameState::new(&c, "Teste", 7);
        App::new(c, s, Config::default(), Lang::Pt)
    }

    fn drain(a: &mut App) {
        let mut g = 0;
        while a.anim_busy() && g < 10_000 {
            a.on_anim_tick(33);
            g += 1;
        }
    }

    fn action_hit(a: Action) -> HitId {
        HitId::Action(Action::ALL.iter().position(|x| *x == a).unwrap_or(5) as u8)
    }

    #[test]
    fn manual_fight_from_hub_to_result() {
        let mut a = app();
        assert!(matches!(a.stage().phase, Phase::Hub { .. }));
        a.on_strip_hit(HitId::Fight);
        assert!(matches!(
            a.stage().phase,
            Phase::Fight {
                buttons: Some(_),
                ..
            }
        ));
        let mut rounds = 0;
        while matches!(a.state().screen, Screen::Fight(_)) {
            drain(&mut a);
            // Ação válida qualquer: avança se puder, senão ataque médio, senão defende.
            let f = match &a.state().screen {
                Screen::Fight(f) => f.clone(),
                _ => break,
            };
            let c = a.content().clone();
            let action = [
                Action::Medium,
                Action::Light,
                Action::Advance,
                Action::Defend,
            ]
            .into_iter()
            .find(|x| f.is_valid(&c, Side::Player, *x))
            .unwrap_or(Action::Defend);
            a.on_strip_hit(action_hit(action));
            rounds += 1;
            assert!(rounds < 100);
        }
        drain(&mut a);
        assert!(matches!(a.stage().phase, Phase::Result { .. }));
        a.on_strip_hit(HitId::Continue);
        assert!(matches!(a.stage().phase, Phase::Hub { .. }));
    }

    #[test]
    fn each_stage_tells_the_player_what_to_do() {
        let mut a = app();
        // Preparar: mostra o próximo oponente e os dois lutadores nas casas iniciais.
        let hub = a.stage().clone();
        assert!(hub.caption.starts_with("PRÓXIMO: Taro"), "{}", hub.caption);
        assert!(hub.hint.contains("PONTOS NO MENU"), "{}", hub.hint);
        assert_eq!(hub.opponent.name, "Taro");
        assert!(hub.player.pos_pm < hub.opponent.pos_pm);
        // Luta, vez do jogador: convite à ação, com o número da rodada.
        a.on_strip_hit(HitId::Fight);
        assert_eq!(a.stage().caption, "RODADA 1: ESCOLHA SUA AÇÃO");
        assert_eq!(a.stage().tone, Tone::Prompt);
        // Rodada em andamento: a legenda narra e os botões esperam.
        a.on_strip_hit(action_hit(Action::Advance));
        a.on_anim_tick(33);
        let during = a.stage().clone();
        // Quem tem a iniciativa age primeiro; as duas ações passam pela legenda, uma por vez.
        let mut narrated = vec![during.caption.clone()];
        let mut guard = 0;
        while a.resolving() && guard < 1000 {
            a.on_anim_tick(33);
            if narrated.last() != Some(&a.stage().caption) {
                narrated.push(a.stage().caption.clone());
            }
            guard += 1;
        }
        assert!(
            narrated.iter().any(|c| c == "Teste: AVANÇA"),
            "legendas da rodada: {narrated:?}"
        );
        assert!(
            narrated.iter().any(|c| c.starts_with("Taro: ")),
            "legendas da rodada: {narrated:?}"
        );
        assert!(matches!(
            during.phase,
            Phase::Fight {
                buttons: Some(b),
                ..
            } if b.iter().all(|s| *s == ButtonState::Disabled)
        ));
        // Fim da rodada: volta o convite, já na rodada seguinte.
        drain(&mut a);
        assert_eq!(a.stage().caption, "RODADA 2: ESCOLHA SUA AÇÃO");
    }

    #[test]
    fn hovering_an_action_explains_it() {
        let mut a = app();
        a.on_strip_hit(HitId::Fight);
        // No início os lutadores estão longe: golpe fora de alcance.
        let fx = a.set_hover(Some(action_hit(Action::Heavy)));
        assert!(fx.redraw_strip);
        assert_eq!(a.stage().caption, "GOLPE PESADO: FORA DE ALCANCE");
        assert_eq!(a.stage().tone, Tone::Bad);
        // Clicar em ação inválida não gasta a rodada.
        a.on_strip_hit(action_hit(Action::Heavy));
        assert!(matches!(&a.state().screen, Screen::Fight(f) if f.turn == 0));
        a.set_hover(Some(action_hit(Action::Defend)));
        assert_eq!(a.stage().caption, "DEFENDER: REDUZ O DANO RECEBIDO");
        // O mouse saiu: volta o convite.
        a.set_hover(None);
        assert_eq!(a.stage().caption, "RODADA 1: ESCOLHA SUA AÇÃO");
        assert!(!a.set_hover(None).redraw_strip);
    }

    #[test]
    fn captions_fit_the_stage() {
        // A legenda tem 31 caracteres úteis; nenhuma explicação de ação pode ser cortada.
        for lang in [Lang::Pt, Lang::En] {
            let mut a = app();
            a.lang = lang;
            a.on_strip_hit(HitId::Fight);
            for round in 0..6 {
                for action in Action::ALL {
                    a.set_hover(Some(action_hit(action)));
                    let n = a.stage().caption.chars().count();
                    assert!(n <= 31, "{lang:?} rodada {round}: {:?}", a.stage().caption);
                }
                a.set_hover(None);
                a.on_strip_hit(action_hit(Action::Advance));
                drain(&mut a);
            }
        }
    }

    #[test]
    fn auto_mode_runs_by_itself() {
        let mut a = app();
        let mut pv = a.panel_view();
        let mode_row = pv
            .rows
            .iter()
            .position(|r| r.text == t(Lang::Pt, Key::Mode))
            .unwrap_or(0) as u16;
        a.on_panel_hit(HitId::Row(mode_row));
        assert!(matches!(a.state().mode, Mode::Auto(_)));
        assert!(a.auto_delay_ms().is_some());
        let mut fights = 0;
        for _ in 0..5_000 {
            drain(&mut a);
            let before = a.state().stats.fights;
            a.on_auto_tick();
            if a.state().stats.fights > before {
                fights += 1;
            }
            if fights >= 3 {
                break;
            }
        }
        assert!(fights >= 3, "o modo automático não completou lutas");
        assert_eq!(
            a.state().unspent_points,
            0,
            "pontos deveriam ser distribuídos sozinhos"
        );
        pv = a.panel_view();
        assert_eq!(pv.active_tab, 0);
    }

    #[test]
    fn shop_buy_from_panel() {
        let mut a = app();
        a.on_panel_hit(HitId::Tab(1));
        let pv = a.panel_view();
        // Sem ouro: botões de compra (exceto os de preço 0) desabilitados.
        assert!(pv.rows.iter().all(|r| match &r.button {
            Some((label, enabled)) if label == t(Lang::Pt, Key::Buy) =>
                !enabled || r.value.ends_with(" 0$"),
            _ => true,
        }));
    }

    #[test]
    fn look_tab_changes_the_fighter_on_stage() {
        let mut a = app();
        a.on_panel_hit(HitId::Tab(3));
        let pv = a.panel_view();
        assert_eq!(pv.preview, Some(a.stage().player.look));
        let before = a.stage().player.look;
        // Linha 1 = cabelo, linha 4 = barba.
        let fx = a.on_panel_hit(HitId::Row(1));
        assert!(fx.redraw_strip && fx.save_soon);
        a.on_panel_hit(HitId::Row(4));
        let after = a.stage().player.look;
        assert_ne!(after.hair, before.hair);
        assert_ne!(after.beard, before.beard);
        assert_eq!(a.panel_view().preview, Some(after));
        // Dar a volta em todas as opções retorna ao começo.
        for _ in 0..looks::HAIRS - 1 {
            a.on_panel_hit(HitId::Row(1));
        }
        assert_eq!(a.stage().player.look.hair, before.hair);
        // A aparência vai para o save e volta.
        let bytes = match game::save::to_bytes(a.content(), a.state()) {
            Ok(b) => b,
            Err(e) => panic!("{e}"),
        };
        let loaded = match game::save::from_bytes(a.content(), &bytes) {
            Ok(l) => l,
            Err(e) => panic!("{e}"),
        };
        assert_eq!(loaded.state.appearance, a.state().appearance);
    }

    #[test]
    fn size_setting_asks_for_relayout() {
        let mut a = app();
        a.on_panel_hit(HitId::Tab(4));
        let pv = a.panel_view();
        let row = pv
            .rows
            .iter()
            .position(|r| r.text == t(Lang::Pt, Key::Size))
            .unwrap_or(0) as u16;
        let fx = a.on_panel_hit(HitId::Row(row));
        assert!(fx.relayout && fx.save_config);
        assert_eq!(a.config().size, 1);
    }

    #[test]
    fn language_toggle_changes_stage_text() {
        let mut a = app();
        a.on_panel_hit(HitId::Tab(4));
        let _ = a.panel_view();
        a.on_panel_hit(HitId::Row(0));
        assert_eq!(a.lang(), Lang::En);
        match &a.stage().phase {
            Phase::Hub { fight_label, .. } => assert_eq!(fight_label, "FIGHT"),
            other => panic!("{other:?}"),
        }
    }
}
