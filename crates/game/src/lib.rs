//! Lógica pura do jogo: conteúdo, combate, IA, progressão, estado e save.
//!
//! Este crate não depende do Windows nem acessa disco ou relógio. Ver `docs/04-fase-2-jogo.md`
//! e `docs/11-design-do-jogo.md`.
#![forbid(unsafe_code)]

pub mod ai;
pub mod combat;
pub mod content;
pub mod fighter;
pub mod progression;
pub mod rng;
pub mod save;
pub mod state;

pub use combat::{Action, FightEvent, FightState, OpponentRef, Outcome, Side};
pub use content::{Archetype, Content, ContentError, ItemId, ItemKind};
pub use fighter::{Attr, Attributes, Equipment, Fighter};
pub use state::{GameEvent, GameState, Input, Mode, Rejection, SaveHint, Screen, StepResult, step};

/// Versão do crate, usada em logs.
pub const CRATE_VERSION: &str = env!("CARGO_PKG_VERSION");
