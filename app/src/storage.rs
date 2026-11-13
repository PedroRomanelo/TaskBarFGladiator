//! Gravação atômica com backup e leitura com recuperação (portável).
//!
//! Sequência de gravação: escreve `arquivo.tmp` → `sync_all` → renomeia o arquivo atual para
//! `arquivo.bak` → renomeia `arquivo.tmp` para o nome final. Em qualquer ponto de interrupção resta
//! um arquivo válido (`arquivo` ou `arquivo.bak`). Ver `docs/04-fase-2-jogo.md`, seção 2.2.

use std::fs;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::JoinHandle;

fn with_suffix(path: &Path, suffix: &str) -> PathBuf {
    let mut s = path.as_os_str().to_owned();
    s.push(suffix);
    PathBuf::from(s)
}

/// Caminho do backup.
pub fn backup_path(path: &Path) -> PathBuf {
    with_suffix(path, ".bak")
}

/// Grava de forma atômica, mantendo o arquivo anterior como `.bak`.
pub fn write_atomic(path: &Path, bytes: &[u8]) -> io::Result<()> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }
    let tmp = with_suffix(path, ".tmp");
    {
        let mut f = fs::File::create(&tmp)?;
        f.write_all(bytes)?;
        f.sync_all()?;
    }
    if path.exists() {
        let bak = backup_path(path);
        // Falha ao criar o backup não impede a gravação (o .tmp já está seguro em disco).
        let _ = fs::rename(path, &bak);
    }
    fs::rename(&tmp, path)
}

/// Lê no máximo `limit` bytes. Arquivos maiores são recusados sem serem lidos por inteiro.
pub fn read_limited(path: &Path, limit: usize) -> io::Result<Vec<u8>> {
    let meta = fs::metadata(path)?;
    if meta.len() > limit as u64 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "arquivo grande demais",
        ));
    }
    let mut buf = Vec::with_capacity(meta.len() as usize);
    fs::File::open(path)?
        .take(limit as u64 + 1)
        .read_to_end(&mut buf)?;
    if buf.len() > limit {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "arquivo grande demais",
        ));
    }
    Ok(buf)
}

/// Resultado de uma leitura com recuperação.
#[derive(Debug)]
pub enum Recovered<T> {
    /// Arquivo principal válido.
    Primary(T),
    /// Principal inválido; backup válido.
    Backup(T),
    /// Nenhum arquivo existe.
    Missing,
    /// Os dois existem e são inválidos (foram renomeados para `.corrupt-<n>`).
    Corrupt,
}

/// Lê `path` com `parse`; se falhar, tenta o `.bak`. Arquivos inválidos são preservados com o
/// sufixo `.corrupt-<n>` para análise.
pub fn load_with_recovery<T>(
    path: &Path,
    limit: usize,
    mut parse: impl FnMut(&[u8]) -> Option<T>,
) -> Recovered<T> {
    let bak = backup_path(path);
    let primary_exists = path.exists();
    let backup_exists = bak.exists();
    if !primary_exists && !backup_exists {
        return Recovered::Missing;
    }
    if primary_exists && let Some(v) = read_limited(path, limit).ok().and_then(|b| parse(&b)) {
        return Recovered::Primary(v);
    }
    if backup_exists && let Some(v) = read_limited(&bak, limit).ok().and_then(|b| parse(&b)) {
        if primary_exists {
            preserve_corrupt(path);
        }
        return Recovered::Backup(v);
    }
    if primary_exists {
        preserve_corrupt(path);
    }
    if backup_exists {
        preserve_corrupt(&bak);
    }
    Recovered::Corrupt
}

fn preserve_corrupt(path: &Path) {
    for n in 1..1000 {
        let target = with_suffix(path, &format!(".corrupt-{n}"));
        if !target.exists() {
            let _ = fs::rename(path, target);
            return;
        }
    }
}

/// Pedido de gravação.
#[derive(Debug)]
struct Job {
    path: PathBuf,
    bytes: Vec<u8>,
}

/// Thread de gravação: antivírus, sincronização de nuvem ou perfil em rede podem bloquear o disco
/// por segundos, e a interface não pode congelar.
#[derive(Debug)]
pub struct SaveWorker {
    tx: Option<Sender<Job>>,
    handle: Option<JoinHandle<()>>,
    errors: Receiver<String>,
}

