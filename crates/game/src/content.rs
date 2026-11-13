//! Conteúdo e regras do jogo, carregados de arquivos JSON.
//!
//! Os arquivos ficam em `crates/game/content/` e são embutidos no executável. Todo número que afeta o
//! balanceamento vive nesses arquivos, nunca no código. Formato documentado em
//! `docs/14-formato-do-conteudo.md`.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fmt;

/// Texto embutido de `rules.json`.
pub const EMBEDDED_RULES: &str = include_str!("../content/rules.json");
/// Texto embutido de `items.json`.
pub const EMBEDDED_ITEMS: &str = include_str!("../content/items.json");
/// Texto embutido de `leagues.json`.
pub const EMBEDDED_LEAGUES: &str = include_str!("../content/leagues.json");

/// Número de atributos primários.
pub const ATTR_COUNT: usize = 6;

/// Identificador estável de item. Nunca reutilizado (aparece no save).
pub type ItemId = u16;

/// Texto traduzido.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Localized {
    /// Português do Brasil.
    pub pt: String,
    /// Inglês.
    pub en: String,
}

/// Regras da arena.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArenaRules {
    /// Número de casas.
    pub cells: u8,
    /// Casa inicial do jogador (lado esquerdo).
    pub player_start: u8,
    /// Casa inicial do oponente (lado direito).
    pub opponent_start: u8,
}

/// Limites e distribuição de atributos.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AttributeRules {
    /// Valor mínimo de um atributo.
    pub min: u8,
    /// Valor máximo de um atributo.
    pub max: u8,
    /// Valor inicial de cada atributo.
    pub start_value: u8,
    /// Pontos livres na criação.
    pub start_free_points: u16,
    /// Pontos ganhos por nível.
    pub points_per_level: u16,
}

/// Fórmulas de atributos derivados.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DerivedRules {
    /// HP base.
    pub hp_base: i32,
    /// HP por ponto de Vitalidade.
    pub hp_per_vitality: i32,
    /// HP por nível.
    pub hp_per_level: i32,
    /// Fôlego base.
    pub stamina_base: i32,
    /// Fôlego por ponto de Fôlego.
    pub stamina_per_point: i32,
    /// Fôlego máximo mínimo (após penalidades).
    pub stamina_min: i32,
    /// Recuperação base por turno.
    pub regen_base: i32,
    /// Divisor do atributo Fôlego na recuperação.
    pub regen_divisor: i32,
    /// Dano base.
    pub damage_base: i32,
    /// Dano por ponto de Força, em décimos (10 = +1 por ponto).
    pub damage_per_strength_tenth: i32,
    /// Chance de crítico base (‰).
    pub crit_base_pm: i32,
    /// Chance de crítico por ponto de Técnica, em décimos de ‰.
    pub crit_per_technique_tenth_pm: i32,
    /// Chance de crítico máxima vinda de atributos (‰).
    pub crit_max_pm: i32,
    /// Multiplicador de dano crítico (‰).
    pub crit_multiplier_pm: i32,
    /// Absorção por ponto de Defesa.
    pub absorb_per_defense: i32,
    /// Lados do dado de iniciativa.
    pub initiative_die: u32,
}

/// Regras de acerto.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HitRules {
    /// Chance mínima (‰).
    pub min_pm: i32,
    /// Chance máxima (‰).
    pub max_pm: i32,
    /// ‰ por ponto de diferença entre Técnica do atacante e Agilidade do defensor.
    pub per_point_pm: i32,
}

/// Regras de dano.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DamageRules {
    /// Variação mínima (‰).
    pub variance_min_pm: u32,
    /// Variação máxima (‰).
    pub variance_max_pm: u32,
    /// Dano mínimo de um acerto.
    pub min_damage: i32,
    /// Escala da redução: `dano × escala / (escala + absorção)`.
    pub absorb_scale: i32,
}

/// Custo de uma ação de movimento.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MoveRules {
    /// Custo de fôlego.
    pub stamina: i32,
}

