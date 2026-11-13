//! Simulação de carreiras completas (docs/12, seção 4.2).

use game::{
    Archetype, Attr, Content, GameState, Input, ItemKind, Mode, OpponentRef, Screen, Side, step,
};
use std::fmt::Write as _;

/// Limite de lutas por carreira (proteção contra carreiras que nunca terminam).
pub const MAX_FIGHTS: u32 = 1_000;

/// Resultado de uma carreira.
#[derive(Debug, Clone, Default)]
pub struct CareerResult {
    /// Semente.
    pub seed: u64,
    /// Liga 1 concluída.
    pub completed: bool,
    /// Nível ao vencer o campeão (ou nível final).
    pub level: u8,
    /// Lutas disputadas.
    pub fights: u32,
    /// Vitórias.
    pub wins: u32,
    /// Itens comprados.
    pub items_bought: u32,
    /// Maior sequência de derrotas.
    pub max_loss_streak: u32,
    /// Ouro sem uso ao final.
    pub unused_gold: u32,
    /// Rodadas jogadas no total.
    pub rounds: u64,
}

const ALLOC_ORDER: [Attr; 6] = Attr::ALL;

fn allocate_balanced(content: &Content, s: &mut GameState) {
    let mut k = 0usize;
    while s.unspent_points > 0 && k < 10_000 {
        // Distribui no atributo de menor valor (empate: ordem fixa), mantendo a build equilibrada.
        let attr = ALLOC_ORDER
            .iter()
            .copied()
            .min_by_key(|&a| (s.player.attrs.get(a), a.index()))
            .unwrap_or(Attr::Strength);
        let r = step(content, s, Input::AllocatePoint(attr));
        if r.events
            .iter()
            .any(|e| matches!(e, game::GameEvent::Rejected(_)))
        {
            break;
        }
        k += 1;
    }
}

fn shop(content: &Content, s: &mut GameState) {
    for kind in [ItemKind::Weapon, ItemKind::Armor, ItemKind::Shield] {
        let current_price = s
            .player
            .equipment
            .slot(kind)
            .and_then(|id| content.item(id))
            .map_or(0, |i| i.price);
        let best = content
            .items
            .iter()
            .filter(|i| i.kind == kind && i.price > current_price && i.price <= s.gold)
            .max_by_key(|i| (i.price, i.id))
            .map(|i| i.id);
        if let Some(id) = best {
            step(content, s, Input::Buy(id));
            step(content, s, Input::Equip(id));
        }
    }
}

/// Executa uma carreira até concluir a liga 1 ou atingir [`MAX_FIGHTS`].
pub fn run(content: &Content, seed: u64) -> CareerResult {
    run_traced(content, seed, None)
}

/// Igual a [`run`], registrando cada luta em `log` quando presente.
pub fn run_traced(content: &Content, seed: u64, mut log: Option<&mut String>) -> CareerResult {
    let mut s = GameState::new(content, "sim", seed);
    step(
        content,
        &mut s,
        Input::SetMode(Mode::Auto(Archetype::Balanced)),
    );
    let league_len = content.leagues.first().map_or(0, |l| l.opponents.len()) as u16;
    let mut losses_vs_next = 0u32;
    let mut rounds = 0u64;
    let mut result = CareerResult {
        seed,
        ..Default::default()
    };
    while s.stats.fights < MAX_FIGHTS {
        allocate_balanced(content, &mut s);
        shop(content, &mut s);
        let next = s.league_progress.first().copied().unwrap_or(0);
        if next >= league_len {
            result.completed = true;
            break;
        }
        // Política: treina contra o oponente anterior enquanto estiver mais de um nível abaixo do
        // próximo, ou depois de duas derrotas seguidas contra ele.
        let next_level = content
            .opponent(0, usize::from(next))
            .map_or(1, |o| o.level);
        let underleveled = s.player.level.saturating_add(1) < next_level;
        let index = if next > 0 && (underleveled || losses_vs_next >= 2) {
            losses_vs_next = 0;
            next - 1
        } else {
            next
        };
        step(
            content,
            &mut s,
            Input::StartFight(OpponentRef { league: 0, index }),
        );
        while matches!(s.screen, Screen::Fight(_)) {
            step(content, &mut s, Input::AutoRound);
            rounds += 1;
        }
        if let (Some(log), Screen::PostFight(summary)) = (log.as_deref_mut(), &s.screen) {
            let _ = writeln!(
                log,
                "luta {:3} nivel {:2} vs #{} ({}) -> {} em {} turnos | ouro {}",
                s.stats.fights,
                s.player.level,
                summary.opponent.index,
                summary.opponent_name,
                if summary.outcome.winner == Side::Player {
                    "V"
                } else {
                    "d"
                },
                summary.outcome.turns,
                s.gold
            );
        }
        if let Screen::PostFight(summary) = &s.screen
            && summary.opponent.index == next
        {
            if summary.outcome.winner == Side::Player {
                losses_vs_next = 0;
            } else {
                losses_vs_next += 1;
            }
        }
        step(content, &mut s, Input::Continue);
    }
    result.level = s.player.level;
    result.fights = s.stats.fights;
    result.wins = s.stats.wins;
    result.items_bought = s.stats.items_bought;
    result.max_loss_streak = s.stats.max_loss_streak;
    result.unused_gold = s.gold;
    result.rounds = rounds;
    result
}

/// Executa `n` carreiras (sementes `1..=n`) em paralelo.
pub fn run_many(content: &Content, n: u32) -> Vec<CareerResult> {
    let seeds: Vec<u64> = (1..=u64::from(n)).collect();
    let threads = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1)
        .clamp(1, 16);
    let mut out: Vec<CareerResult> = vec![CareerResult::default(); seeds.len()];
    std::thread::scope(|scope| {
        let size = seeds.len().div_ceil(threads).max(1);
        for (chunk, seed_chunk) in out.chunks_mut(size).zip(seeds.chunks(size)) {
            scope.spawn(move || {
                for (slot, &seed) in chunk.iter_mut().zip(seed_chunk) {
                    *slot = run(content, seed);
                }
            });
        }
    });
    out
}

/// CSV determinístico.
pub fn to_csv(results: &[CareerResult]) -> String {
    let mut s = String::from(
        "semente,concluida,nivel,lutas,vitorias,itens,maior_seq_derrotas,ouro_sem_uso,rodadas\n",
    );
    for r in results {
        let _ = writeln!(
            s,
            "{},{},{},{},{},{},{},{},{}",
            r.seed,
            u8::from(r.completed),
            r.level,
            r.fights,
            r.wins,
            r.items_bought,
            r.max_loss_streak,
            r.unused_gold,
            r.rounds
        );
    }
    s
}
