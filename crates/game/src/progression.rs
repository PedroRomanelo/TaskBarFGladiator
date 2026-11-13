//! Experiência, níveis e recompensas. Ver `docs/11-design-do-jogo.md`, seção 6.

use crate::content::Content;

/// XP necessário para passar do nível `n` para `n + 1`.
pub fn xp_to_next(content: &Content, level: u8) -> u32 {
    let c = &content.rules.progression.xp_curve;
    let n = u32::from(level);
    c.a.saturating_add(c.b.saturating_mul(n))
        .saturating_add(c.c.saturating_mul(n).saturating_mul(n))
}

/// Recompensa de uma luta.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Reward {
    /// XP ganho.
    pub xp: u32,
    /// Ouro ganho.
    pub gold: u32,
}

/// Calcula a recompensa de uma luta.
pub fn reward(
    content: &Content,
    player_level: u8,
    opponent_level: u8,
    won: bool,
    rematch: bool,
) -> Reward {
    let p = &content.rules.progression;
    let ol = u32::from(opponent_level);
    let mut xp = p
        .xp_win_base
        .saturating_add(p.xp_win_per_level.saturating_mul(ol));
    if opponent_level > player_level {
        xp = xp.saturating_mul(p.xp_higher_level_pm) / 1000;
    }
    let mut gold = p
        .gold_win_base
        .saturating_add(p.gold_win_per_level.saturating_mul(ol));
    if !won {
        xp = xp.saturating_mul(p.xp_loss_pm) / 1000;
        gold = gold.saturating_mul(p.gold_loss_pm) / 1000;
    }
    if rematch {
        xp = xp.saturating_mul(p.rematch_reward_pm) / 1000;
        gold = gold.saturating_mul(p.rematch_reward_pm) / 1000;
    }
    Reward { xp, gold }
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
    fn xp_curve_matches_rules() {
        let c = content();
        let k = &c.rules.progression.xp_curve;
        assert_eq!(xp_to_next(&c, 1), k.a + k.b + k.c);
        assert_eq!(xp_to_next(&c, 10), k.a + 10 * k.b + 100 * k.c);
    }

    #[test]
    fn loss_and_rematch_reduce_reward() {
        let c = content();
        let win = reward(&c, 5, 5, true, false);
        let loss = reward(&c, 5, 5, false, false);
        let re = reward(&c, 5, 5, true, true);
        assert!(loss.xp < win.xp && loss.gold < win.gold);
        assert!(re.xp < win.xp && re.gold < win.gold);
    }

    #[test]
    fn higher_level_opponent_gives_bonus_xp() {
        let c = content();
        let same = reward(&c, 5, 5, true, false);
        let higher = reward(&c, 4, 5, true, false);
        assert!(higher.xp > same.xp);
    }
}