/// Parâmetros de um ataque.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AttackRules {
    /// Custo de fôlego (antes do peso da arma).
    pub stamina: i32,
    /// Distância mínima.
    pub range_min: u8,
    /// Distância máxima.
    pub range_max: u8,
    /// Precisão base (‰).
    pub accuracy_pm: i32,
    /// Penalidade de precisão por casa além da distância 1 (‰).
    pub far_penalty_pm: i32,
    /// Multiplicador de dano (‰).
    pub damage_pm: i32,
    /// Bônus de crítico (‰).
    pub crit_bonus_pm: i32,
}

/// Parâmetros da postura Defender.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DefendRules {
    /// Multiplicador do dano recebido (‰).
    pub damage_taken_pm: i32,
}

/// Parâmetros de Recuperar.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecoverRules {
    /// Múltiplo da recuperação por turno.
    pub regen_multiplier: i32,
    /// Multiplicador do dano recebido (‰).
    pub damage_taken_pm: i32,
}

/// Parâmetros de Provocar.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TauntRules {
    /// Custo de fôlego.
    pub stamina: i32,
    /// Bônus de crítico no próximo ataque (‰).
    pub next_crit_bonus_pm: i32,
    /// Multiplicador de dano do próximo ataque (‰).
    pub next_damage_pm: i32,
    /// Multiplicador da própria absorção até a próxima ação (‰).
    pub self_absorb_pm: i32,
}

/// Todas as ações.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ActionRules {
    /// Avançar.
    pub advance: MoveRules,
    /// Recuar.
    pub retreat: MoveRules,
    /// Ataque leve.
    pub light: AttackRules,
    /// Ataque médio.
    pub medium: AttackRules,
    /// Ataque pesado.
    pub heavy: AttackRules,
    /// Defender.
    pub defend: DefendRules,
    /// Recuperar.
    pub recover: RecoverRules,
    /// Provocar.
    pub taunt: TauntRules,
}

/// Regras de término da luta.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FightEndRules {
    /// Turno em que a exaustão começa.
    pub exhaustion_turn: u16,
    /// Perda de fôlego máximo por turno durante a exaustão (‰ do valor original).
    pub exhaustion_pm_per_turn: i32,
    /// Último turno.
    pub max_turns: u16,
}

/// Curva de XP: `a + b·n + c·n²`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct XpCurve {
    /// Termo constante.
    pub a: u32,
    /// Termo linear.
    pub b: u32,
    /// Termo quadrático.
    pub c: u32,
}

/// Regras de progressão e economia.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProgressionRules {
    /// Nível máximo.
    pub max_level: u8,
    /// Curva de XP.
    pub xp_curve: XpCurve,
    /// XP base por vitória.
    pub xp_win_base: u32,
    /// XP por nível do oponente.
    pub xp_win_per_level: u32,
    /// Multiplicador de XP quando o oponente tem nível maior (‰).
    pub xp_higher_level_pm: u32,
    /// XP de derrota (‰ da vitória).
    pub xp_loss_pm: u32,
    /// Ouro base por vitória.
    pub gold_win_base: u32,
    /// Ouro por nível do oponente.
    pub gold_win_per_level: u32,
    /// Ouro de derrota (‰ da vitória).
    pub gold_loss_pm: u32,
    /// Recompensa contra oponente já vencido (‰).
    pub rematch_reward_pm: u32,
    /// Preço de venda (‰ do preço de compra).
    pub sell_pm: u32,
    /// Ouro inicial.
    pub start_gold: u32,
    /// Itens máximos no inventário.
    pub inventory_max: u16,
}

/// Pesos de utilidade de um arquétipo de IA.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArchetypeRules {
    /// Distância preferida.
    pub preferred_distance: u8,
    /// Peso do dano esperado.
    pub w_damage: i64,
    /// Peso do risco recebido.
    pub w_risk: i64,
    /// Peso do fôlego restante.
    pub w_stamina: i64,
    /// Peso da distância ideal.
    pub w_distance: i64,
    /// Peso das oportunidades.
    pub w_opportunity: i64,
    /// Fôlego mínimo (‰ do máximo) para considerar o ataque pesado.
    pub heavy_min_stamina_pm: i64,
}

