//! Fila de eventos da thread de interface.
//!
//! O procedimento de janela e os callbacks de WinEvent só chamam [`push`]. O empréstimo da fila
//! dura apenas a inserção/remoção: nenhuma função Win32 é chamada com a fila emprestada, então a
//! reentrância do procedimento de janela nunca encontra a fila ocupada.

use crate::event::Event;
use std::cell::RefCell;
use std::collections::VecDeque;

/// Limite da fila (proteção contra rajadas de eventos de localização).
const MAX_EVENTS: usize = 1024;

thread_local! {
    static QUEUE: RefCell<VecDeque<Event>> = RefCell::new(VecDeque::with_capacity(64));
}

/// Enfileira um evento. Eventos repetidos consecutivos de mudança de posição são agrupados.
pub fn push(event: Event) {
    QUEUE.with(|q| {
        if let Ok(mut q) = q.try_borrow_mut() {
            let coalesce = matches!(
                event,
                Event::TaskbarMoved | Event::ForegroundChanged | Event::MouseMove { .. }
            ) && q.back() == Some(&event);
            if coalesce {
                return;
            }
            if let (Event::MouseMove { .. }, Some(Event::MouseMove { .. })) = (event, q.back()) {
                q.pop_back();
            }
            if q.len() < MAX_EVENTS {
                q.push_back(event);
            }
        }
    });
}

/// Retira o próximo evento.
pub fn pop() -> Option<Event> {
    QUEUE.with(|q| q.try_borrow_mut().ok().and_then(|mut q| q.pop_front()))
}
