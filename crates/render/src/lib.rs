//! Rasterização por software em buffer BGRA com alpha premultiplicado.
//!
//! Este crate não depende do Windows nem do crate `game`: recebe visões prontas para exibir.
//! Ver `docs/01-arquitetura.md`, seção 3.
#![forbid(unsafe_code)]

pub mod draw;
pub mod png;
pub mod scene;
pub mod surface;

pub use assets::Look;
pub use scene::{
    ButtonState, FighterView, FloatText, Floater, Hit, HitId, PanelRow, PanelView, Phase,
    StageView, Team, Theme, Tone, draw_panel, draw_stage, hit_test, stage_scale,
};
pub use surface::{Rgba, Surface};

/// Versão do crate.
pub const CRATE_VERSION: &str = env!("CARGO_PKG_VERSION");