impl SaveWorker {
    /// Inicia a thread (pilha pequena: o trabalho é só I/O).
    pub fn start() -> io::Result<Self> {
        let (tx, rx) = mpsc::channel::<Job>();
        let (etx, erx) = mpsc::channel::<String>();
        let handle = std::thread::Builder::new()
            .name("save".into())
            .stack_size(64 * 1024)
            .spawn(move || {
                while let Ok(first) = rx.recv() {
                    // Se vários pedidos se acumularam, só o mais recente de cada arquivo importa.
                    let mut pending: Vec<Job> = vec![first];
                    while let Ok(newer) = rx.try_recv() {
                        match pending.iter_mut().find(|j| j.path == newer.path) {
                            Some(slot) => *slot = newer,
                            None => pending.push(newer),
                        }
                    }
                    for job in pending {
                        if let Err(e) = write_atomic(&job.path, &job.bytes) {
                            let _ = etx.send(format!("{}: {e}", job.path.display()));
                        }
                    }
                }
            })?;
        Ok(Self {
            tx: Some(tx),
            handle: Some(handle),
            errors: erx,
        })
    }

    /// Enfileira uma gravação.
    pub fn save(&self, path: PathBuf, bytes: Vec<u8>) {
        if let Some(tx) = &self.tx {
            let _ = tx.send(Job { path, bytes });
        }
    }

    /// Erros de gravação ocorridos desde a última consulta.
    pub fn take_errors(&self) -> Vec<String> {
        self.errors.try_iter().collect()
    }

    /// Termina a thread depois de gravar o que estiver pendente.
    pub fn finish(&mut self) {
        self.tx = None;
        if let Some(h) = self.handle.take() {
            let _ = h.join();
        }
    }
}

impl Drop for SaveWorker {
    fn drop(&mut self) {
        self.finish();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("tbg-test-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&d);
        let _ = fs::create_dir_all(&d);
        d
    }

    #[test]
    fn atomic_write_keeps_backup() {
        let d = temp_dir("atomic");
        let p = d.join("save.json");
        write_atomic(&p, b"um").unwrap_or(());
        write_atomic(&p, b"dois").unwrap_or(());
        assert_eq!(fs::read(&p).unwrap_or_default(), b"dois");
        assert_eq!(fs::read(backup_path(&p)).unwrap_or_default(), b"um");
        let _ = fs::remove_dir_all(d);
    }

    #[test]
    fn recovers_from_backup_and_preserves_corrupt() {
        let d = temp_dir("recover");
        let p = d.join("save.json");
        write_atomic(&p, b"ok").unwrap_or(());
        write_atomic(&p, b"ok2").unwrap_or(());
        fs::write(&p, b"lixo").unwrap_or(());
        let parse = |b: &[u8]| b.starts_with(b"ok").then(|| b.to_vec());
        match load_with_recovery(&p, 1024, parse) {
            Recovered::Backup(v) => assert_eq!(v, b"ok"),
            other => panic!("{other:?}"),
        }
        assert!(with_suffix(&p, ".corrupt-1").exists());
        let _ = fs::remove_dir_all(d);
    }

    #[test]
    fn missing_and_too_large() {
        let d = temp_dir("missing");
        let p = d.join("nada.json");
        assert!(matches!(
            load_with_recovery(&p, 10, |_| Some(())),
            Recovered::Missing
        ));
        fs::write(&p, vec![b'x'; 100]).unwrap_or(());
        assert!(read_limited(&p, 10).is_err());
        let _ = fs::remove_dir_all(d);
    }

    #[test]
    fn accented_paths_work() {
        let d = temp_dir("acentos").join("João Ação");
        let p = d.join("save.json");
        write_atomic(&p, b"ok").unwrap_or(());
        assert_eq!(fs::read(&p).unwrap_or_default(), b"ok");
        let _ = fs::remove_dir_all(d.parent().map(Path::to_path_buf).unwrap_or(d));
    }

    #[test]
    fn worker_writes_latest() {
        let d = temp_dir("worker");
        let p = d.join("save.json");
        let mut w = match SaveWorker::start() {
            Ok(w) => w,
            Err(e) => panic!("{e}"),
        };
        for i in 0..20 {
            w.save(p.clone(), format!("v{i}").into_bytes());
        }
        let other = d.join("config.json");
        w.save(other.clone(), b"cfg".to_vec());
        w.save(p.clone(), b"v20".to_vec());
        w.finish();
        assert_eq!(fs::read(&p).unwrap_or_default(), b"v20");
        assert_eq!(fs::read(&other).unwrap_or_default(), b"cfg");
        assert!(w.take_errors().is_empty());
        let _ = fs::remove_dir_all(d);
    }
}
