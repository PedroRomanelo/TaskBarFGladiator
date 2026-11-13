//! Gerador pseudoaleatório PCG32 (XSH-RR), determinístico e serializável.
//!
//! Implementação própria para garantir o mesmo resultado em qualquer plataforma e versão de
//! dependência. Referência do algoritmo: M. E. O'Neill, "PCG: A Family of Simple Fast
//! Space-Efficient Statistically Good Algorithms for Random Number Generation" (2014).

use serde::{Deserialize, Serialize};

const MULTIPLIER: u64 = 6_364_136_223_846_793_005;

/// Estado do gerador.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Pcg32 {
    state: u64,
    inc: u64,
}

impl Pcg32 {
    /// Cria um gerador a partir de uma semente e de um identificador de sequência.
    pub fn new(seed: u64, stream: u64) -> Self {
        let mut rng = Self {
            state: 0,
            inc: (stream << 1) | 1,
        };
        rng.next_u32();
        rng.state = rng.state.wrapping_add(seed);
        rng.next_u32();
        rng
    }

    /// Próximo número de 32 bits.
    pub fn next_u32(&mut self) -> u32 {
        let old = self.state;
        self.state = old.wrapping_mul(MULTIPLIER).wrapping_add(self.inc);
        let xorshifted = (((old >> 18) ^ old) >> 27) as u32;
        let rot = (old >> 59) as u32;
        xorshifted.rotate_right(rot)
    }

    /// Próximo número de 64 bits.
    pub fn next_u64(&mut self) -> u64 {
        (u64::from(self.next_u32()) << 32) | u64::from(self.next_u32())
    }

    /// Inteiro uniforme em `[0, bound)`, sem viés (método de rejeição). `bound == 0` retorna 0.
    pub fn below(&mut self, bound: u32) -> u32 {
        if bound == 0 {
            return 0;
        }
        let threshold = bound.wrapping_neg() % bound;
        loop {
            let r = self.next_u32();
            if r >= threshold {
                return r % bound;
            }
        }
    }

    /// Inteiro uniforme em `[lo, hi]` (inclusivo). Se `hi < lo`, retorna `lo`.
    pub fn range_inclusive(&mut self, lo: u32, hi: u32) -> u32 {
        if hi <= lo {
            return lo;
        }
        lo + self.below(hi - lo + 1)
    }

    /// Verdadeiro com probabilidade `per_mille / 1000`. Valores fora de `[0, 1000]` são limitados.
    pub fn chance(&mut self, per_mille: i32) -> bool {
        let p = per_mille.clamp(0, 1000) as u32;
        self.below(1000) < p
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_same_sequence() {
        let mut a = Pcg32::new(42, 7);
        let mut b = Pcg32::new(42, 7);
        for _ in 0..1000 {
            assert_eq!(a.next_u32(), b.next_u32());
        }
    }

    #[test]
    fn different_streams_differ() {
        let mut a = Pcg32::new(42, 1);
        let mut b = Pcg32::new(42, 2);
        let same = (0..100).filter(|_| a.next_u32() == b.next_u32()).count();
        assert!(same < 5);
    }

    #[test]
    fn matches_reference_pcg32() {
        // Vetor de teste da implementação de referência (pcg32-demo: semente 42, sequência 54).
        // Detecta qualquer mudança acidental no algoritmo, que mudaria todo o balanceamento.
        let mut r = Pcg32::new(42, 54);
        let got: Vec<u32> = (0..6).map(|_| r.next_u32()).collect();
        assert_eq!(
            got,
            vec![
                0xa15c02b7, 0x7b47f409, 0xba1d3330, 0x83d2f293, 0xbfa4784b, 0xcbed606e
            ]
        );
    }

    #[test]
    fn below_is_in_range_and_roughly_uniform() {
        let mut r = Pcg32::new(9, 9);
        let mut counts = [0u32; 10];
        for _ in 0..100_000 {
            let v = r.below(10) as usize;
            counts[v] += 1;
        }
        for c in counts {
            assert!((9_000..=11_000).contains(&c), "{c}");
        }
    }

    #[test]
    fn chance_extremes() {
        let mut r = Pcg32::new(3, 3);
        assert!((0..1000).all(|_| !r.chance(0)));
        assert!((0..1000).all(|_| r.chance(1000)));
        assert!((0..1000).all(|_| r.chance(5000)));
        assert!((0..1000).all(|_| !r.chance(-5)));
    }
}
