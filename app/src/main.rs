//! Executável do jogo.
#![forbid(unsafe_code)]
#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]
// Fora do Windows, os módulos usados pela integração com o sistema só são exercitados pelos testes.
#![cfg_attr(not(windows), allow(dead_code))]

mod anim;
mod config;
mod controller;
mod i18n;
mod identity;
mod logging;
mod looks;
mod storage;
#[cfg(windows)]
mod win_main;

fn main() {
    #[cfg(windows)]
    win_main::run();
    #[cfg(not(windows))]
    {
        eprintln!(
            "{} {} roda apenas no Windows.",
            identity::DISPLAY_NAME,
            env!("CARGO_PKG_VERSION")
        );
        std::process::exit(1);
    }
}
