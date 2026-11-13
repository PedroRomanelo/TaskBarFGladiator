//! Eventos entregues ao aplicativo (portável).

/// Janela de origem.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowId {
    /// Faixa na barra de tarefas.
    Strip,
    /// Painel expandido.
    Panel,
}

/// Botão do mouse.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    /// Esquerdo.
    Left,
    /// Direito.
    Right,
}

/// Evento de sistema ou de entrada.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Event {
    /// Botão pressionado (coordenadas no cliente da janela, pixels físicos).
    MouseDown {
        /// Janela.
        window: WindowId,
        /// Botão.
        button: MouseButton,
        /// X.
        x: i32,
        /// Y.
        y: i32,
    },
    /// Botão solto.
    MouseUp {
        /// Janela.
        window: WindowId,
        /// Botão.
        button: MouseButton,
        /// X.
        x: i32,
        /// Y.
        y: i32,
    },
    /// Movimento do mouse.
    MouseMove {
        /// Janela.
        window: WindowId,
        /// X.
        x: i32,
        /// Y.
        y: i32,
    },
    /// Tecla pressionada no painel (código de tecla virtual).
    Key(u32),
    /// Temporizador.
    Timer(usize),
    /// A barra de tarefas mudou de posição ou tamanho.
    TaskbarMoved,
    /// O Explorer reiniciou (barra e ícone da bandeja recriados).
    ExplorerRestarted,
    /// Janela em primeiro plano mudou ou se moveu (verificar tela cheia e z-order).
    ForegroundChanged,
    /// Monitores, resolução ou DPI mudaram.
    DisplayChanged,
    /// Tema claro/escuro ou alto contraste mudou.
    ThemeChanged,
    /// Sessão bloqueada ou desconectada.
    SessionLocked,
    /// Sessão desbloqueada ou reconectada.
    SessionUnlocked,
    /// O sistema vai suspender.
    Suspending,
    /// O sistema retomou da suspensão.
    Resumed,
    /// O Windows vai encerrar a sessão: salvar de forma síncrona agora.
    EndingSession,
    /// Clique esquerdo no ícone da bandeja.
    TrayClick,
    /// Pedido de menu de contexto no ícone da bandeja.
    TrayMenu,
    /// O painel perdeu o foco.
    PanelDeactivated,
    /// Outra instância pediu para mostrar esta.
    ShowRequested,
    /// Pedido de fechamento.
    CloseRequested,
}
