//! Rasterização por software em buffer BGRA com alpha premultiplicado.
//!
//! Este crate não depende do Windows. Ver `docs/01-arquitetura.md`, seção 3.
#![forbid(unsafe_code)]

/// Versão do crate.
pub const CRATE_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    #[test]
    fn crate_version_is_set() {
        assert!(!super::CRATE_VERSION.is_empty());
    }
}
