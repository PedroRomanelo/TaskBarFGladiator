//! Desenhar o palco não pode alocar memória depois do primeiro quadro (docs/08, seção 2.2).
//!
//! Fica em um arquivo próprio porque o contador de alocações é global ao processo: outros testes
//! rodando em paralelo seriam contados.

mod common;

use render::scene::{STAGE_HEIGHT, STAGE_WIDTH};
use render::{Surface, draw_stage};
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

#[test]
fn stage_drawing_does_not_allocate_after_first_frame() {
    let views = common::scenes();
    let (theme, _) = common::themes()[0];
    let mut s = Surface::new((STAGE_WIDTH * 2) as usize, (STAGE_HEIGHT * 2) as usize);
    let mut hits = Vec::with_capacity(16);
    // Primeiro quadro: aquece capacidades.
    for (_, v, hover) in &views {
        draw_stage(&mut s, v, theme, *hover, &mut hits);
    }
    let before = ALLOCS.load(Ordering::Relaxed);
    for _ in 0..100 {
        for (_, v, hover) in &views {
            draw_stage(&mut s, v, theme, *hover, &mut hits);
        }
    }
    let after = ALLOCS.load(Ordering::Relaxed);
    assert_eq!(after - before, 0, "alocações durante o desenho");
}
