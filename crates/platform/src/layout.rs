//! Cálculo de posição do palco, acima da barra de tarefas (portável, testável em qualquer sistema).
//!
//! Ver `docs/03-fase-1-plataforma.md`, seções 1.5, 1.6 e 1.10.

/// Retângulo em pixels físicos (coordenadas de tela; podem ser negativas).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Rect {
    /// Esquerda.
    pub left: i32,
    /// Topo.
    pub top: i32,
    /// Direita (exclusiva).
    pub right: i32,
    /// Base (exclusiva).
    pub bottom: i32,
}

impl Rect {
    /// Largura.
    pub fn width(&self) -> i32 {
        self.right - self.left
    }

    /// Altura.
    pub fn height(&self) -> i32 {
        self.bottom - self.top
    }

    /// Verdadeiro se tem área positiva.
    pub fn is_valid(&self) -> bool {
        self.width() > 0 && self.height() > 0
    }

    /// Verdadeiro se `other` está totalmente dentro deste retângulo.
    pub fn contains_rect(&self, other: &Rect) -> bool {
        other.left >= self.left
            && other.top >= self.top
            && other.right <= self.right
            && other.bottom <= self.bottom
    }
}

/// Borda da tela em que a barra de tarefas está.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Edge {
    /// Inferior.
    Bottom,
    /// Superior.
    Top,
    /// Esquerda.
    Left,
    /// Direita.
    Right,
}

/// Informações da barra de tarefas.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskbarInfo {
    /// Retângulo atual da barra (inclui a parte escondida quando em ocultação automática).
    pub rect: Rect,
    /// Borda.
    pub edge: Edge,
    /// Ocultação automática ligada.
    pub autohide: bool,
    /// Retângulo do monitor da barra.
    pub monitor: Rect,
    /// DPI efetivo do monitor (96 = 100 %).
    pub dpi: u32,
    /// Nome do dispositivo do monitor (chave do deslocamento salvo).
    pub device: String,
    /// Retângulo da área de notificação, quando encontrado.
    pub tray: Option<Rect>,
}

/// Onde o palco deve ficar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Placement {
    /// Apoiado em cima da barra de tarefas (a base do palco encosta no topo da barra).
    AboveTaskbar(Rect),
    /// Janela flutuante (barra ausente, vertical ou superior).
    Floating(Rect),
    /// Oculto (não há espaço para o palco na tela).
    Hidden,
}

/// Altura da barra de tarefas (em pixels físicos) por unidade de escala da arte. O lutador tem 32
/// pixels lógicos de altura: com este divisor ele fica com cerca de 2,3 vezes a altura da barra.
pub const BAR_PER_SCALE: i32 = 14;
/// Margem padrão (px lógicos de tela) entre o palco e a borda direita da barra, quando não há
/// posição salva.
pub const DEFAULT_RIGHT_MARGIN: i32 = 8;
/// Altura lógica de barra de tarefas assumida no modo flutuante.
pub const FLOATING_BAR_HEIGHT: i32 = 48;

/// Converte pixels lógicos em físicos para um DPI.
pub fn to_physical(logical: i32, dpi: u32) -> i32 {
    (i64::from(logical) * i64::from(dpi.max(48)) / 96) as i32
}

/// Converte pixels físicos em lógicos para um DPI.
pub fn to_logical(physical: i32, dpi: u32) -> i32 {
    (i64::from(physical) * 96 / i64::from(dpi.max(48))) as i32
}

/// Escala inteira da arte para uma altura física de barra de tarefas, arredondando para a mais
/// próxima. `adjust` é o ajuste de tamanho escolhido pelo usuário (−1, 0 ou +1).
pub fn stage_scale(taskbar_height: i32, adjust: i32) -> i32 {
    ((taskbar_height * 2 + BAR_PER_SCALE) / (BAR_PER_SCALE * 2) + adjust).max(1)
}