/// Arquétipos de IA.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Archetypes {
    /// Agressivo.
    pub aggressive: ArchetypeRules,
    /// Defensivo.
    pub defensive: ArchetypeRules,
    /// Oportunista.
    pub opportunist: ArchetypeRules,
    /// Equilibrado.
    pub balanced: ArchetypeRules,
}

/// Parâmetros gerais da IA.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AiRules {
    /// Ruído máximo somado à pontuação.
    pub noise_max: u32,
    /// Valor base da provocação.
    pub taunt_value: i64,
    /// Valor de Recuperar por ‰ de fôlego faltante.
    pub recover_value_per_missing_pm: i64,
    /// Peso (‰) do potencial ofensivo na nova distância em relação ao dano imediato.
    pub potential_pm: i64,
}

/// Conjunto completo de regras.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Rules {
    /// Arena.
    pub arena: ArenaRules,
    /// Atributos.
    pub attributes: AttributeRules,
    /// Derivados.
    pub derived: DerivedRules,
    /// Acerto.
    pub hit: HitRules,
    /// Dano.
    pub damage: DamageRules,
    /// Ações.
    pub actions: ActionRules,
    /// Término.
    pub fight_end: FightEndRules,
    /// Progressão.
    pub progression: ProgressionRules,
    /// Arquétipos.
    pub archetypes: Archetypes,
    /// IA.
    pub ai: AiRules,
}

/// Tipo de item.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum ItemKind {
    /// Arma.
    Weapon,
    /// Armadura.
    Armor,
    /// Escudo.
    Shield,
}

/// Definição de item.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ItemDef {
    /// Identificador estável.
    pub id: ItemId,
    /// Tipo.
    pub kind: ItemKind,
    /// Nome.
    pub name: Localized,
    /// Preço de compra.
    pub price: u32,
    /// Dano adicional (armas).
    #[serde(default)]
    pub damage: i32,
    /// Modificador de precisão (armas, ‰).
    #[serde(default)]
    pub accuracy_pm: i32,
    /// Custo extra de fôlego por ataque (armas).
    #[serde(default)]
    pub weight: i32,
    /// Armadura (armaduras).
    #[serde(default)]
    pub armor: i32,
    /// Penalidade de Agilidade (armaduras).
    #[serde(default)]
    pub agility_penalty: i32,
    /// Valor de escudo (escudos).
    #[serde(default)]
    pub shield: i32,
    /// Penalidade de fôlego máximo (escudos).
    #[serde(default)]
    pub stamina_penalty: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ItemsFile {
    items: Vec<ItemDef>,
}

/// Arquétipo de IA.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum Archetype {
    /// Prioriza dano.
    Aggressive,
    /// Prioriza segurança.
    Defensive,
    /// Explora fraquezas momentâneas.
    Opportunist,
    /// Pesos médios.
    Balanced,
}

/// Oponente de uma liga.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OpponentDef {
    /// Identificador dentro da liga.
    pub id: u16,
    /// Nome exibido.
    pub name: String,
    /// Nível.
    pub level: u8,
    /// Arquétipo de IA.
    pub archetype: Archetype,
    /// Pesos de distribuição de pontos na ordem dos atributos.
    pub build: [u16; ATTR_COUNT],
    /// Arma.
    pub weapon: Option<ItemId>,
    /// Armadura.
    pub armor: Option<ItemId>,
    /// Escudo.
    pub shield: Option<ItemId>,
}

/// Liga: sequência de oponentes; o último é o campeão.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LeagueDef {
    /// Identificador.
    pub id: u16,
    /// Nome.
    pub name: Localized,
    /// Oponentes em ordem.
    pub opponents: Vec<OpponentDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct LeaguesFile {
    leagues: Vec<LeagueDef>,
}

/// Conteúdo completo validado.
#[derive(Debug, Clone)]
pub struct Content {
    /// Regras.
    pub rules: Rules,
    /// Itens.
    pub items: Vec<ItemDef>,
    /// Ligas.
    pub leagues: Vec<LeagueDef>,
    /// Versão do balanceamento: hash FNV-1a de 64 bits dos três arquivos, em hexadecimal.
    pub balance_version: String,
}

