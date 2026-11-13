//! Cálculo de posição da faixa (portável, testável em qualquer sistema).
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

/// Onde a faixa deve ficar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Placement {
    /// Sobre a barra de tarefas.
    OnTaskbar(Rect),
    /// Janela flutuante (barra ausente, vertical ou superior).
    Floating(Rect),
    /// Oculta (barra recolhida pela ocultação automática).
    Hidden,
}

/// Altura lógica mínima visível da barra para a faixa aparecer.
pub const MIN_VISIBLE_HEIGHT: i32 = 24;
/// Deslocamento horizontal padrão (px lógicos) a partir da esquerda da barra: deixa livre o botão de
/// Widgets/clima do Windows 11.
pub const DEFAULT_LEFT_OFFSET: i32 = 160;
/// Fração máxima da largura da barra que a faixa pode ocupar (‰).
pub const MAX_WIDTH_PM: i32 = 400;

/// Converte pixels lógicos em físicos para um DPI.
pub fn to_physical(logical: i32, dpi: u32) -> i32 {
    (i64::from(logical) * i64::from(dpi.max(48)) / 96) as i32
}

/// Converte pixels físicos em lógicos para um DPI.
pub fn to_logical(physical: i32, dpi: u32) -> i32 {
    (i64::from(physical) * 96 / i64::from(dpi.max(48))) as i32
}

/// Escala inteira da arte para uma altura física (altura base 24).
pub fn art_scale(height: i32) -> i32 {
    (height / 24).max(1)
}

/// Calcula a posição da faixa.
///
/// * `strip_logical_width`: largura da faixa em pixels lógicos da arte (multiplicada pela escala
///   inteira da arte, não pelo DPI).
/// * `offset`: deslocamento horizontal salvo, em pixels lógicos de tela (DPI), a partir da esquerda
///   da barra; `None` usa o padrão.
/// * `floating_area`: área de trabalho do monitor principal e seu DPI, para o modo flutuante.
pub fn place_strip(
    taskbar: Option<&TaskbarInfo>,
    strip_logical_width: i32,
    offset: Option<i32>,
    floating_area: (Rect, u32),
) -> Placement {
    let Some(tb) = taskbar.filter(|t| t.edge == Edge::Bottom && t.rect.is_valid()) else {
        return floating(strip_logical_width, floating_area);
    };
    // Parte visível da barra dentro do monitor (na ocultação automática, a barra fica quase toda
    // abaixo da borda do monitor).
    let visible_top = tb.rect.top.max(tb.monitor.top);
    let visible_bottom = tb.rect.bottom.min(tb.monitor.bottom);
    let visible_h = visible_bottom - visible_top;
    if visible_h < to_physical(MIN_VISIBLE_HEIGHT, tb.dpi) {
        return Placement::Hidden;
    }
    let height = visible_h;
    let scale = art_scale(height);
    let max_w = tb.rect.width() * MAX_WIDTH_PM / 1000;
    let width = (strip_logical_width * scale).min(max_w).max(1);
    let left_limit = tb.rect.left;
    let right_limit = tb.tray.map_or(tb.rect.right, |t| t.left.min(tb.rect.right));
    let off = offset.unwrap_or(DEFAULT_LEFT_OFFSET);
    let x = (tb.rect.left + to_physical(off, tb.dpi))
        .clamp(left_limit, (right_limit - width).max(left_limit));
    Placement::OnTaskbar(Rect {
        left: x,
        top: visible_top,
        right: x + width,
        bottom: visible_top + height,
    })
}

fn floating(strip_logical_width: i32, (work, dpi): (Rect, u32)) -> Placement {
    let height = to_physical(40, dpi).max(24);
    let scale = art_scale(height);
    let width = strip_logical_width * scale;
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

/// Deslocamento lógico correspondente a uma posição física `x` da faixa (para salvar após arrastar).
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

    #[test]
    fn places_on_bottom_taskbar_with_integer_scale() {
        let t = tb(1032, 48, 96);
        let Placement::OnTaskbar(r) = place_strip(Some(&t), 252, None, area()) else {
            panic!("esperava barra");
        };
        assert_eq!(r.height(), 48);
        assert_eq!(r.width(), 252 * 2);
        assert_eq!(r.left, 160);
        assert_eq!(r.top, 1032);
    }

    #[test]
    fn never_overlaps_tray_area() {
        let t = tb(1032, 48, 96);
        let Placement::OnTaskbar(r) = place_strip(Some(&t), 252, Some(5000), area()) else {
            panic!("esperava barra");
        };
        assert!(r.right <= 1700);
    }

    #[test]
    fn hidden_when_autohide_collapsed() {
        let mut t = tb(1078, 48, 96);
        t.autohide = true;
        assert_eq!(place_strip(Some(&t), 252, None, area()), Placement::Hidden);
    }

    #[test]
    fn vertical_or_missing_taskbar_floats() {
        let mut t = tb(1032, 48, 96);
        t.edge = Edge::Left;
        assert!(matches!(
            place_strip(Some(&t), 252, None, area()),
            Placement::Floating(_)
        ));
        assert!(matches!(
            place_strip(None, 252, None, area()),
            Placement::Floating(_)
        ));
    }

    #[test]
    fn negative_coordinates_are_supported() {
        let mut t = tb(1032, 48, 144);
        t.rect.left -= 1920;
        t.rect.right -= 1920;
        t.monitor.left -= 1920;
        t.monitor.right -= 1920;
        t.tray = None;
        let Placement::OnTaskbar(r) = place_strip(Some(&t), 252, Some(10), area()) else {
            panic!("esperava barra");
        };
        assert!(r.left >= -1920 && r.right <= 0);
    }

    #[test]
    fn width_is_capped_on_small_taskbars() {
        let mut t = tb(1032, 72, 96);
        t.rect.right = 800;
        t.tray = None;
        let Placement::OnTaskbar(r) = place_strip(Some(&t), 252, None, area()) else {
            panic!("esperava barra");
        };
        assert!(r.width() <= 800 * MAX_WIDTH_PM / 1000);
    }

    #[test]
    fn logical_physical_round_trip() {
        assert_eq!(to_physical(100, 144), 150);
        assert_eq!(to_logical(150, 144), 100);
    }
}
