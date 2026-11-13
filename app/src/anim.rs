//! Animação da luta: converte eventos de combate em passos com duração (portável).
//!
//! O estado do jogo muda de uma vez por rodada; a animação mostra a rodada em passos (movimento,
//! golpe, dano) e, ao final, sincroniza os valores exibidos com o estado real.

use assets::Frame;
use game::combat::FightEvent;
use game::{Action, FightState, Side};
use render::FighterView;
use std::collections::VecDeque;

/// Duração dos passos (ms).
const MOVE_MS: u32 = 220;
const ATTACK_MS: u32 = 240;
const HIT_MS: u32 = 160;
const BANNER_MS: u32 = 700;

/// Texto do banner sobre a arena.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Banner {
    /// Golpe crítico.
    Critical,
    /// Erro.
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
    flash_ms: u32,
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
    Hit {
        target: Side,
        damage: i32,
        critical: bool,
    },
    Miss,
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
    names: [String; 2],
    levels: [u8; 2],
    queue: VecDeque<Step>,
    current: Option<(Step, u32, i32)>,
    banner: Option<(Banner, u32)>,
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
        flash_ms: 0,
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
            names: [
                fight.fighter(Side::Player).name.clone(),
                fight.fighter(Side::Opponent).name.clone(),
            ],
            levels: [
                fight.fighter(Side::Player).level,
                fight.fighter(Side::Opponent).level,
            ],
            queue: VecDeque::with_capacity(16),
            current: None,
            banner: None,
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
                    match action {
                        Action::Light | Action::Medium | Action::Heavy => {
                            self.push(
                                StepKind::Pose {
                                    side,
                                    frame: Frame::Attack,
                                },
                                ATTACK_MS,
                            );
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
                        }
                        Action::Advance | Action::Retreat | Action::Recover | Action::Taunt => {}
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
                        HIT_MS,
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
                    self.push(StepKind::Miss, HIT_MS);
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
                FightEvent::Ended(outcome) => {
                    self.push(
                        StepKind::Pose {
                            side: outcome.winner.other(),
                            frame: Frame::Down,
                        },
                        0,
                    );
                }
                FightEvent::Initiative(_)
                | FightEvent::Converted { .. }
                | FightEvent::Recovered { .. } => {}
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
        }
        self.banner = None;
    }

    /// Verdadeiro enquanto há passos pendentes (o temporizador de animação deve continuar ligado).
    pub fn busy(&self) -> bool {
        self.current.is_some()
            || !self.queue.is_empty()
            || self.banner.is_some()
            || self.vis.iter().any(|v| v.flash_ms > 0)
    }

    /// Avança `dt_ms`. Retorna `true` se algo visível mudou.
    pub fn tick(&mut self, dt_ms: u32) -> bool {
        let dt = dt_ms.min(100); // após suspensão o dt é limitado
        let mut changed = false;
        for v in &mut self.vis {
            if v.flash_ms > 0 {
                v.flash_ms = v.flash_ms.saturating_sub(dt);
                changed = true;
            }
        }
        if let Some((_, rem)) = &mut self.banner {
            *rem = rem.saturating_sub(dt);
            if *rem == 0 {
                self.banner = None;
            }
            changed = true;
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

    fn apply_start(&mut self, step: Step) {
        match step.kind {
            StepKind::Pose { side, frame } => self.vis[side.index()].frame = frame,
            StepKind::Hit {
                target,
                damage,
                critical,
            } => {
                let v = &mut self.vis[target.index()];
                v.hp = (v.hp - damage).max(0);
                v.flash_ms = if self.reduce_motion { 0 } else { HIT_MS };
                if critical && !self.reduce_motion {
                    self.banner = Some((Banner::Critical, BANNER_MS));
                }
            }
            StepKind::Miss => {
                if !self.reduce_motion {
                    self.banner = Some((Banner::Miss, BANNER_MS / 2));
                }
            }
            StepKind::Move { .. } | StepKind::Sync => {}
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
            StepKind::Move { side, to_pm } => self.vis[side.index()].pos_pm = to_pm,
            StepKind::Sync => {
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
                            v.frame = Frame::Down;
                        }
                    }
                }
            }
            StepKind::Pose { .. } | StepKind::Hit { .. } | StepKind::Miss => {}
        }
    }

    /// Banner atual.
    pub fn banner(&self) -> Option<Banner> {
        self.banner.map(|(b, _)| b)
    }

    /// Atualiza uma visão existente sem alocar (o nome só é copiado se mudou).
    pub fn fill_view(&self, side: Side, out: &mut FighterView) {
        let v = &self.vis[side.index()];
        if out.name != self.names[side.index()] {
            out.name.clone_from(&self.names[side.index()]);
        }
        out.level = self.levels[side.index()];
        out.hp = v.hp;
        out.hp_max = v.hp_max;
        out.stamina = v.stamina;
        out.stamina_max = v.stamina_max;
        out.pos_pm = v.pos_pm;
        out.frame = v.frame;
        out.flash = v.flash_ms > 0;
    }

    /// Visão de um lutador para a renderização.
    pub fn view(&self, side: Side) -> FighterView {
        let v = &self.vis[side.index()];
        FighterView {
            name: self.names[side.index()].clone(),
            level: self.levels[side.index()],
            hp: v.hp,
            hp_max: v.hp_max,
            stamina: v.stamina,
            stamina_max: v.stamina_max,
            pos_pm: v.pos_pm,
            frame: v.frame,
            flash: v.flash_ms > 0,
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
                let v = a.view(side);
                let real = f.fighter(side);
                assert_eq!(v.hp, real.hp);
                assert_eq!(v.pos_pm, i32::from(real.pos) * 1000);
            }
        }
        let loser = f.outcome.map(|o| o.winner.other());
        if let Some(l) = loser
            && f.fighter(l).hp == 0
        {
            assert_eq!(a.view(l).frame, Frame::Down);
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
            a.view(Side::Player).pos_pm,
            i32::from(f.fighter(Side::Player).pos) * 1000
        );
    }
}
