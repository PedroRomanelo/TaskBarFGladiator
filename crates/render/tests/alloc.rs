//! Desenhar a faixa não pode alocar memória depois do primeiro quadro (docs/08, seção 2.2).
//!
//! Fica em um arquivo próprio porque o contador de alocações é global ao processo: outros testes
//! rodando em paralelo seriam contados.

use assets::Frame;
use render::scene::STRIP_WIDTH;
use render::{ButtonState, FighterView, StripView, Surface, Theme, draw_strip};
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

/// Alocador que conta alocações (para o teste de zero alocação).
struct Counting;

static ALLOCS: AtomicUsize = AtomicUsize::new(0);

// SAFETY: delega integralmente ao alocador do sistema; só incrementa um contador atômico.
unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOCS.fetch_add(1, Ordering::Relaxed);
        // SAFETY: mesmo contrato de `GlobalAlloc::alloc`, repassado ao sistema.
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // SAFETY: `ptr` veio de `System.alloc` com o mesmo `layout`.
        unsafe { System.dealloc(ptr, layout) }
    }
}

#[global_allocator]
static GLOBAL: Counting = Counting;

fn fighter(pos: i32, frame: Frame, hp: i32, flash: bool) -> FighterView {
    FighterView {
        name: "TESTE".into(),
        level: 5,
        hp,
        hp_max: 80,
        stamina: 30,
        stamina_max: 60,
        pos_pm: pos * 1000,
        frame,
        flash,
    }
}

fn scenes() -> Vec<(&'static str, StripView)> {
    vec![
        (
            "luta_manual",
            StripView::Fight {
                player: fighter(3, Frame::Attack, 50, false),
                opponent: fighter(4, Frame::Defend, 20, true),
                cells: 10,
                buttons: Some([ButtonState::Enabled; 8]),
                paused: false,
                banner: Some("CRÍTICO!".into()),
            },
        ),
        (
            "luta_auto",
            StripView::Fight {
                player: fighter(2, Frame::Idle, 80, false),
                opponent: fighter(7, Frame::Down, 0, false),
                cells: 10,
                buttons: None,
                paused: true,
                banner: None,
            },
        ),
        (
            "hub",
            StripView::Hub {
                name: "Ação Ç".into(),
                level: 12,
                gold: 98765,
                unspent: 3,
                next_label: "PRÓXIMO: TESTE".into(),
                fight_label: "LUTAR".into(),
            },
        ),
        (
            "resultado",
            StripView::Result {
                title: "DERROTA".into(),
                detail: "+10 XP +5 OURO".into(),
                won: false,
                ok_label: "OK".into(),
            },
        ),
    ]
}

#[test]
fn strip_drawing_does_not_allocate_after_first_frame() {
    let views = scenes();
    let mut s = Surface::new((STRIP_WIDTH * 2) as usize, 48);
    let mut hits = Vec::with_capacity(16);
    let theme = Theme {
        light: false,
        high_contrast: false,
    };
    // Primeiro quadro: aquece capacidades.
    for (_, v) in &views {
        draw_strip(&mut s, v, theme, &mut hits);
    }
    let before = ALLOCS.load(Ordering::Relaxed);
    for _ in 0..250 {
        for (_, v) in &views {
            draw_strip(&mut s, v, theme, &mut hits);
        }
    }
    let after = ALLOCS.load(Ordering::Relaxed);
    assert_eq!(after - before, 0, "alocações durante o desenho");
}