/// Erro de carga ou validação de conteúdo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContentError {
    /// JSON inválido.
    Parse {
        /// Arquivo.
        file: &'static str,
        /// Mensagem do parser.
        message: String,
    },
    /// Regra violada.
    Invalid(String),
}

impl fmt::Display for ContentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Parse { file, message } => write!(f, "{file}: {message}"),
            Self::Invalid(m) => write!(f, "conteúdo inválido: {m}"),
        }
    }
}

impl std::error::Error for ContentError {}

/// Hash FNV-1a de 64 bits (estável entre versões do Rust e plataformas).
pub fn fnv1a64(parts: &[&[u8]]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for part in parts {
        for &b in *part {
            h ^= u64::from(b);
            h = h.wrapping_mul(0x0000_0100_0000_01b3);
        }
        // Separador entre arquivos, para que mover bytes de um arquivo para outro mude o hash.
        h ^= 0xff;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

impl Content {
    /// Carrega o conteúdo embutido no executável.
    pub fn embedded() -> Result<Self, ContentError> {
        Self::from_json(EMBEDDED_RULES, EMBEDDED_ITEMS, EMBEDDED_LEAGUES)
    }

    /// Carrega e valida conteúdo a partir dos textos JSON.
    pub fn from_json(rules: &str, items: &str, leagues: &str) -> Result<Self, ContentError> {
        let parsed_rules: Rules = serde_json::from_str(rules).map_err(|e| ContentError::Parse {
            file: "rules.json",
            message: e.to_string(),
        })?;
        let parsed_items: ItemsFile =
            serde_json::from_str(items).map_err(|e| ContentError::Parse {
                file: "items.json",
                message: e.to_string(),
            })?;
        let parsed_leagues: LeaguesFile =
            serde_json::from_str(leagues).map_err(|e| ContentError::Parse {
                file: "leagues.json",
                message: e.to_string(),
            })?;
        let hash = fnv1a64(&[rules.as_bytes(), items.as_bytes(), leagues.as_bytes()]);
        let content = Self {
            rules: parsed_rules,
            items: parsed_items.items,
            leagues: parsed_leagues.leagues,
            balance_version: format!("{hash:016x}"),
        };
        content.validate()?;
        Ok(content)
    }

    /// Procura um item pelo identificador.
    pub fn item(&self, id: ItemId) -> Option<&ItemDef> {
        self.items.iter().find(|i| i.id == id)
    }

    /// Procura um oponente.
    pub fn opponent(&self, league: usize, index: usize) -> Option<&OpponentDef> {
        self.leagues.get(league)?.opponents.get(index)
    }

    /// Item inicial gratuito (a arma de preço 0 com menor identificador), se houver.
    pub fn starting_weapon(&self) -> Option<ItemId> {
        self.items
            .iter()
            .filter(|i| i.kind == ItemKind::Weapon && i.price == 0)
            .map(|i| i.id)
            .min()
    }

    fn validate(&self) -> Result<(), ContentError> {
        let r = &self.rules;
        let bad = |m: String| Err(ContentError::Invalid(m));
        if r.arena.cells < 4 || r.arena.cells > 32 {
            return bad("arena.cells deve estar entre 4 e 32".into());
        }
        if r.arena.player_start >= r.arena.opponent_start || r.arena.opponent_start >= r.arena.cells
        {
            return bad("posições iniciais inválidas".into());
        }
        if r.attributes.min == 0 || r.attributes.min > r.attributes.max {
            return bad("attributes.min/max inválidos".into());
        }
        if r.attributes.start_value < r.attributes.min
            || r.attributes.start_value > r.attributes.max
        {
            return bad("attributes.start_value fora dos limites".into());
        }
        if r.derived.regen_divisor <= 0 || r.derived.initiative_die == 0 {
            return bad("derived.regen_divisor e initiative_die devem ser positivos".into());
        }
        if r.derived.stamina_min <= 0 {
            return bad("derived.stamina_min deve ser positivo".into());
        }
        if r.hit.min_pm < 0 || r.hit.min_pm > r.hit.max_pm || r.hit.max_pm > 1000 {
            return bad("hit.min_pm/max_pm inválidos".into());
        }
        if r.damage.variance_min_pm > r.damage.variance_max_pm || r.damage.absorb_scale <= 0 {
            return bad("damage inválido".into());
        }
        if r.damage.min_damage < 1 {
            return bad("damage.min_damage deve ser >= 1".into());
        }
        for (name, a) in [
            ("light", &r.actions.light),
            ("medium", &r.actions.medium),
            ("heavy", &r.actions.heavy),
        ] {
            if a.range_min == 0 || a.range_min > a.range_max || a.stamina < 0 {
                return bad(format!("actions.{name}: alcance ou custo inválido"));
            }
        }
        if r.fight_end.max_turns == 0 || r.fight_end.exhaustion_turn > r.fight_end.max_turns {
            return bad("fight_end inválido".into());
        }
        if r.progression.max_level == 0 {
            return bad("progression.max_level deve ser >= 1".into());
        }
        let mut ids = HashSet::new();
        for item in &self.items {
            if !ids.insert(item.id) {
                return bad(format!("item {} duplicado", item.id));
            }
            if item.weight < 0 || item.armor < 0 || item.shield < 0 {
                return bad(format!("item {}: valores negativos", item.id));
            }
        }
        if self.starting_weapon().is_none() {
            return bad("é necessária ao menos uma arma com preço 0 (arma inicial)".into());
        }
        if self.leagues.is_empty() {
            return bad("é necessária ao menos uma liga".into());
        }
        for league in &self.leagues {
            if league.opponents.is_empty() {
                return bad(format!("liga {} sem oponentes", league.id));
            }
            for o in &league.opponents {
                if o.level == 0 || o.level > r.progression.max_level {
                    return bad(format!("oponente {}: nível inválido", o.name));
                }
                if o.build.iter().all(|&w| w == 0) {
                    return bad(format!("oponente {}: build sem pesos", o.name));
                }
                for (slot, id, kind) in [
                    ("weapon", o.weapon, ItemKind::Weapon),
                    ("armor", o.armor, ItemKind::Armor),
                    ("shield", o.shield, ItemKind::Shield),
                ] {
                    if let Some(id) = id {
                        match self.item(id) {
                            Some(def) if def.kind == kind => {}
                            _ => return bad(format!("oponente {}: {slot} {id} inválido", o.name)),
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_content_is_valid() {
        let c = Content::embedded();
        assert!(c.is_ok(), "{c:?}");
    }

    #[test]
    fn balance_version_changes_with_content() {
        let a = Content::from_json(EMBEDDED_RULES, EMBEDDED_ITEMS, EMBEDDED_LEAGUES)
            .map(|c| c.balance_version);
        let items2 = EMBEDDED_ITEMS.replace("\"price\": 60", "\"price\": 61");
        let b = Content::from_json(EMBEDDED_RULES, &items2, EMBEDDED_LEAGUES)
            .map(|c| c.balance_version);
        assert!(a.is_ok() && b.is_ok());
        assert_ne!(a, b);
    }

    #[test]
    fn duplicate_item_is_rejected() {
        let items = r#"{"items":[
            {"id":1,"kind":"weapon","name":{"pt":"a","en":"a"},"price":0},
            {"id":1,"kind":"weapon","name":{"pt":"b","en":"b"},"price":0}]}"#;
        let r = Content::from_json(EMBEDDED_RULES, items, EMBEDDED_LEAGUES);
        assert!(matches!(r, Err(ContentError::Invalid(_))));
    }

    #[test]
    fn unknown_rule_field_is_rejected() {
        let rules = EMBEDDED_RULES.replacen("\"cells\": 10", "\"cells\": 10, \"typo\": 1", 1);
        let r = Content::from_json(&rules, EMBEDDED_ITEMS, EMBEDDED_LEAGUES);
        assert!(matches!(r, Err(ContentError::Parse { .. })));
    }
}
