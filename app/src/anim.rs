//! Animação da luta: converte eventos de combate em passos com duração (portável).
//!
//! O estado do jogo muda de uma vez por rodada; a animação mostra a rodada em passos (quem age, o
//! movimento, o golpe, o dano) e, ao final, sincroniza os valores exibidos com o estado real. Cada
//! passo traz uma [`Caption`]: é ela que a interface transforma na legenda do palco, para o
//! jogador acompanhar o que aconteceu e em que ordem.

use assets::Frame;
use game::combat::{EndReason, FightEvent};
use game::{Action, FightState, Side};
use render::{FighterView, FloatText, Floater, Look};
use std::collections::VecDeque;

/// Duração dos passos (ms).
const MOVE_MS: u32 = 260;
/// Tempo em que a legenda "fulano: golpe" fica visível antes do golpe sair.
const WINDUP_MS: u32 = 320;
const ATTACK_MS: u32 = 200;
/// Pausa depois de um acerto ou erro, para o resultado ser lido.
const RESULT_MS: u32 = 460;
/// Defender, recuperar, provocar e o anúncio de um movimento.
const POSE_MS: u32 = 460;
const STEP_MS: u32 = 140;
const END_MS: u32 = 800;
const FLASH_MS: u32 = 160;
/// Vida de um texto flutuante e quantas linhas lógicas ele sobe nesse tempo.
const FLOAT_MS: u32 = 900;
const FLOAT_RISE: u32 = 7;
/// Tempo de pose usado quando as animações estão reduzidas: as poses de uma vez só (queda) já
/// aparecem no último quadro.
pub const POSE_HOLD_MS: u32 = 1_000_000;

/// O que está acontecendo agora na luta (a interface traduz para a legenda do palco).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Caption {
    /// Um lutador anuncia e executa uma ação.
    Acted {
        /// Lado.
        side: Side,
        /// Ação.
        action: Action,
    },
    /// Um golpe acertou.
    Hit {
        /// Atacante.
        attacker: Side,
        /// Dano.
        damage: i32,
        /// Crítico.
        critical: bool,
    },
    /// Um golpe errou.
    Miss {
        /// Atacante.
        attacker: Side,
    },
    /// Fôlego recuperado.
    Recovered {
        /// Lado.
        side: Side,
        /// Quantidade.
        amount: i32,
    },
    /// A luta acabou por nocaute.
    Knockout {
        /// Quem caiu.
        loser: Side,
    },
    /// A luta acabou pelo limite de turnos.
    TimeUp,
}

/// Texto flutuante ainda não traduzido.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FloatKind {
    Damage { amount: i32, critical: bool },
    Stamina(i32),
    Miss,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Visual {
    pos_pm: i32,
    hp: i32,
    hp_max: i32,
    stamina: i32,
    stamina_max: i32,
    frame: Frame,
    /// Tempo desde o início da pose atual.
    pose_ms: u32,
    flash_ms: u32,
    float: Option<(FloatKind, u32)>,
}

