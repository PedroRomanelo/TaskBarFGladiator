//! Lutador: atributos, equipamento e atributos derivados.

use crate::content::{ATTR_COUNT, Content, ItemId, ItemKind, OpponentDef};
use serde::{Deserialize, Serialize};

/// Atributo primário.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Attr {
    /// Força: dano.
    Strength = 0,
    /// Técnica: precisão e crítico.
    Technique = 1,
    /// Agilidade: esquiva e iniciativa.
    Agility = 2,
    /// Defesa: absorção.
    Defense = 3,
    /// Vitalidade: HP.
    Vitality = 4,
    /// Fôlego: fôlego máximo e recuperação.
    Stamina = 5,
}

impl Attr {
    /// Todos os atributos na ordem do vetor.
    pub const ALL: [Attr; ATTR_COUNT] = [
        Attr::Strength,
        Attr::Technique,
        Attr::Agility,
        Attr::Defense,
        Attr::Vitality,
        Attr::Stamina,
    ];

    /// Índice no vetor de atributos.
    pub const fn index(self) -> usize {
        self as usize
    }
}

/// Valores dos seis atributos primários.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Attributes(pub [u8; ATTR_COUNT]);

impl Attributes {
    /// Valor de um atributo.
    pub fn get(&self, a: Attr) -> u8 {
        self.0[a.index()]
    }

    /// Altera um atributo.
    pub fn set(&mut self, a: Attr, v: u8) {
        self.0[a.index()] = v;
    }

    /// Soma de todos os atributos.
    pub fn total(&self) -> u32 {
        self.0.iter().map(|&v| u32::from(v)).sum()
    }
}

/// Equipamento em uso.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Equipment {
    /// Arma.
    pub weapon: Option<ItemId>,
    /// Armadura.
    pub armor: Option<ItemId>,
    /// Escudo.
    pub shield: Option<ItemId>,
}

impl Equipment {
    /// Slot correspondente a um tipo de item.
    pub fn slot_mut(&mut self, kind: ItemKind) -> &mut Option<ItemId> {
        match kind {
            ItemKind::Weapon => &mut self.weapon,
            ItemKind::Armor => &mut self.armor,
            ItemKind::Shield => &mut self.shield,
        }
    }

    /// Item equipado em um slot.
    pub fn slot(&self, kind: ItemKind) -> Option<ItemId> {
        match kind {
            ItemKind::Weapon => self.weapon,
            ItemKind::Armor => self.armor,
            ItemKind::Shield => self.shield,
        }
    }
}

/// Definição de um lutador (persistente entre lutas).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Fighter {
    /// Nome exibido.
    pub name: String,
    /// Nível.
    pub level: u8,
    /// Atributos.
    pub attrs: Attributes,
    /// Equipamento.
    pub equipment: Equipment,
}

/// Atributos derivados prontos para o combate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Stats {
    /// HP máximo.
    pub hp_max: i32,
    /// Fôlego máximo.
    pub stamina_max: i32,
    /// Recuperação de fôlego por turno.
    pub regen: i32,
    /// Dano base (antes do multiplicador da ação).
    pub damage: i32,
    /// Técnica efetiva.
    pub technique: i32,
    /// Agilidade efetiva (após penalidade da armadura, mínimo 1).
    pub agility: i32,
    /// Chance de crítico base (‰).
    pub crit_pm: i32,
    /// Absorção.
    pub absorb: i32,
    /// Modificador de precisão da arma (‰).
    pub accuracy_pm: i32,
    /// Custo extra de fôlego por ataque.
    pub weight: i32,
}

impl Fighter {
    /// Calcula os atributos derivados com as regras e itens do conteúdo. Itens desconhecidos são
    /// ignorados (o save é validado antes de chegar aqui).
    pub fn stats(&self, content: &Content) -> Stats {
        let d = &content.rules.derived;
        let a = |x: Attr| i32::from(self.attrs.get(x));
        let item = |id: Option<ItemId>| id.and_then(|i| content.item(i));
        let weapon = item(self.equipment.weapon);
        let armor = item(self.equipment.armor);
        let shield = item(self.equipment.shield);

        let hp_max = d.hp_base
            + d.hp_per_vitality * a(Attr::Vitality)
            + d.hp_per_level * i32::from(self.level);
        let stamina_penalty = shield.map_or(0, |s| s.stamina_penalty);
        let stamina_max = (d.stamina_base + d.stamina_per_point * a(Attr::Stamina)
            - stamina_penalty)
            .max(d.stamina_min);
        let regen = d.regen_base + a(Attr::Stamina) / d.regen_divisor;
        let damage = d.damage_base
            + a(Attr::Strength) * d.damage_per_strength_tenth / 10
            + weapon.map_or(0, |w| w.damage);
        let agility = (a(Attr::Agility) - armor.map_or(0, |x| x.agility_penalty)).max(1);
        let crit_pm = (d.crit_base_pm + a(Attr::Technique) * d.crit_per_technique_tenth_pm / 10)
            .min(d.crit_max_pm);
        let absorb = d.absorb_per_defense * a(Attr::Defense)
            + armor.map_or(0, |x| x.armor)
            + shield.map_or(0, |s| s.shield);
        Stats {
            hp_max: hp_max.max(1),
            stamina_max,
            regen,
            damage: damage.max(1),
            technique: a(Attr::Technique),
            agility,
            crit_pm,
            absorb: absorb.max(0),
            accuracy_pm: weapon.map_or(0, |w| w.accuracy_pm),
            weight: weapon.map_or(0, |w| w.weight),
        }
    }

