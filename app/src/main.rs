//! Executável do jogo.
#![forbid(unsafe_code)]

mod identity;

fn main() {
    if !platform::is_supported_platform() {
        eprintln!(
            "{} {} roda apenas no Windows.",
            identity::DISPLAY_NAME,
            env!("CARGO_PKG_VERSION")
        );
        std::process::exit(1);
    }
}
