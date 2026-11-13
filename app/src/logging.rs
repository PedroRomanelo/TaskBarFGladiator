//! Log em arquivo com rotação e registro de panics (portável).
//!
//! Arquivo `<pasta>/game.log`, rotacionado ao atingir [`MAX_LOG_BYTES`] (mantém 3 arquivos).
//! Caminhos com o nome do usuário são registrados com o prefixo do perfil substituído por
//! `%USERPROFILE%` (privacidade).

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

/// Tamanho máximo de cada arquivo de log.
pub const MAX_LOG_BYTES: u64 = 1024 * 1024;
const KEEP: usize = 3;

static LOG: OnceLock<Mutex<Logger>> = OnceLock::new();

/// Nível.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Level {
    /// Erro.
    Error,
    /// Aviso.
    Warn,
    /// Informação.
    Info,
}

#[derive(Debug)]
struct Logger {
    path: PathBuf,
    profile: Option<String>,
}

/// Inicializa o log na pasta indicada. `profile` é o caminho do perfil do usuário, que será
/// substituído por `%USERPROFILE%` nas mensagens.
pub fn init(dir: &Path, profile: Option<String>) {
    let _ = fs::create_dir_all(dir);
    let _ = LOG.set(Mutex::new(Logger {
        path: dir.join("game.log"),
        profile,
    }));
}

/// Remove dados pessoais conhecidos de uma mensagem.
pub fn scrub(msg: &str, profile: Option<&str>) -> String {
    match profile {
        Some(p) if !p.is_empty() => msg.replace(p, "%USERPROFILE%"),
        _ => msg.to_owned(),
    }
}

fn timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

fn rotate(path: &Path) {
    if fs::metadata(path).map_or(0, |m| m.len()) < MAX_LOG_BYTES {
        return;
    }
    for i in (1..KEEP).rev() {
        let from = path.with_extension(format!("{i}.log"));
        let to = path.with_extension(format!("{}.log", i + 1));
        let _ = fs::rename(from, to);
    }
    let _ = fs::rename(path, path.with_extension("1.log"));
}

/// Registra uma mensagem (sem efeito se o log não foi inicializado).
pub fn log(level: Level, msg: &str) {
    let Some(lock) = LOG.get() else { return };
    let Ok(logger) = lock.lock() else { return };
    rotate(&logger.path);
    let line = scrub(msg, logger.profile.as_deref());
    if let Ok(mut f) = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&logger.path)
    {
        let _ = writeln!(f, "{} {:?} {}", timestamp(), level, line);
    }
}

/// Atalho para `log(Level::Info, ...)`.
pub fn info(msg: &str) {
    log(Level::Info, msg);
}

/// Atalho para `log(Level::Warn, ...)`.
pub fn warn(msg: &str) {
    log(Level::Warn, msg);
}

/// Atalho para `log(Level::Error, ...)`.
pub fn error(msg: &str) {
    log(Level::Error, msg);
}

/// Instala o hook de panic: grava `crash-<timestamp>.txt` na pasta de logs antes de abortar.
pub fn install_panic_hook(dir: PathBuf, version: &'static str) {
    std::panic::set_hook(Box::new(move |info| {
        let msg = format!(
            "versão {version}\n{info}\n{}",
            std::backtrace::Backtrace::force_capture()
        );
        let profile = LOG
            .get()
            .and_then(|l| l.lock().ok().and_then(|g| g.profile.clone()));
        let msg = scrub(&msg, profile.as_deref());
        let _ = fs::create_dir_all(&dir);
        let _ = fs::write(
            dir.join(format!("crash-{}.txt", timestamp())),
            msg.as_bytes(),
        );
        error(&format!("panic: {info}"));
    }));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scrub_removes_profile() {
        let s = scrub("C:\\Users\\João\\AppData\\x", Some("C:\\Users\\João"));
        assert_eq!(s, "%USERPROFILE%\\AppData\\x");
        assert_eq!(scrub("abc", None), "abc");
    }
}
