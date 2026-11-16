//! Corpo do lutador desenhado por artista, em folhas de sprites.
//!
//! As folhas PNG ficam em `assets-src/sprites/<conjunto>/` e são convertidas no build (`build.rs`) para
//! um formato próprio com paleta indexada e compressão por repetição; aqui esse formato é lido uma
//! única vez. Cada cor da paleta tem um papel ([`Role`]): as cores de time, pele, olhos e armadura são
//! trocadas no desenho, o resto fica como o artista pintou.
//!
//! Sem conjunto em `assets-src/sprites/`, [`fighter_set`] devolve `None` e o jogo usa o boneco em
//! camadas de [`crate::parts`].

use crate::Frame;
use std::sync::OnceLock;

include!(concat!(env!("OUT_DIR"), "/sprite_sets.rs"));

/// Animação do corpo. A ordem é a dos identificadores gravados pelo `build.rs`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Anim {
    /// Parado, respirando (repete).
    Idle,
    /// Andando (repete).
    Walk,
    /// Golpe leve.
    AttackLight,
    /// Golpe médio.
    AttackMedium,
    /// Golpe pesado.
    AttackHeavy,
    /// Em guarda (repete).
    Guard,
    /// Golpe aparado pelo escudo.
    Block,
    /// Golpe recebido.
    Hurt,
    /// Queda (para no último quadro).
    Death,
}

impl Anim {
    /// Todas, na ordem dos identificadores.
    pub const ALL: [Anim; 9] = [
        Anim::Idle,
        Anim::Walk,
        Anim::AttackLight,
        Anim::AttackMedium,
        Anim::AttackHeavy,
        Anim::Guard,
        Anim::Block,
        Anim::Hurt,
        Anim::Death,
    ];

    /// Animação que mostra um quadro do jogo.
    pub fn of(frame: Frame) -> Anim {
        match frame {
            Frame::Idle => Anim::Idle,
            Frame::Walk => Anim::Walk,
            Frame::Attack => Anim::AttackLight,
            Frame::AttackMedium => Anim::AttackMedium,
            Frame::AttackHeavy => Anim::AttackHeavy,
            Frame::Defend => Anim::Guard,
            Frame::Block => Anim::Block,
            Frame::Hurt => Anim::Hurt,
            Frame::Down => Anim::Death,
        }
    }

    /// Animação usada quando esta não existe no conjunto.
    fn fallback(self) -> Anim {
        match self {
            Anim::AttackHeavy => Anim::AttackMedium,
            Anim::AttackMedium => Anim::AttackLight,
            Anim::Block => Anim::Guard,
            _ => Anim::Idle,
        }
    }
}

/// Papel de uma cor da paleta.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    /// Fica como o artista pintou.
    Fixed,
    /// Cor do time; tom 0 (claro) a 2 (escuro).
    Team(u8),
    /// Pele; tom 0 a 2.
    Skin(u8),
    /// Íris.
    Eyes,
    /// Material da armadura; tom 0 a 2.
    Armor(u8),
}

impl Role {
    fn from_code(c: u8) -> Role {
        let k = c & 0x0f;
        match c >> 4 {
            1 => Role::Team(k.min(2)),
            2 => Role::Skin(k.min(2)),
            3 => Role::Eyes,
            4 => Role::Armor(k.min(2)),
            _ => Role::Fixed,
        }
    }
}

/// Um quadro: retângulo de índices de paleta (0 = transparente).
#[derive(Debug)]
pub struct SpriteFrame {
    /// Coluna da borda esquerda, relativa ao eixo do corpo.
    pub x: i32,
    /// Linha da borda de cima, relativa ao chão (negativa: acima dos pés).
    pub y: i32,
    /// Largura.
    pub w: usize,
    /// Altura.
    pub h: usize,
    /// Índices, linha por linha.
    pub px: Vec<u8>,
}

/// Sequência de quadros com a mesma duração.
#[derive(Debug)]
pub struct Animation {
    /// Duração de cada quadro (ms).
    pub ms: u32,
    /// Recomeça ao terminar; senão, para no último quadro.
    pub looped: bool,
    /// Quadros.
    pub frames: Vec<SpriteFrame>,
}

impl Animation {
    /// Duração total (ms).
    pub fn duration_ms(&self) -> u32 {
        self.ms * self.frames.len() as u32
    }

    /// Índice do quadro mostrado `t_ms` depois do início.
    pub fn frame_index(&self, t_ms: u32) -> usize {
        let n = self.frames.len().max(1);
        let i = (t_ms / self.ms.max(1)) as usize;
        if self.looped { i % n } else { i.min(n - 1) }
    }

    /// Quadro mostrado `t_ms` depois do início.
    pub fn frame_at(&self, t_ms: u32) -> Option<&SpriteFrame> {
        self.frames.get(self.frame_index(t_ms))
    }
}

/// Conjunto de animações de um corpo, com a paleta e o papel de cada cor.
#[derive(Debug)]
pub struct SpriteSet {
    /// Nome da pasta de origem.
    pub name: &'static str,
    /// Cores `0xRRGGBB` por índice (o índice 0 é transparente e não é usado).
    pub colors: Vec<u32>,
    /// Papel de cada índice.
    pub roles: Vec<Role>,
    anims: [Option<Animation>; 9],
}

struct Reader<'a> {
    b: &'a [u8],
    at: usize,
}