/// Calcula a posição do palco.
///
/// * `stage`: largura e altura do palco em pixels lógicos da arte (multiplicadas pela escala
///   inteira da arte, não pelo DPI).
/// * `offset`: deslocamento horizontal salvo, em pixels lógicos de tela (DPI), a partir da esquerda
///   da barra; `None` usa o padrão (canto direito, acima da área de notificação).
/// * `floating_area`: área de trabalho do monitor principal e seu DPI, para o modo flutuante.
/// * `adjust`: ajuste de tamanho escolhido pelo usuário (−1, 0 ou +1 na escala).
pub fn place_stage(
    taskbar: Option<&TaskbarInfo>,
    stage: (i32, i32),
    offset: Option<i32>,
    floating_area: (Rect, u32),
    adjust: i32,
) -> Placement {
    let Some(tb) = taskbar.filter(|t| t.edge == Edge::Bottom && t.rect.is_valid()) else {
        return floating(stage, floating_area, adjust);
    };
    // Base do palco: o topo da parte visível da barra. Com a ocultação automática a barra fica
    // quase toda abaixo da borda do monitor; o palco então desce junto e fica na base da tela.
    let bottom = tb.rect.top.clamp(tb.monitor.top, tb.monitor.bottom);
    let room_w = tb.rect.width().min(tb.monitor.width());
    let room_h = bottom - tb.monitor.top;
    // A escala vem da altura total da barra (não só da parte visível) e diminui se não couber.
    let mut scale = stage_scale(tb.rect.height(), adjust);
    while scale > 1 && (stage.0 * scale > room_w || stage.1 * scale > room_h) {
        scale -= 1;
    }
    let (width, height) = (stage.0 * scale, stage.1 * scale);
    if width > room_w || height > room_h {
        return Placement::Hidden;
    }
    let max_x = (tb.rect.right - width).max(tb.rect.left);
    let x = match offset {
        Some(off) => tb.rect.left + to_physical(off, tb.dpi),
        None => max_x - to_physical(DEFAULT_RIGHT_MARGIN, tb.dpi),
    }
    .clamp(tb.rect.left, max_x);
    Placement::AboveTaskbar(Rect {
        left: x,
        top: bottom - height,
        right: x + width,
        bottom,
    })
}

fn floating(stage: (i32, i32), (work, dpi): (Rect, u32), adjust: i32) -> Placement {
    let mut scale = stage_scale(to_physical(FLOATING_BAR_HEIGHT, dpi), adjust);
    while scale > 1 && (stage.0 * scale > work.width() || stage.1 * scale > work.height()) {
        scale -= 1;
    }
    let (width, height) = (stage.0 * scale, stage.1 * scale);
    let margin = to_physical(8, dpi);
    let right = work.right - margin;
    let bottom = work.bottom - margin;
    Placement::Floating(Rect {
        left: right - width,
        top: bottom - height,
        right,
        bottom,
    })
}

/// Deslocamento lógico correspondente a uma posição física `x` do palco (para salvar após arrastar).
pub fn offset_from_x(tb: &TaskbarInfo, x: i32) -> i32 {
    to_logical(x - tb.rect.left, tb.dpi).max(0)
}

/// Chave de configuração do deslocamento por monitor.
pub fn offset_key(tb: &TaskbarInfo) -> String {
    format!(
        "{}@{}x{}",
        tb.device,
        tb.monitor.width(),
        tb.monitor.height()
    )
}

