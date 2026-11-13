//! Serialização e leitura defensiva do save.
//!
//! Este módulo não acessa disco: trabalha com bytes. A gravação atômica em arquivo fica no
//! aplicativo. Ver `docs/04-fase-2-jogo.md`, seção 2.2.

use crate::combat::{FightState, Side};
use crate::content::{Content, ItemKind};
use crate::fighter::Fighter;
use crate::state::{GameState, SCHEMA_VERSION, Screen};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Tamanho máximo aceito para um save (bytes).
pub const MAX_SAVE_BYTES: usize = 1024 * 1024;

/// Identificador do formato.
pub const FORMAT_ID: &str = "taskbar-gladiator-save";

/// Envelope gravado em disco.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveFile {
    /// Identificador do formato.
    pub format: String,
    /// Versão do esquema.
    pub version: u32,
    /// Versão do balanceamento em vigor quando o save foi gravado.
    pub balance_version: String,
    /// Estado do jogo.
    pub game: GameState,
}

/// Erro de leitura do save.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SaveError {
    /// Arquivo maior que [`MAX_SAVE_BYTES`].
    TooLarge(usize),
    /// JSON inválido ou estrutura incompatível.
    Parse(String),
    /// Formato não reconhecido.
    WrongFormat,
    /// Versão de esquema mais nova que a suportada.
    NewerVersion(u32),
}

impl fmt::Display for SaveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooLarge(n) => write!(f, "save muito grande ({n} bytes)"),
            Self::Parse(m) => write!(f, "save inválido: {m}"),
            Self::WrongFormat => write!(f, "arquivo não é um save deste jogo"),
            Self::NewerVersion(v) => write!(f, "save de versão mais nova ({v})"),
        }
    }
}

impl std::error::Error for SaveError {}

/// Resultado de uma leitura bem-sucedida.
#[derive(Debug, Clone)]
pub struct Loaded {
    /// Estado validado.
    pub state: GameState,
    /// Correções aplicadas (para o log).
    pub corrections: Vec<String>,
}

/// Serializa o estado em JSON.
pub fn to_bytes(content: &Content, state: &GameState) -> Result<Vec<u8>, SaveError> {
    let file = SaveFile {
        format: FORMAT_ID.to_owned(),
        version: SCHEMA_VERSION,
        balance_version: content.balance_version.clone(),
        game: state.clone(),
    };
    serde_json::to_vec_pretty(&file).map_err(|e| SaveError::Parse(e.to_string()))
}

/// Lê, migra e valida um save.
pub fn from_bytes(content: &Content, bytes: &[u8]) -> Result<Loaded, SaveError> {
    if bytes.len() > MAX_SAVE_BYTES {
        return Err(SaveError::TooLarge(bytes.len()));
    }
    let value: serde_json::Value =
        serde_json::from_slice(bytes).map_err(|e| SaveError::Parse(e.to_string()))?;
    if value.get("format").and_then(|v| v.as_str()) != Some(FORMAT_ID) {
        return Err(SaveError::WrongFormat);
    }
    let version = value
        .get("version")
        .and_then(serde_json::Value::as_u64)
        .and_then(|v| u32::try_from(v).ok())
        .ok_or_else(|| SaveError::Parse("campo version ausente".into()))?;
    if version > SCHEMA_VERSION {
        return Err(SaveError::NewerVersion(version));
    }
    let value = migrate(value, version)?;
    let file: SaveFile =
        serde_json::from_value(value).map_err(|e| SaveError::Parse(e.to_string()))?;
    let mut state = file.game;
    let mut corrections = Vec::new();
    sanitize(content, &mut state, &mut corrections);
    if file.balance_version != content.balance_version
        && let Screen::Fight(fight) = &state.screen
    {
        // Continuar no meio com regras novas produziria um estado que nenhuma das duas versões
        // geraria: a luta recomeça do turno 0 com a mesma semente.
        if let Some(restarted) = restart_fight(content, &state, fight) {
            state.screen = Screen::Fight(Box::new(restarted));
            corrections.push("regras mudaram: luta em andamento reiniciada".into());
        } else {
            state.screen = Screen::Hub;
            corrections.push("regras mudaram: luta em andamento cancelada".into());
        }
    }
    Ok(Loaded { state, corrections })
}

/// Aplica migrações de esquema em sequência até [`SCHEMA_VERSION`].
fn migrate(value: serde_json::Value, from: u32) -> Result<serde_json::Value, SaveError> {
    // Versão 1 é a primeira; migrações futuras entram aqui (v1 → v2 → ...).
    match from {
        1 => Ok(value),
        v => Err(SaveError::Parse(format!("versão {v} desconhecida"))),
    }
}

fn restart_fight(content: &Content, state: &GameState, fight: &FightState) -> Option<FightState> {
    let r = fight.opponent;
    let def = content.opponent(usize::from(r.league), usize::from(r.index))?;
    let opponent = Fighter::from_opponent(content, def);
    Some(FightState::new(
        content,
        &state.player,
        fight.fighter(Side::Player).archetype,
        &opponent,
        def.archetype,
        r,
        fight.rematch,
        fight.seed,
    ))
}

