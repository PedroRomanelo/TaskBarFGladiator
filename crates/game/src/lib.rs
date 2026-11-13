//! Lógica pura do jogo: estado, combate, IA, progressão e save.
//!
//! Este crate não depende do Windows nem de I/O. Ver `docs/04-fase-2-jogo.md`.
#![forbid(unsafe_code)]

/// Versão do crate, usada em logs e no save.
pub const CRATE_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    #[test]
    fn crate_version_is_set() {
        assert!(!super::CRATE_VERSION.is_empty());
    }
}
