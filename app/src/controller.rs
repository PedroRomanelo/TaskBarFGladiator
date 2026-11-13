//! Controlador do aplicativo: liga o estado do jogo, a animação e as visões de renderização
//! (portável, testado sem Windows).
//!
//! A camada Windows chama os métodos `on_*` e aplica os [`Effects`] devolvidos (redesenhar, abrir
//! painel, salvar, sair). Os temporizadores são definidos a partir de [`App::anim_busy`] e
//! [`App::auto_delay_ms`].

use crate::anim::{Animator, Banner};
use crate::config::Config;
use crate::i18n::{Key, Lang, t};
use game::{
    Action, Archetype, Attr, Content, GameEvent, GameState, Input, ItemKind, Mode, OpponentRef,
    Rejection, SaveHint, Screen, Side, step,
};
use render::{ButtonState, FighterView, HitId, PanelRow, PanelView, StripView};

/// Efeitos pedidos à camada de plataforma.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Effects {
    /// Redesenhar a faixa.
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
    /// Reposicionar a faixa no padrão.
    pub reset_position: bool,
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
    ToggleLang,
    ToggleReduceMotion,
    ToggleStartWithWindows,
    Reposition,
    CyclePace,
    Quit,
}

/// Tempo de exibição do resultado no modo automático (ms).
const RESULT_MS: u32 = 3_000;

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
    strip_view: StripView,
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
            strip_view: StripView::Hub {
                name: String::new(),
                level: 1,
                gold: 0,
                unspent: 0,
                next_label: String::new(),
                fight_label: String::new(),
            },
            league_just_completed: false,
        };
        if let Screen::Fight(f) = &app.state.screen {
            app.anim = Some(Animator::new(f, app.config.reduce_motion));
        }
        app.rebuild_strip_view();
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
        self.rebuild_strip_view();
        Effects {
            redraw_strip: true,
            ..Default::default()
        }
    }

    /// Verdadeiro enquanto há animação (temporizador de quadros ligado).
    pub fn anim_busy(&self) -> bool {
        self.anim.as_ref().is_some_and(Animator::busy)
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

    /// Visão atual da faixa.
    pub fn strip_view(&self) -> &StripView {
        &self.strip_view
    }

    /// Avança a animação. Retorna os efeitos (redesenho).
    pub fn on_anim_tick(&mut self, dt_ms: u32) -> Effects {
        let changed = self.anim.as_mut().is_some_and(|a| a.tick(dt_ms));
        if changed {
            self.refresh_fight_view();
        }
        // A última rodada terminou de animar: mostra o resultado.
        if !self.anim_busy()
            && matches!(self.state.screen, Screen::PostFight(_))
            && matches!(self.strip_view, StripView::Fight { .. })
        {
            self.rebuild_strip_view();
            return Effects {
                redraw_strip: true,
                ..Default::default()
            };
        }
        Effects {
            redraw_strip: changed,
            ..Default::default()
        }
    }

    /// Ação automática (temporizador do modo automático).
    pub fn on_auto_tick(&mut self) -> Effects {
        if self.anim_busy() || self.paused {
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

    /// Clique na faixa.
    pub fn on_strip_hit(&mut self, id: HitId) -> Effects {
        match id {
            HitId::Action(i) => {
                let Some(&action) = Action::ALL.get(usize::from(i)) else {
                    return Effects::default();
                };
                if self.anim_busy() {
                    // Termina a animação atual para responder ao clique sem atraso.
                    if let Some(a) = &mut self.anim {
                        a.finish();
                    }
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
                self.panel_tab = usize::from(i).min(3);
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
            RowAction::ToggleLang => {
                self.lang = self.lang.toggle();
                self.config.language = Some(self.lang);
                self.rebuild_strip_view();
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
        self.rebuild_strip_view();
        fx
    }

    fn refresh_fight_view(&mut self) {
        if let (
            Some(a),
            StripView::Fight {
                player,
                opponent,
                banner,
                ..
            },
        ) = (&self.anim, &mut self.strip_view)
        {
            a.fill_view(Side::Player, player);
            a.fill_view(Side::Opponent, opponent);
            let text = a.banner().map(|b| match b {
                Banner::Critical => t(self.lang, Key::Critical),
                Banner::Miss => t(self.lang, Key::Miss),
            });
            match (text, banner.as_mut()) {
                (Some(tx), Some(b)) if b == tx => {}
                (Some(tx), _) => *banner = Some(tx.to_owned()),
                (None, _) => *banner = None,
            }
        }
        self.refresh_buttons();
    }

    fn refresh_buttons(&mut self) {
        let busy = self.anim_busy();
        let manual = self.state.mode == Mode::Manual;
        let fight = match &self.state.screen {
            Screen::Fight(f) => Some(f),
            _ => None,
        };
        if let StripView::Fight {
            buttons, paused, ..
        } = &mut self.strip_view
        {
            *paused = self.paused;
            *buttons = match (manual, fight) {
                (true, Some(f)) => {
                    let mut states = [ButtonState::Disabled; 8];
                    for (i, a) in Action::ALL.iter().enumerate() {
                        if !busy && f.is_valid(&self.content, Side::Player, *a) {
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

    /// Reconstrói a visão da faixa a partir do estado (aloca; chamado só em mudanças de tela).
    pub fn rebuild_strip_view(&mut self) {
        let l = self.lang;
        self.strip_view = match &self.state.screen {
            Screen::Fight(_) | Screen::PostFight(_)
                if self.anim_busy() || matches!(self.state.screen, Screen::Fight(_)) =>
            {
                let (p, o) = match &self.anim {
                    Some(a) => (a.view(Side::Player), a.view(Side::Opponent)),
                    None => (empty_fighter(), empty_fighter()),
                };
                StripView::Fight {
                    player: p,
                    opponent: o,
                    cells: self.content.rules.arena.cells,
                    buttons: None,
                    paused: self.paused,
                    banner: None,
                }
            }
            Screen::PostFight(sum) => {
                let won = sum.outcome.winner == Side::Player;
                let title = if self.league_just_completed {
                    t(l, Key::LeagueDone).to_owned()
                } else if won {
                    t(l, Key::Victory).to_owned()
                } else {
                    t(l, Key::Defeat).to_owned()
                };
                StripView::Result {
                    title,
                    detail: format!(
                        "+{} {}  +{} {}",
                        sum.xp,
                        t(l, Key::Xp),
                        sum.gold,
                        t(l, Key::Gold)
                    ),
                    won,
                    ok_label: t(l, Key::Ok).to_owned(),
                }
            }
            _ => {
                let r = self.next_challenge();
                let next_label = match self
                    .content
                    .opponent(usize::from(r.league), usize::from(r.index))
                {
                    Some(o) => {
                        let rematch = self
                            .state
                            .league_progress
                            .get(usize::from(r.league))
                            .is_some_and(|&p| r.index < p);
                        let prefix = if rematch {
                            t(l, Key::Training)
                        } else {
                            t(l, Key::Next)
                        };
                        format!("{prefix}: {} ({} {})", o.name, t(l, Key::Level), o.level)
                    }
                    None => String::new(),
                };
                StripView::Hub {
                    name: self.state.player.name.clone(),
                    level: self.state.player.level,
                    gold: self.state.gold,
                    unspent: self.state.unspent_points,
                    next_label,
                    fight_label: t(l, Key::Fight).to_owned(),
                }
            }
        };
        self.refresh_fight_view();
    }

    /// Visão do painel (aloca; o painel não é animado).
    pub fn panel_view(&mut self) -> PanelView {
        let l = self.lang;
        let in_hub = matches!(self.state.screen, Screen::Hub | Screen::PostFight(_));
        let mut rows = Vec::new();
        let mut actions = Vec::new();
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
            _ => {
                header = format!("v{}", env!("CARGO_PKG_VERSION"));
                let pace = match self.config.pace {
                    crate::config::Pace::Slow => t(l, Key::PaceSlow),
                    crate::config::Pace::Normal => t(l, Key::PaceNormal),
                    crate::config::Pace::Fast => t(l, Key::PaceFast),
                };
                let entries: [(Key, String, Key, RowAction); 6] = [
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
                t(l, Key::TabConfig).to_owned(),
            ],
            active_tab: self.panel_tab,
            header,
            rows,
            footer: self
                .message
                .map_or_else(|| t(l, Key::ClickOutside), |k| t(l, k))
                .to_owned(),
        }
    }
}

fn on_off(l: Lang, v: bool) -> String {
    t(l, if v { Key::On } else { Key::Off }).to_owned()
}

fn empty_fighter() -> FighterView {
    FighterView {
        name: String::new(),
        level: 1,
        hp: 0,
        hp_max: 1,
        stamina: 0,
        stamina_max: 1,
        pos_pm: 0,
        frame: assets::Frame::Idle,
        flash: false,
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

    #[test]
    fn manual_fight_from_hub_to_result() {
        let mut a = app();
        assert!(matches!(a.strip_view(), StripView::Hub { .. }));
        a.on_strip_hit(HitId::Fight);
        assert!(matches!(
            a.strip_view(),
            StripView::Fight {
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
            let idx = Action::ALL.iter().position(|x| *x == action).unwrap_or(5) as u8;
            a.on_strip_hit(HitId::Action(idx));
            rounds += 1;
            assert!(rounds < 100);
        }
        drain(&mut a);
        assert!(matches!(a.strip_view(), StripView::Result { .. }));
        a.on_strip_hit(HitId::Continue);
        assert!(matches!(a.strip_view(), StripView::Hub { .. }));
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
    fn language_toggle_changes_strip_text() {
        let mut a = app();
        a.on_panel_hit(HitId::Tab(3));
        let _ = a.panel_view();
        a.on_panel_hit(HitId::Row(0));
        assert_eq!(a.lang(), Lang::En);
        match a.strip_view() {
            StripView::Hub { fight_label, .. } => assert_eq!(fight_label, "FIGHT"),
            other => panic!("{other:?}"),
        }
    }
}
