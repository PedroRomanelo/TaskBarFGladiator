//! Testes de propriedade: invariantes do combate, do estado e do save (docs/09, seção 2.1).

use game::content::{ATTR_COUNT, ItemKind};
use game::{
    Archetype, Attributes, Content, Equipment, FightState, Fighter, GameState, Input, Mode,
    OpponentRef, Side, step,
};
use proptest::prelude::*;

fn content() -> Content {
    match Content::embedded() {
        Ok(c) => c,
        Err(e) => panic!("{e}"),
    }
}

fn archetype() -> impl Strategy<Value = Archetype> {
    prop_oneof![
        Just(Archetype::Aggressive),
        Just(Archetype::Defensive),
        Just(Archetype::Opportunist),
        Just(Archetype::Balanced),
    ]
}

fn fighter(c: &Content) -> impl Strategy<Value = Fighter> + use<> {
    let weapons: Vec<Option<u16>> = std::iter::once(None)
        .chain(
            c.items
                .iter()
                .filter(|i| i.kind == ItemKind::Weapon)
                .map(|i| Some(i.id)),
        )
        .collect();
    let armors: Vec<Option<u16>> = std::iter::once(None)
        .chain(
            c.items
                .iter()
                .filter(|i| i.kind == ItemKind::Armor)
                .map(|i| Some(i.id)),
        )
        .collect();
    let shields: Vec<Option<u16>> = std::iter::once(None)
        .chain(
            c.items
                .iter()
                .filter(|i| i.kind == ItemKind::Shield)
                .map(|i| Some(i.id)),
        )
        .collect();
    (
        prop::array::uniform6(1u8..=99),
        1u8..=30,
        prop::sample::select(weapons),
        prop::sample::select(armors),
        prop::sample::select(shields),
    )
        .prop_map(|(attrs, level, weapon, armor, shield)| Fighter {
            name: "p".into(),
            level,
            attrs: Attributes(attrs),
            equipment: Equipment {
                weapon,
                armor,
                shield,
            },
        })
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    #[test]
    fn every_fight_ends_and_stays_in_bounds(
        seed in any::<u64>(),
        pa in archetype(),
        oa in archetype(),
        (p, o) in {
            let c = content();
            (fighter(&c), fighter(&c))
        },
    ) {
        let c = content();
        let mut f = FightState::new(&c, &p, pa, &o, oa, OpponentRef { league: 0, index: 0 }, false, seed);
        let mut ev = Vec::new();
        let max = c.rules.fight_end.max_turns;
        let mut rounds = 0u32;
        while f.outcome.is_none() {
            let a = game::ai::choose_action(&c, &mut f, Side::Player);
            let b = game::ai::choose_action(&c, &mut f, Side::Opponent);
            prop_assert!(f.is_valid(&c, Side::Player, a));
            prop_assert!(f.is_valid(&c, Side::Opponent, b));
            f.play_round(&c, a, b, &mut ev);
            rounds += 1;
            prop_assert!(rounds <= u32::from(max));
            for x in &f.fighters {
                prop_assert!(x.hp >= 0 && x.hp <= x.stats.hp_max);
                prop_assert!(x.stamina >= 0 && x.stamina <= x.stamina_cap);
                prop_assert!(x.stamina_cap <= x.stats.stamina_max);
                prop_assert!(x.pos < c.rules.arena.cells);
            }
            prop_assert!(f.fighters[0].pos < f.fighters[1].pos);
        }
        prop_assert!(f.turn <= max);
    }

    #[test]
    fn same_seed_same_result(seed in any::<u64>(), (p, o) in {
        let c = content();
        (fighter(&c), fighter(&c))
    }) {
        let c = content();
        let play = || {
            let mut f = FightState::new(&c, &p, Archetype::Balanced, &o, Archetype::Aggressive,
                OpponentRef { league: 0, index: 0 }, false, seed);
            let mut ev = Vec::new();
            while f.outcome.is_none() {
                let a = game::ai::choose_action(&c, &mut f, Side::Player);
                let b = game::ai::choose_action(&c, &mut f, Side::Opponent);
                f.play_round(&c, a, b, &mut ev);
            }
            (f, ev)
        };
        prop_assert_eq!(play(), play());
    }

    #[test]
    fn save_round_trip_after_random_inputs(seed in any::<u64>(), inputs in prop::collection::vec(0u8..12, 0..200)) {
        let c = content();
        let mut s = GameState::new(&c, "Prop", seed);
        for k in inputs {
            let input = match k {
                0 => Input::StartFight(OpponentRef { league: 0, index: 0 }),
                1 => Input::StartFight(OpponentRef { league: 0, index: 1 }),
                2..=4 => Input::AutoRound,
                5 => Input::Continue,
                6 => Input::SetMode(Mode::Auto(Archetype::Balanced)),
                7 => Input::AllocatePoint(game::Attr::ALL[usize::from(k) % ATTR_COUNT]),
                8 => Input::Buy(101),
                9 => Input::Equip(101),
                10 => Input::Sell(101),
                _ => Input::FightAction(game::Action::Defend),
            };
            step(&c, &mut s, input);
            prop_assert!(s.gold < u32::MAX);
        }
        let bytes = match game::save::to_bytes(&c, &s) {
            Ok(b) => b,
            Err(e) => return Err(TestCaseError::fail(e.to_string())),
        };
        let loaded = game::save::from_bytes(&c, &bytes).map(|l| l.state);
        prop_assert_eq!(loaded, Ok(s));
    }

    #[test]
    fn arbitrary_bytes_never_panic(bytes in prop::collection::vec(any::<u8>(), 0..2048)) {
        let c = content();
        let _ = game::save::from_bytes(&c, &bytes);
    }
}
