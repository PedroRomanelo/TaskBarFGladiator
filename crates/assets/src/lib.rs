//! Sprites e fonte embutidos no executável.
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