/// Verdadeiro se `window` cobre todo o `monitor` (heurística de tela cheia sem borda).
pub fn covers_monitor(window: &Rect, monitor: &Rect) -> bool {
    window.contains_rect(monitor)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Palco de teste: 132 × 58 pixels lógicos.
    const STAGE: (i32, i32) = (132, 58);

    fn tb(top: i32, h: i32, dpi: u32) -> TaskbarInfo {
        TaskbarInfo {
            rect: Rect {
                left: 0,
                top,
                right: 1920,
                bottom: top + h,
            },
            edge: Edge::Bottom,
            autohide: false,
            monitor: Rect {
                left: 0,
                top: 0,
                right: 1920,
                bottom: 1080,
            },
            dpi,
            device: "\\\\.\\DISPLAY1".into(),
            tray: Some(Rect {
                left: 1700,
                top,
                right: 1920,
                bottom: top + h,
            }),
        }
    }

    fn area() -> (Rect, u32) {
        (
            Rect {
                left: 0,
                top: 0,
                right: 1920,
                bottom: 1032,
            },
            96,
        )
    }

    fn above(p: Placement) -> Rect {
        match p {
            Placement::AboveTaskbar(r) => r,
            other => panic!("esperava palco acima da barra, veio {other:?}"),
        }
    }

    #[test]
    fn scale_follows_taskbar_height() {
        assert_eq!(stage_scale(40, 0), 3);
        assert_eq!(stage_scale(48, 0), 3);
        assert_eq!(stage_scale(60, 0), 4);
        assert_eq!(stage_scale(72, 0), 5);
        assert_eq!(stage_scale(48, 1), 4);
        assert_eq!(stage_scale(48, -1), 2);
        assert_eq!(stage_scale(10, -1), 1);
    }

    #[test]
    fn sits_on_top_of_the_bottom_taskbar() {
        let t = tb(1032, 48, 96);
        let r = above(place_stage(Some(&t), STAGE, None, area(), 0));
        // A base encosta no topo da barra; nada do palco fica sobre ela.
        assert_eq!(r.bottom, 1032);
        assert_eq!(r.height(), 58 * 3);
        assert_eq!(r.width(), 132 * 3);
        // Sem posição salva: canto direito, com a margem padrão.
        assert_eq!(r.right, 1920 - DEFAULT_RIGHT_MARGIN);
    }

    #[test]
    fn saved_offset_is_measured_from_the_left_and_clamped() {
        let t = tb(1032, 48, 96);
        let r = above(place_stage(Some(&t), STAGE, Some(100), area(), 0));
        assert_eq!(r.left, 100);
        let r = above(place_stage(Some(&t), STAGE, Some(5000), area(), 0));
        assert_eq!(r.right, 1920);
    }

    #[test]
    fn follows_the_taskbar_when_autohide_collapses_it() {
        let mut t = tb(1078, 48, 96);
        t.autohide = true;
        let r = above(place_stage(Some(&t), STAGE, None, area(), 0));
        assert_eq!(r.bottom, 1078);
        // A escala continua a da barra inteira, não a da fresta visível.
        assert_eq!(r.height(), 58 * 3);
    }

    #[test]
    fn vertical_or_missing_taskbar_floats() {
        let mut t = tb(1032, 48, 96);
        t.edge = Edge::Left;
        assert!(matches!(
            place_stage(Some(&t), STAGE, None, area(), 0),
            Placement::Floating(_)
        ));
        let Placement::Floating(r) = place_stage(None, STAGE, None, area(), 0) else {
            panic!("esperava flutuante");
        };
        assert_eq!((r.width(), r.height()), (132 * 3, 58 * 3));
        assert!(r.right <= 1920 && r.bottom <= 1032);
    }

    #[test]
    fn negative_coordinates_are_supported() {
        let mut t = tb(1032, 48, 144);
        t.rect.left -= 1920;
        t.rect.right -= 1920;
        t.monitor.left -= 1920;
        t.monitor.right -= 1920;
        t.tray = None;
        let r = above(place_stage(Some(&t), STAGE, Some(10), area(), 0));
        assert!(r.left >= -1920 && r.right <= 0);
    }

    #[test]
    fn scale_shrinks_to_fit_small_screens() {
        let mut t = tb(552, 48, 96);
        t.rect.right = 300;
        t.monitor.right = 300;
        t.monitor.bottom = 600;
        t.tray = None;
        let r = above(place_stage(Some(&t), STAGE, None, area(), 1));
        assert_eq!(r.width(), 132 * 2);
        // Sem espaço nem na menor escala: oculto.
        t.rect.right = 100;
        t.monitor.right = 100;
        assert_eq!(
            place_stage(Some(&t), STAGE, None, area(), 0),
            Placement::Hidden
        );
    }

    #[test]
    fn logical_physical_round_trip() {
        assert_eq!(to_physical(100, 144), 150);
        assert_eq!(to_logical(150, 144), 100);
    }
}