impl Visual {
    /// Troca de pose; o relógio da pose só recomeça se ela mudou.
    fn pose(&mut self, frame: Frame) {
        if self.frame != frame {
            self.frame = frame;
            self.pose_ms = 0;
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StepKind {
    Move {
        side: Side,
        to_pm: i32,
    },
    Pose {
        side: Side,
        frame: Frame,
    },
    Say(Caption),
    Hit {
        target: Side,
        damage: i32,
        critical: bool,
    },
    Miss {
        attacker: Side,
    },
    Float {
        side: Side,
        kind: FloatKind,
    },
    Sync,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Step {
    kind: StepKind,
    duration_ms: u32,
}

/// Animador de uma luta.
#[derive(Debug, Clone)]
pub struct Animator {
    vis: [Visual; 2],
    levels: [u8; 2],
    queue: VecDeque<Step>,
    current: Option<(Step, u32, i32)>,
    caption: Option<Caption>,
    target: Option<Box<FightState>>,
    reduce_motion: bool,
}

fn visual_of(f: &FightState, side: Side) -> Visual {
    let c = f.fighter(side);
    Visual {
        pos_pm: i32::from(c.pos) * 1000,
        hp: c.hp,
        hp_max: c.stats.hp_max,
        stamina: c.stamina,
        stamina_max: c.stamina_cap,
        frame: if c.hp == 0 {
            Frame::Down
        } else if c.stance == game::combat::Stance::Defend {
            Frame::Defend
        } else {
            Frame::Idle
        },
        pose_ms: POSE_HOLD_MS,
        flash_ms: 0,
        float: None,
    }
}

impl Animator {
    /// Cria o animador sincronizado com a luta.
    pub fn new(fight: &FightState, reduce_motion: bool) -> Self {
        Self {
            vis: [
                visual_of(fight, Side::Player),
                visual_of(fight, Side::Opponent),
            ],
            levels: [
                fight.fighter(Side::Player).level,
                fight.fighter(Side::Opponent).level,
            ],
            queue: VecDeque::with_capacity(24),
            current: None,
            caption: None,
            target: None,
            reduce_motion,
        }
    }

    /// Liga/desliga a redução de animações.
    pub fn set_reduce_motion(&mut self, on: bool) {
        self.reduce_motion = on;
    }

    /// Enfileira os passos de uma rodada. `after` é o estado da luta depois da rodada.
    pub fn push_round(&mut self, events: &[FightEvent], after: &FightState) {
        self.push_events(events);
        self.target = Some(Box::new(after.clone()));
        self.push(StepKind::Sync, 0);
        if self.reduce_motion {
            self.finish();
        }
    }

    fn push_events(&mut self, events: &[FightEvent]) {
        let mut pending_attack: Option<Side> = None;
        for e in events {
            match *e {
                FightEvent::Acted { side, action } => {
                    // A postura anterior termina quando o lutador age de novo.
                    self.push(
                        StepKind::Pose {
                            side,
                            frame: Frame::Idle,
                        },
                        0,
                    );
                    let say = StepKind::Say(Caption::Acted { side, action });
                    match action {
                        Action::Light | Action::Medium | Action::Heavy => {
                            let frame = match action {
                                Action::Light => Frame::Attack,
                                Action::Medium => Frame::AttackMedium,
                                _ => Frame::AttackHeavy,
                            };
                            self.push(say, WINDUP_MS);
                            self.push(StepKind::Pose { side, frame }, ATTACK_MS);
                            pending_attack = Some(side);
                        }
                        Action::Defend => {
                            self.push(
                                StepKind::Pose {
                                    side,
                                    frame: Frame::Defend,
                                },
                                0,
                            );
                            self.push(say, POSE_MS);
                        }
                        Action::Advance | Action::Retreat => self.push(say, STEP_MS),
                        Action::Recover | Action::Taunt => self.push(say, POSE_MS),
                    }
                }
                FightEvent::Moved { side, to } => {
                    self.push(
                        StepKind::Move {
                            side,
                            to_pm: i32::from(to) * 1000,
                        },
                        MOVE_MS,
                    );
                }
                FightEvent::Hit {
                    attacker,
                    damage,
                    critical,
                } => {
                    self.push(
                        StepKind::Hit {
                            target: attacker.other(),
                            damage,
                            critical,
                        },
                        RESULT_MS,
                    );
                    if pending_attack.take() == Some(attacker) {
                        self.push(
                            StepKind::Pose {
                                side: attacker,
                                frame: Frame::Idle,
                            },
                            0,
                        );
                    }
                }
                FightEvent::Miss { attacker } => {
                    self.push(StepKind::Miss { attacker }, RESULT_MS);
                    if pending_attack.take() == Some(attacker) {
                        self.push(
                            StepKind::Pose {
                                side: attacker,
                                frame: Frame::Idle,
                            },
                            0,
                        );
                    }
                }
                FightEvent::Recovered { side, amount } => {
                    self.push(
                        StepKind::Float {
                            side,
                            kind: FloatKind::Stamina(amount),
                        },
                        0,
                    );
                    self.push(StepKind::Say(Caption::Recovered { side, amount }), 0);
                }
                FightEvent::Ended(outcome) => {
                    let loser = outcome.winner.other();
                    let caption = match outcome.reason {
                        EndReason::Knockout => {
                            self.push(
                                StepKind::Pose {
                                    side: loser,
                                    frame: Frame::Down,
                                },
                                0,
                            );
                            Caption::Knockout { loser }
                        }
                        EndReason::TurnLimit => Caption::TimeUp,
                    };
                    self.push(StepKind::Say(caption), END_MS);
                }
                FightEvent::Initiative(_) | FightEvent::Converted { .. } => {}
            }
        }
    }

    /// Enfileira a última rodada de uma luta que terminou (o estado final da luta não existe mais
    /// no jogo; os valores exibidos vêm dos próprios eventos).
    pub fn push_round_final(&mut self, events: &[FightEvent]) {
        self.target = None;
        self.push_events(events);
        if self.reduce_motion {
            self.finish();
        }
    }

    fn push(&mut self, kind: StepKind, duration_ms: u32) {
        let duration_ms = if self.reduce_motion { 0 } else { duration_ms };
        self.queue.push_back(Step { kind, duration_ms });
    }

    /// Termina tudo imediatamente.
    pub fn finish(&mut self) {
        if let Some((step, _, _)) = self.current.take() {
            self.apply_end(step);
        }
        while let Some(step) = self.queue.pop_front() {
            self.apply_start(step);
            self.apply_end(step);
        }
        for v in &mut self.vis {
            v.flash_ms = 0;
            v.float = None;
        }
    }

    /// Verdadeiro enquanto há passos pendentes (o temporizador de animação deve continuar ligado).
    pub fn busy(&self) -> bool {
        self.current.is_some()
            || !self.queue.is_empty()
            || self.vis.iter().any(|v| v.flash_ms > 0 || v.float.is_some())
    }

    /// Verdadeiro enquanto a rodada ainda está sendo mostrada (os textos flutuantes que sobram no
    /// fim não contam: o jogador já pode escolher a próxima ação).
    pub fn resolving(&self) -> bool {
        self.current.is_some() || !self.queue.is_empty()
    }

    /// Avança `dt_ms`. Retorna `true` se algo visível mudou.
    pub fn tick(&mut self, dt_ms: u32) -> bool {
        let dt = dt_ms.min(100); // após suspensão o dt é limitado
        let mut changed = false;
        for v in &mut self.vis {
            v.pose_ms = v.pose_ms.saturating_add(dt).min(POSE_HOLD_MS);
            if v.flash_ms > 0 {
                v.flash_ms = v.flash_ms.saturating_sub(dt);
                changed = true;
            }
            if let Some((kind, age)) = v.float {
                let age = age + dt;
                v.float = (age < FLOAT_MS).then_some((kind, age));
                changed = true;
            }
        }
        let mut budget = dt;
        loop {
            if self.current.is_none() {
                let Some(step) = self.queue.pop_front() else {
                    break;
                };
                let from = self.start_pos(step);
                self.apply_start(step);
                self.current = Some((step, 0, from));
                changed = true;
            }
            let Some((step, elapsed, from)) = self.current else {
                break;
            };
            let left = step.duration_ms.saturating_sub(elapsed);
            if left > budget {
                let e = elapsed + budget;
                self.current = Some((step, e, from));
                self.interpolate(step, e, from);
                changed = true;
                break;
            }
            budget -= left;
            self.current = None;
            self.apply_end(step);
            changed = true;
            if budget == 0 && !self.queue.is_empty() {
                break;
            }
        }
        changed
    }

    fn start_pos(&self, step: Step) -> i32 {
        match step.kind {
            StepKind::Move { side, .. } => self.vis[side.index()].pos_pm,
            _ => 0,
        }
    }

    fn float(&mut self, side: Side, kind: FloatKind) {
        if !self.reduce_motion {
            self.vis[side.index()].float = Some((kind, 0));
        }
    }

    fn apply_start(&mut self, step: Step) {
        match step.kind {
            StepKind::Pose { side, frame } => self.vis[side.index()].pose(frame),
            StepKind::Say(c) => self.caption = Some(c),
            StepKind::Move { side, .. } => {
                let v = &mut self.vis[side.index()];
                if v.frame == Frame::Idle {
                    v.pose(Frame::Walk);
                }
            }
            StepKind::Hit {
                target,
                damage,
                critical,
            } => {
                let v = &mut self.vis[target.index()];
                v.hp = (v.hp - damage).max(0);
                v.flash_ms = if self.reduce_motion { 0 } else { FLASH_MS };
                // Quem está em guarda apara com o escudo; quem não está, sente o golpe.
                match v.frame {
                    Frame::Defend | Frame::Block => {
                        v.frame = Frame::Block;
                        v.pose_ms = 0;
                    }
                    Frame::Idle | Frame::Walk | Frame::Hurt => {
                        v.frame = Frame::Hurt;
                        v.pose_ms = 0;
                    }
                    _ => {}
                }
                self.float(
                    target,
                    FloatKind::Damage {
                        amount: damage,
                        critical,
                    },
                );
                self.caption = Some(Caption::Hit {
                    attacker: target.other(),
                    damage,
                    critical,
                });
            }
            StepKind::Miss { attacker } => {
                self.float(attacker, FloatKind::Miss);
                self.caption = Some(Caption::Miss { attacker });
            }
            StepKind::Float { side, kind } => self.float(side, kind),
            StepKind::Sync => {}
        }
    }

    fn interpolate(&mut self, step: Step, elapsed: u32, from: i32) {
        if let StepKind::Move { side, to_pm } = step.kind
            && step.duration_ms > 0
        {
            let t = i64::from(elapsed) * 1000 / i64::from(step.duration_ms);
            let pos = i64::from(from) + (i64::from(to_pm) - i64::from(from)) * t / 1000;
            self.vis[side.index()].pos_pm = pos as i32;
        }
    }

    fn apply_end(&mut self, step: Step) {
        match step.kind {
            StepKind::Move { side, to_pm } => {
                let v = &mut self.vis[side.index()];
                v.pos_pm = to_pm;
                if v.frame == Frame::Walk {
                    v.pose(Frame::Idle);
                }
            }
            StepKind::Hit { target, .. } => {
                // Fim da reação ao golpe: volta à postura em que estava.
                let v = &mut self.vis[target.index()];
                match v.frame {
                    Frame::Hurt => v.pose(Frame::Idle),
                    Frame::Block => v.pose(Frame::Defend),
                    _ => {}
                }
            }
            StepKind::Sync => {
                // Fim da rodada: a legenda volta a ser o convite para a próxima ação.
                self.caption = None;
                if let Some(t) = self.target.take() {
                    for side in [Side::Player, Side::Opponent] {
                        let real = visual_of(&t, side);
                        let v = &mut self.vis[side.index()];
                        v.pos_pm = real.pos_pm;
                        v.hp = real.hp;
                        v.hp_max = real.hp_max;
                        v.stamina = real.stamina;
                        v.stamina_max = real.stamina_max;
                        if real.frame == Frame::Down {
                            v.pose(Frame::Down);
                        }
                    }
                }
            }
            StepKind::Pose { .. }
            | StepKind::Say(_)
            | StepKind::Miss { .. }
            | StepKind::Float { .. } => {}
        }
    }

    /// O que está acontecendo agora (`None` entre rodadas).
    pub fn caption(&self) -> Option<Caption> {
        self.caption
    }

    /// Tempo de pose exibido (com animações reduzidas, sempre o quadro final).
    fn shown_pose_ms(&self, v: &Visual) -> u32 {
        if self.reduce_motion {
            POSE_HOLD_MS
        } else {
            v.pose_ms
        }
    }

    /// Atualiza só o tempo de pose de uma visão (relógio das animações paradas).
    pub fn fill_pose_ms(&self, side: Side, out: &mut FighterView) {
        out.anim_ms = self.shown_pose_ms(&self.vis[side.index()]);
    }

    fn floater(v: &Visual, miss_label: &'static str) -> Option<Floater> {
        v.float.map(|(kind, age)| Floater {
            text: match kind {
                FloatKind::Damage { amount, critical } => FloatText::Damage { amount, critical },
                FloatKind::Stamina(n) => FloatText::Stamina(n),
                FloatKind::Miss => FloatText::Label(miss_label),
            },
            rise: (age * FLOAT_RISE / FLOAT_MS) as i32,
        })
    }

    /// Atualiza uma visão existente sem alocar (o nome e a aparência não são tocados).
    /// `miss_label` é o texto traduzido do erro de golpe.
    pub fn fill_view(&self, side: Side, out: &mut FighterView, miss_label: &'static str) {
        let v = &self.vis[side.index()];
        out.level = self.levels[side.index()];
        out.hp = v.hp;
        out.hp_max = v.hp_max;
        out.stamina = v.stamina;
        out.stamina_max = v.stamina_max;
        out.pos_pm = v.pos_pm;
        out.frame = v.frame;
        out.anim_ms = self.shown_pose_ms(v);
        out.flash = v.flash_ms > 0;
        out.float = Self::floater(v, miss_label);
    }

    /// Visão de um lutador para a renderização, com o nome curto e a aparência dados.
    pub fn view(&self, side: Side, name: String, look: Look) -> FighterView {
        let v = &self.vis[side.index()];
        FighterView {
            name,
            level: self.levels[side.index()],
            hp: v.hp,
            hp_max: v.hp_max,
            stamina: v.stamina,
            stamina_max: v.stamina_max,
            pos_pm: v.pos_pm,
            frame: v.frame,
            anim_ms: self.shown_pose_ms(v),
            flash: v.flash_ms > 0,
            look,
            float: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use game::{Archetype, Content, Fighter, OpponentRef};

    fn fight() -> (Content, FightState) {
        let c = match Content::embedded() {
            Ok(c) => c,
            Err(e) => panic!("{e}"),
        };
        let f = Fighter {
            name: "A".into(),
            level: 5,
            attrs: Fighter::distribute(&c, [1; 6], 5),
            equipment: Default::default(),
        };
        let fs = FightState::new(
            &c,
            &f,
            Archetype::Balanced,
            &f,
            Archetype::Aggressive,
            OpponentRef {
                league: 0,
                index: 0,
            },
            false,
            3,
        );
        (c, fs)
    }

    fn view(a: &Animator, side: Side) -> FighterView {
        a.view(side, String::new(), Look::default())
    }

    #[test]
    fn animation_converges_to_real_state() {
        let (c, mut f) = fight();
        let mut a = Animator::new(&f, false);
        let mut ev = Vec::new();
        while f.outcome.is_none() {
            ev.clear();
            let p = game::ai::choose_action(&c, &mut f, Side::Player);
            let o = game::ai::choose_action(&c, &mut f, Side::Opponent);
            f.play_round(&c, p, o, &mut ev);
            a.push_round(&ev, &f);
            let mut guard = 0;
            while a.busy() {
                a.tick(33);
                guard += 1;
                assert!(guard < 1000, "animação não termina");
            }
            for side in [Side::Player, Side::Opponent] {
                let v = view(&a, side);
                let real = f.fighter(side);
                assert_eq!(v.hp, real.hp);
                assert_eq!(v.pos_pm, i32::from(real.pos) * 1000);
            }
        }
        let loser = f.outcome.map(|o| o.winner.other());
        if let Some(l) = loser
            && f.fighter(l).hp == 0
        {
            assert_eq!(view(&a, l).frame, Frame::Down);
        }
    }

    #[test]
    fn every_round_is_narrated_and_ends_without_caption() {
        let (c, mut f) = fight();
        let mut a = Animator::new(&f, false);
        let mut ev = Vec::new();
        f.play_round(&c, Action::Advance, Action::Advance, &mut ev);
        a.push_round(&ev, &f);
        let mut seen = Vec::new();
        let mut guard = 0;
        while a.busy() {
            a.tick(33);
            if let Some(cap) = a.caption()
                && seen.last() != Some(&cap)
            {
                seen.push(cap);
            }
            guard += 1;
            assert!(guard < 1000, "animação não termina");
        }
        // Os dois lados anunciaram o que fizeram.
        for side in [Side::Player, Side::Opponent] {
            assert!(
                seen.iter()
                    .any(|c| matches!(c, Caption::Acted { side: s, .. } if *s == side)),
                "{side:?} não foi narrado: {seen:?}"
            );
        }
        // Entre rodadas não há legenda de animação: a interface mostra o convite à ação.
        assert_eq!(a.caption(), None);
        assert!(!a.resolving());
    }

    #[test]
    fn poses_follow_the_round() {
        // Golpe recebido sem guarda: reação de dano e volta a ficar parado.
        let (c, mut f) = fight();
        let mut a = Animator::new(&f, false);
        let mut ev = Vec::new();
        while f.outcome.is_none() {
            ev.clear();
            let p = game::ai::choose_action(&c, &mut f, Side::Player);
            let o = game::ai::choose_action(&c, &mut f, Side::Opponent);
            f.play_round(&c, p, o, &mut ev);
            a.push_round(&ev, &f);
            let mut seen = Vec::new();
            let mut guard = 0;
            while a.resolving() {
                a.tick(33);
                for side in [Side::Player, Side::Opponent] {
                    seen.push(view(&a, side).frame);
                }
                guard += 1;
                assert!(guard < 1000, "animação não termina");
            }
            let hit = ev.iter().any(|e| matches!(e, FightEvent::Hit { .. }));
            if hit {
                assert!(
                    seen.iter().any(|f| matches!(f, Frame::Hurt | Frame::Block)),
                    "golpe sem reação: {seen:?}"
                );
            }
            for side in [Side::Player, Side::Opponent] {
                let shown = view(&a, side).frame;
                assert!(
                    matches!(shown, Frame::Idle | Frame::Defend | Frame::Down),
                    "pose que não termina: {shown:?}"
                );
            }
        }
    }

    #[test]
    fn reduce_motion_finishes_instantly() {
        let (c, mut f) = fight();
        let mut a = Animator::new(&f, true);
        let mut ev = Vec::new();
        f.play_round(&c, Action::Advance, Action::Advance, &mut ev);
        a.push_round(&ev, &f);
        assert!(!a.busy());
        assert_eq!(
            view(&a, Side::Player).pos_pm,
            i32::from(f.fighter(Side::Player).pos) * 1000
        );
    }
}
