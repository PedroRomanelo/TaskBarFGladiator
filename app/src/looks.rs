//! Do estado do jogo para o desenho: aparência do jogador, aparência dos oponentes e a arte de
//! cada item (portável).
//!
//! O crate `game` só guarda índices e identificadores; aqui eles viram um [`Look`], que é o que o
//! renderizador sabe desenhar.

use assets::{Armor, Beard, Hair, Look, Shield, Weapon};
use game::{Appearance, Content, Equipment, ItemId, ItemKind};

/// Quantidade de tons de pele.
pub const SKINS: u8 = assets::SKIN_TONES.len() as u8;
/// Quantidade de cortes de cabelo.
pub const HAIRS: u8 = Hair::ALL.len() as u8;
/// Quantidade de cores de cabelo.
pub const HAIR_COLORS: u8 = assets::HAIR_COLORS.len() as u8;
/// Quantidade de cores de olhos.
pub const EYES: u8 = assets::EYE_COLORS.len() as u8;
/// Quantidade de barbas.
pub const BEARDS: u8 = Beard::ALL.len() as u8;

/// Arte de cada tipo de item, do mais barato ao mais caro. O item usa a arte da sua posição na
/// loja; itens além da lista repetem a última. O equipamento é fixo: o mesmo item tem o mesmo
/// desenho para qualquer lutador.
const WEAPON_ART: [Weapon; 4] = [
    Weapon::Knife,
    Weapon::Shortsword,
    Weapon::Spear,
    Weapon::Axe,
];
const ARMOR_ART: [Armor; 4] = [Armor::Tunic, Armor::Leather, Armor::Mail, Armor::Plates];
const SHIELD_ART: [Shield; 4] = [Shield::Buckler, Shield::Round, Shield::Oval, Shield::Tower];

/// Posição de um item entre os do mesmo tipo, ordenados por preço (e por identificador no empate).
fn tier(content: &Content, id: ItemId, kind: ItemKind) -> Option<usize> {
    let item = content.item(id).filter(|i| i.kind == kind)?;
    Some(
        content
            .items
            .iter()
            .filter(|i| i.kind == kind && (i.price, i.id) < (item.price, item.id))
            .count(),
    )
}

fn art<T: Copy + Default>(
    content: &Content,
    id: Option<ItemId>,
    kind: ItemKind,
    table: &[T; 4],
) -> T {
    id.and_then(|id| tier(content, id, kind))
        .map_or_else(T::default, |t| table[t.min(table.len() - 1)])
}

/// Desenho de um lutador a partir da aparência e do equipamento em uso.
pub fn look(content: &Content, appearance: Appearance, equipment: &Equipment) -> Look {
    Look {
        skin: appearance.skin % SKINS,
        hair: Hair::from_index(appearance.hair),
        hair_color: appearance.hair_color % HAIR_COLORS,
        eyes: appearance.eyes % EYES,
        beard: Beard::from_index(appearance.beard),
        weapon: art(content, equipment.weapon, ItemKind::Weapon, &WEAPON_ART),
        armor: art(content, equipment.armor, ItemKind::Armor, &ARMOR_ART),
        shield: art(content, equipment.shield, ItemKind::Shield, &SHIELD_ART),
    }
}

/// Aparência de um oponente, derivada do nome: cada oponente tem sempre a mesma cara, sem precisar
/// de dados extras no conteúdo.
pub fn opponent_appearance(name: &str) -> Appearance {
    // FNV-1a de 32 bits.
    let mut h: u32 = 0x811c_9dc5;
    for b in name.bytes() {
        h ^= u32::from(b);
        h = h.wrapping_mul(0x0100_0193);
    }
    let pick = |shift: u32, count: u8| ((h >> shift) % u32::from(count)) as u8;
    Appearance {
        skin: pick(0, SKINS),
        hair: pick(5, HAIRS),
        hair_color: pick(10, HAIR_COLORS),
        eyes: pick(15, EYES),
        beard: pick(20, BEARDS),
    }
}

/// Nome curto para a legenda e a faixa de status: a primeira palavra do nome.
pub fn short_name(name: &str) -> String {
    name.split(|c: char| !(c.is_alphanumeric() || c == '-'))
        .find(|w| !w.is_empty())
        .unwrap_or("")
        .chars()
        .take(10)
        .collect()
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
    fn items_map_to_art_by_shop_order() {
        let c = content();
        let eq = Equipment {
            weapon: Some(3),
            armor: Some(104),
            shield: Some(201),
        };
        let l = look(&c, Appearance::default(), &eq);
        assert_eq!(l.weapon, Weapon::Spear);
        assert_eq!(l.armor, Armor::Plates);
        assert_eq!(l.shield, Shield::Buckler);
        let bare = look(&c, Appearance::default(), &Equipment::default());
        assert_eq!(
            (bare.weapon, bare.armor, bare.shield),
            (Weapon::None, Armor::None, Shield::None)
        );
    }

    #[test]
    fn every_content_item_has_art() {
        let c = content();
        for item in &c.items {
            let mut eq = Equipment::default();
            *eq.slot_mut(item.kind) = Some(item.id);
            let l = look(&c, Appearance::default(), &eq);
            let drawn = match item.kind {
                ItemKind::Weapon => l.weapon != Weapon::None,
                ItemKind::Armor => l.armor != Armor::None,
                ItemKind::Shield => l.shield != Shield::None,
            };
            assert!(drawn, "item {} sem desenho", item.id);
        }
    }

    #[test]
    fn out_of_range_appearance_wraps() {
        let c = content();
        let a = Appearance {
            skin: 250,
            hair: 250,
            hair_color: 250,
            eyes: 250,
            beard: 250,
        };
        let l = look(&c, a, &Equipment::default());
        assert!(l.skin < SKINS && l.hair_color < HAIR_COLORS && l.eyes < EYES);
    }

    #[test]
    fn opponents_keep_their_face_and_names_are_shortened() {
        assert_eq!(
            opponent_appearance("Taro, o Pescador"),
            opponent_appearance("Taro, o Pescador")
        );
        assert_ne!(
            opponent_appearance("Taro, o Pescador"),
            opponent_appearance("Verna das Docas")
        );
        assert_eq!(short_name("Taro, o Pescador"), "Taro");
        assert_eq!(short_name("Ossian Mão-Pesada"), "Ossian");
        assert_eq!(short_name(""), "");
    }
}