impl Reader<'_> {
    fn bytes(&mut self, n: usize) -> Result<&[u8], String> {
        let end = self.at.checked_add(n).filter(|&e| e <= self.b.len());
        let end = end.ok_or("conjunto de sprites truncado")?;
        let s = &self.b[self.at..end];
        self.at = end;
        Ok(s)
    }
    fn u8(&mut self) -> Result<u8, String> {
        Ok(self.bytes(1)?[0])
    }
    fn u16(&mut self) -> Result<u16, String> {
        let s = self.bytes(2)?;
        Ok(u16::from_le_bytes([s[0], s[1]]))
    }
    fn i16(&mut self) -> Result<i16, String> {
        let s = self.bytes(2)?;
        Ok(i16::from_le_bytes([s[0], s[1]]))
    }
    fn u32(&mut self) -> Result<u32, String> {
        let s = self.bytes(4)?;
        Ok(u32::from_le_bytes([s[0], s[1], s[2], s[3]]))
    }
}

impl SpriteSet {
    /// Lê um conjunto no formato gerado pelo `build.rs`.
    pub fn parse(name: &'static str, bytes: &[u8]) -> Result<SpriteSet, String> {
        let mut r = Reader { b: bytes, at: 0 };
        if r.bytes(4)? != b"GSP1" {
            return Err(format!("{name}: assinatura inválida"));
        }
        let n = usize::from(r.u8()?.max(1));
        let mut colors = vec![0];
        for _ in 1..n {
            let c = r.bytes(3)?;
            colors.push((u32::from(c[0]) << 16) | (u32::from(c[1]) << 8) | u32::from(c[2]));
        }
        let mut roles = vec![Role::Fixed];
        for _ in 1..n {
            roles.push(Role::from_code(r.u8()?));
        }
        let mut anims: [Option<Animation>; 9] = Default::default();
        for _ in 0..r.u8()? {
            let id = usize::from(r.u8()?);
            let ms = u32::from(r.u16()?);
            let looped = r.u8()? != 0;
            let count = r.u8()?;
            let mut frames = Vec::with_capacity(usize::from(count));
            for _ in 0..count {
                let x = i32::from(r.i16()?);
                let y = i32::from(r.i16()?);
                let w = usize::from(r.u16()?);
                let h = usize::from(r.u16()?);
                let len = r.u32()? as usize;
                let data = r.bytes(len)?;
                let mut px = Vec::with_capacity(w * h);
                for pair in data.chunks_exact(2) {
                    let idx = if usize::from(pair[1]) < n { pair[1] } else { 0 };
                    px.extend(std::iter::repeat_n(idx, usize::from(pair[0])));
                }
                if px.len() != w * h {
                    return Err(format!("{name}: quadro com tamanho errado"));
                }
                frames.push(SpriteFrame { x, y, w, h, px });
            }
            let slot = anims
                .get_mut(id)
                .ok_or_else(|| format!("{name}: animação {id} desconhecida"))?;
            *slot = Some(Animation { ms, looped, frames });
        }
        if anims[0].is_none() {
            return Err(format!("{name}: sem animação parada"));
        }
        Ok(SpriteSet {
            name,
            colors,
            roles,
            anims,
        })
    }

    /// Animação pedida, ou a mais próxima que o conjunto tem.
    pub fn anim(&self, mut a: Anim) -> Option<&Animation> {
        loop {
            if let Some(found) = self.anims[a as usize].as_ref() {
                return Some(found);
            }
            if a == Anim::Idle {
                return None;
            }
            a = a.fallback();
        }
    }

    /// Verdadeiro se o conjunto tem a animação (sem substituta).
    pub fn has(&self, a: Anim) -> bool {
        self.anims[a as usize].is_some()
    }
}

/// O corpo de lutador embutido no build (o primeiro conjunto de `assets-src/sprites/`, em ordem
/// alfabética), ou `None` se não há nenhum ou se ele não pôde ser lido.
pub fn fighter_set() -> Option<&'static SpriteSet> {
    static SET: OnceLock<Option<SpriteSet>> = OnceLock::new();
    SET.get_or_init(|| {
        let (name, bytes) = SETS.first()?;
        SpriteSet::parse(name, bytes).ok()
    })
    .as_ref()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_sets_parse() {
        for (name, bytes) in SETS {
            let set = match SpriteSet::parse(name, bytes) {
                Ok(s) => s,
                Err(e) => panic!("{e}"),
            };
            assert!(set.colors.len() <= 64);
            for a in Anim::ALL {
                let anim = set.anim(a);
                assert!(anim.is_some(), "{name}: {a:?}");
                for f in anim.into_iter().flat_map(|a| &a.frames) {
                    assert_eq!(f.px.len(), f.w * f.h);
                    // Os pés ficam no chão; só efeitos (poeira, rastro do golpe) passam um pouco.
                    assert!(f.y + f.h as i32 <= 3, "{name}: {a:?} desce abaixo do chão");
                }
            }
        }
    }

    #[test]
    fn truncated_data_is_an_error() {
        assert!(SpriteSet::parse("x", b"GSP").is_err());
        assert!(SpriteSet::parse("x", b"GSP1\x02\x01\x02\x03\x00\x01").is_err());
    }

    #[test]
    fn frame_timing_loops_or_holds() {
        let frames = || {
            (0..3)
                .map(|_| SpriteFrame {
                    x: 0,
                    y: 0,
                    w: 0,
                    h: 0,
                    px: Vec::new(),
                })
                .collect()
        };
        let once = Animation {
            ms: 100,
            looped: false,
            frames: frames(),
        };
        let looping = Animation {
            ms: 100,
            looped: true,
            frames: frames(),
        };
        assert_eq!(once.frame_index(250), 2);
        assert_eq!(once.frame_index(10_000), 2);
        assert_eq!(looping.frame_index(350), 0);
        assert_eq!(once.duration_ms(), 300);
    }
}
