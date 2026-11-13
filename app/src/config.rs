//! Configurações do usuário (`config.json`), separadas do save do jogo.

use crate::i18n::Lang;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Tamanho máximo do arquivo de configuração.
pub const MAX_CONFIG_BYTES: usize = 64 * 1024;

/// Ritmo do modo automático.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Pace {
    /// 2,5 s por rodada.
    Slow,
    /// 1,5 s por rodada.
    #[default]
    Normal,
    /// 0,8 s por rodada.
    Fast,
}

impl Pace {
    /// Milissegundos entre rodadas.
    pub fn round_ms(self) -> u32 {
        match self {
            Pace::Slow => 2_500,
            Pace::Normal => 1_500,
            Pace::Fast => 800,
        }
    }

    /// Pausa entre lutas no modo automático (ms).
    pub fn rest_ms(self) -> u32 {
        match self {
            Pace::Slow => 20_000,
            Pace::Normal => 10_000,
            Pace::Fast => 4_000,
        }
    }

    /// Próximo ritmo (ciclo).
    pub fn next(self) -> Pace {
        match self {
            Pace::Slow => Pace::Normal,
            Pace::Normal => Pace::Fast,
            Pace::Fast => Pace::Slow,
        }
    }
}

/// Configurações.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Versão do formato.
    pub version: u32,
    /// Idioma escolhido (`None` = do sistema).
    pub language: Option<Lang>,
    /// Deslocamento horizontal da faixa por monitor (px lógicos), chave `dispositivo@LxA`.
    pub offsets: BTreeMap<String, i32>,
    /// Reduzir animações.
    pub reduce_motion: bool,
    /// Iniciar com o Windows (espelha a chave Run; a fonte da verdade é o registro).
    pub start_with_windows: bool,
    /// Ritmo do modo automático.
    pub pace: Pace,
    /// Distribuir pontos automaticamente no modo automático.
    pub auto_allocate: bool,
    /// Faixa oculta pelo usuário.
    pub hidden: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            version: 1,
            language: None,
            offsets: BTreeMap::new(),
            reduce_motion: false,
            start_with_windows: false,
            pace: Pace::Normal,
            auto_allocate: true,
            hidden: false,
        }
    }
}

impl Config {
    /// Lê a configuração; valores fora dos limites são corrigidos. JSON inválido → `None`.
    pub fn from_bytes(bytes: &[u8]) -> Option<Config> {
        let mut c: Config = serde_json::from_slice(bytes).ok()?;
        c.version = 1;
        // Limites: chaves curtas, no máximo 32 monitores, deslocamentos razoáveis.
        c.offsets
            .retain(|k, v| k.len() <= 128 && (0..=20_000).contains(v));
        while c.offsets.len() > 32 {
            let Some(k) = c.offsets.keys().next().cloned() else {
                break;
            };
            c.offsets.remove(&k);
        }
        Some(c)
    }

    /// Serializa.
    pub fn to_bytes(&self) -> Vec<u8> {
        serde_json::to_vec_pretty(self).unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_and_defaults() {
        let c = Config::default();
        assert_eq!(Config::from_bytes(&c.to_bytes()), Some(c));
        let partial = Config::from_bytes(br#"{"reduce_motion": true}"#);
        assert_eq!(
            partial.map(|c| (c.reduce_motion, c.pace)),
            Some((true, Pace::Normal))
        );
    }

    #[test]
    fn invalid_values_are_dropped() {
        let c = Config::from_bytes(br#"{"offsets": {"a": -5, "b": 100}}"#);
        assert_eq!(c.map(|c| c.offsets.len()), Some(1));
        assert!(Config::from_bytes(b"lixo").is_none());
    }
}