    /// Total de pontos de atributo que um lutador do nível `level` possui (iniciais + livres +
    /// ganhos por nível).
    pub fn total_points_for_level(content: &Content, level: u8) -> u32 {
        let r = &content.rules.attributes;
        u32::from(r.start_value) * ATTR_COUNT as u32
            + u32::from(r.start_free_points)
            + u32::from(r.points_per_level) * u32::from(level.saturating_sub(1))
    }

    /// Distribui `points` pontos proporcionalmente aos pesos, partindo de `start_value` em cada
    /// atributo e respeitando o máximo. Determinístico: o resto vai para os maiores pesos, em
    /// ordem de atributo.
    pub fn distribute(content: &Content, weights: [u16; ATTR_COUNT], level: u8) -> Attributes {
        let r = &content.rules.attributes;
        let total = Self::total_points_for_level(content, level);
        let base = u32::from(r.start_value) * ATTR_COUNT as u32;
        let mut free = total.saturating_sub(base);
        let mut values = [r.start_value; ATTR_COUNT];
        let wsum: u32 = weights.iter().map(|&w| u32::from(w)).sum();
        if wsum == 0 {
            return Attributes(values);
        }
        let mut given = [0u32; ATTR_COUNT];
        for i in 0..ATTR_COUNT {
            given[i] = free * u32::from(weights[i]) / wsum;
        }
        let used: u32 = given.iter().sum();
        free -= used;
        let mut order: Vec<usize> = (0..ATTR_COUNT).collect();
        order.sort_by(|&x, &y| weights[y].cmp(&weights[x]).then(x.cmp(&y)));
        let mut k = 0;
        while free > 0 && k < ATTR_COUNT * 200 {
            let i = order[k % ATTR_COUNT];
            if weights[i] > 0 {
                given[i] += 1;
                free -= 1;
            }
            k += 1;
        }
        for i in 0..ATTR_COUNT {
            let v = u32::from(values[i]) + given[i];
            values[i] = v.min(u32::from(r.max)) as u8;
        }
        Attributes(values)
    }

    /// Cria o lutador de um oponente da liga.
    pub fn from_opponent(content: &Content, def: &OpponentDef) -> Self {
        Self {
            name: def.name.clone(),
            level: def.level,
            attrs: Self::distribute(content, def.build, def.level),
            equipment: Equipment {
                weapon: def.weapon,
                armor: def.armor,
                shield: def.shield,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn content() -> Content {
        match Content::embedded() {
            Ok(c) => c,
            Err(e) => panic!("{e}"),
        }
    }

    #[test]
    fn distribute_uses_all_points() {
        let c = content();
        for level in [1u8, 5, 16, 30] {
            let a = Fighter::distribute(&c, [3, 1, 1, 1, 2, 2], level);
            assert_eq!(a.total(), Fighter::total_points_for_level(&c, level));
        }
    }

    #[test]
    fn balanced_build_is_even() {
        let c = content();
        let a = Fighter::distribute(&c, [1; ATTR_COUNT], 1);
        let min = a.0.iter().min().copied().unwrap_or(0);
        let max = a.0.iter().max().copied().unwrap_or(0);
        assert!(max - min <= 1);
    }

    #[test]
    fn stats_follow_rules_example() {
        // Exemplo da seção 4 de docs/11 com os valores atuais de rules.json.
        let c = content();
        let mut f = Fighter {
            name: "t".into(),
            level: 1,
            attrs: Attributes([10, 8, 6, 5, 5, 5]),
            equipment: Equipment::default(),
        };
        f.equipment.weapon = Some(2);
        let s = f.stats(&c);
        let d = &c.rules.derived;
        assert_eq!(
            s.damage,
            d.damage_base + 10 * d.damage_per_strength_tenth / 10 + 4
        );
        assert_eq!(s.absorb, d.absorb_per_defense * 5);
        assert_eq!(s.hp_max, d.hp_base + d.hp_per_vitality * 5 + d.hp_per_level);
        assert_eq!(s.stamina_max, d.stamina_base + d.stamina_per_point * 5);
        assert_eq!(s.regen, d.regen_base + 5 / d.regen_divisor);
    }

    #[test]
    fn armor_penalty_never_drops_agility_below_one() {
        let c = content();
        let f = Fighter {
            name: "t".into(),
            level: 1,
            attrs: Attributes([5, 5, 1, 5, 5, 5]),
            equipment: Equipment {
                weapon: None,
                armor: Some(104),
                shield: Some(204),
            },
        };
        let s = f.stats(&c);
        assert_eq!(s.agility, 1);
        assert!(s.stamina_max >= c.rules.derived.stamina_min);
    }
}
