//! Rasterização por software em buffer BGRA com alpha premultiplicado.
//!
//! Este crate não depende do Windows nem do crate `game`: recebe visões prontas para exibir.
//! Ver `docs/01-arquitetura.md`, seção 3.
#![forbid(unsafe_code)]

pub mod draw;
pub mod png;
pub mod scene;
pub mod surface;

pub use scene::{
    ButtonState, FighterView, Hit, HitId, PanelRow, PanelView, StripView, Team, Theme, draw_panel,
    draw_strip, hit_test, strip_scale,
};
pub use surface::{Rgba, Surface};

/// Versão do crate.
pub const CRATE_VERSION: &str = env!("CARGO_PKG_VERSION");
