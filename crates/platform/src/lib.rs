//! Integração com o Windows: janela, barra de tarefas, bandeja, DPI, sessão e energia.
//!
//! Único crate do workspace autorizado a usar `unsafe`. Todo bloco `unsafe` precisa de um
//! comentário `// SAFETY:`. Ver `docs/03-fase-1-plataforma.md` e `docs/07-seguranca.md`.

/// Versão do crate.
pub const CRATE_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Indica se a plataforma atual é suportada pelo jogo.
pub const fn is_supported_platform() -> bool {
    cfg!(windows)
}

#[cfg(test)]
mod tests {
    #[test]
    fn crate_version_is_set() {
        assert!(!super::CRATE_VERSION.is_empty());
    }
}
