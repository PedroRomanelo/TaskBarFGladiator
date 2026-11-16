//! Integração com o Windows: janela, barra de tarefas, bandeja, DPI, sessão e energia.
//!
//! Único crate do workspace autorizado a usar `unsafe`, e só no módulo `win`. Todo bloco `unsafe`
//! tem um comentário `// SAFETY:`. Os módulos `layout` e `event` são portáveis e testados em
//! qualquer sistema. Ver `docs/03-fase-1-plataforma.md` e `docs/07-seguranca.md`.
// Chamadas FFI frequentemente exigem vários passos inseguros encadeados; cada bloco continua
// obrigado a ter `// SAFETY:` (lint `undocumented_unsafe_blocks` em nível deny).
#![allow(clippy::multiple_unsafe_ops_per_block)]

pub mod event;
pub mod layout;

#[cfg(windows)]
mod win;
#[cfg(windows)]
pub use win::*;

pub use event::{Event, MouseButton, WindowId};
pub use layout::{Edge, Placement, Rect, TaskbarInfo, place_stage};

/// Versão do crate.
pub const CRATE_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Indica se a plataforma atual é suportada pelo jogo.
pub const fn is_supported_platform() -> bool {
    cfg!(windows)
}