/// Corrige valores fora dos limites. Cada correção é registrada.
pub fn sanitize(content: &Content, state: &mut GameState, out: &mut Vec<String>) {
    let r = &content.rules;
    state.version = SCHEMA_VERSION;

    let max_level = r.progression.max_level;
    if state.player.level == 0 || state.player.level > max_level {
        out.push(format!("nível {} ajustado", state.player.level));
        state.player.level = state.player.level.clamp(1, max_level);
    }
    for v in &mut state.player.attrs.0 {
        if *v < r.attributes.min || *v > r.attributes.max {
            out.push(format!("atributo {v} ajustado"));
            *v = (*v).clamp(r.attributes.min, r.attributes.max);
        }
    }
    let max_points = Fighter::total_points_for_level(content, state.player.level);
    let max_points = u16::try_from(max_points).unwrap_or(u16::MAX);
    if state.unspent_points > max_points {
        out.push("pontos livres ajustados".into());
        state.unspent_points = max_points;
    }
    state.player.name = state.player.name.chars().take(24).collect();

    // Itens desconhecidos viram ouro (preço de venda não disponível → 0) e são removidos.
    let before = state.inventory.len();
    state.inventory.retain(|&id| content.item(id).is_some());
    if state.inventory.len() != before {
        out.push(format!(
            "{} item(ns) desconhecido(s) removido(s)",
            before - state.inventory.len()
        ));
    }
    let max_inv = usize::from(r.progression.inventory_max);
    if state.inventory.len() > max_inv {
        out.push("inventário truncado".into());
        state.inventory.truncate(max_inv);
    }
    for kind in [ItemKind::Weapon, ItemKind::Armor, ItemKind::Shield] {
        let slot = state.player.equipment.slot(kind);
        if let Some(id) = slot {
            let ok =
                state.inventory.contains(&id) && content.item(id).is_some_and(|d| d.kind == kind);
            if !ok {
                out.push(format!("item equipado {id} inválido removido"));
                *state.player.equipment.slot_mut(kind) = None;
            }
        }
    }

    // Progresso das ligas: um valor por liga, cada um até o número de oponentes.
    state.league_progress.resize(content.leagues.len(), 0);
    for (p, def) in state.league_progress.iter_mut().zip(&content.leagues) {
        let len = u16::try_from(def.opponents.len()).unwrap_or(u16::MAX);
        if *p > len {
            out.push("progresso de liga ajustado".into());
            *p = len;
        }
    }

    // Luta em andamento com dados incoerentes: recomeça.
    if let Screen::Fight(fight) = &state.screen {
        let cells = r.arena.cells;
        let coherent = fight
            .fighters
            .iter()
            .all(|c| c.pos < cells && c.hp >= 0 && c.hp <= c.stats.hp_max && c.stamina >= 0)
            && fight.fighters[0].pos != fight.fighters[1].pos
            && fight.turn <= r.fight_end.max_turns
            && content
                .opponent(
                    usize::from(fight.opponent.league),
                    usize::from(fight.opponent.index),
                )
                .is_some();
        if !coherent {
            out.push("luta em andamento incoerente: reiniciada".into());
            state.screen = match restart_fight(content, state, fight) {
                Some(f) => Screen::Fight(Box::new(f)),
                None => Screen::Hub,
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::{Input, Mode, step};

    fn content() -> Content {
        match Content::embedded() {
            Ok(c) => c,
            Err(e) => panic!("{e}"),
        }
    }

    #[test]
    fn round_trip() {
        let c = content();
        let s = GameState::new(&c, "João", 5);
        let bytes = to_bytes(&c, &s).unwrap_or_default();
        let loaded = from_bytes(&c, &bytes).map(|l| l.state);
        assert_eq!(loaded, Ok(s));
    }

    #[test]
    fn round_trip_mid_fight() {
        let c = content();
        let mut s = GameState::new(&c, "T", 5);
        step(
            &c,
            &mut s,
            Input::SetMode(Mode::Auto(crate::content::Archetype::Balanced)),
        );
        step(
            &c,
            &mut s,
            Input::StartFight(crate::combat::OpponentRef {
                league: 0,
                index: 0,
            }),
        );
        step(&c, &mut s, Input::AutoRound);
        let bytes = to_bytes(&c, &s).unwrap_or_default();
        let loaded = from_bytes(&c, &bytes).map(|l| l.state);
        assert_eq!(loaded, Ok(s));
    }

    #[test]
    fn rejects_garbage_and_large_files() {
        let c = content();
        assert!(matches!(from_bytes(&c, b""), Err(SaveError::Parse(_))));
        assert!(matches!(
            from_bytes(&c, b"{\"a\":1}"),
            Err(SaveError::WrongFormat)
        ));
        let big = vec![b' '; MAX_SAVE_BYTES + 1];
        assert!(matches!(from_bytes(&c, &big), Err(SaveError::TooLarge(_))));
    }

    #[test]
    fn rejects_newer_version() {
        let c = content();
        let s = GameState::new(&c, "T", 5);
        let bytes = to_bytes(&c, &s).unwrap_or_default();
        let text = String::from_utf8(bytes).unwrap_or_default();
        let text = text.replacen("\"version\": 1", "\"version\": 99", 1);
        assert_eq!(
            from_bytes(&c, text.as_bytes()).map(|_| ()),
            Err(SaveError::NewerVersion(99))
        );
    }

    #[test]
    fn sanitizes_out_of_range_values() {
        let c = content();
        let mut s = GameState::new(&c, "T", 5);
        s.player.level = 250;
        s.player.attrs.0[0] = 0;
        s.inventory.push(9999);
        s.player.equipment.armor = Some(9999);
        s.league_progress = vec![500, 7];
        let mut out = Vec::new();
        sanitize(&c, &mut s, &mut out);
        assert_eq!(s.player.level, c.rules.progression.max_level);
        assert_eq!(s.player.attrs.0[0], c.rules.attributes.min);
        assert!(!s.inventory.contains(&9999));
        assert_eq!(s.player.equipment.armor, None);
        assert_eq!(s.league_progress.len(), c.leagues.len());
        assert!(!out.is_empty());
    }
}
