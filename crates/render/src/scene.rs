//! Cenas: a faixa da barra de tarefas e o painel expandido.
//!
//! As cenas recebem "visões" (dados prontos para exibir, sem depender do crate `game`) e devolvem as
//! regiões clicáveis. Todas as medidas abaixo são em pixels lógicos (altura base 24); a escala
//! inteira é aplicada no desenho.

use crate::draw::{self, SpritePalette, TeamColors};
use crate::surface::{Rgba, Surface};
use assets::{Frame, Icon};

/// Altura lógica da faixa.
pub const STRIP_BASE_HEIGHT: i32 = 24;
/// Largura lógica da faixa.
pub const STRIP_WIDTH: i32 = 252;
/// Largura lógica do painel.
pub const PANEL_WIDTH: i32 = 200;
/// Altura lógica do painel.
pub const PANEL_HEIGHT: i32 = 150;

/// Identificador de uma região clicável.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HitId {
    /// Botão de ação de combate (índice na ordem de [`ACTION_ICONS`]).
    Action(u8),
    /// Abre o painel.
    OpenPanel,
    /// Pausa/continua o modo automático.
    TogglePause,
    /// Inicia a próxima luta.
    Fight,
    /// Fecha a tela de resultado.
    Continue,
    /// Aba do painel.
    Tab(u8),
    /// Botão de uma linha do painel.
    Row(u16),
    /// Fecha o painel.
    ClosePanel,
}

/// Região clicável em pixels físicos.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Hit {
    /// X.
    pub x: i32,
    /// Y.
    pub y: i32,
    /// Largura.
    pub w: i32,
    /// Altura.
    pub h: i32,
    /// Identificador.
    pub id: HitId,
}

impl Hit {
    /// Verdadeiro se o ponto está dentro.
    pub fn contains(&self, x: i32, y: i32) -> bool {
        x >= self.x && y >= self.y && x < self.x + self.w && y < self.y + self.h
    }
}

/// Primeira região que contém o ponto.
pub fn hit_test(hits: &[Hit], x: i32, y: i32) -> Option<HitId> {
    hits.iter().find(|h| h.contains(x, y)).map(|h| h.id)
}

/// Ícones dos botões de ação, na ordem das ações do jogo.
pub const ACTION_ICONS: [Icon; 8] = [
    Icon::Advance,
    Icon::Retreat,
    Icon::Light,
    Icon::Medium,
    Icon::Heavy,
    Icon::Defend,
    Icon::Recover,
    Icon::Taunt,
];

/// Tema visual.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Theme {
    /// Barra de tarefas clara (texto escuro com contorno claro).
    pub light: bool,
    /// Alto contraste (cores sólidas preto/branco).
    pub high_contrast: bool,
}

impl Theme {
    fn fg(&self) -> Rgba {
        if self.light && !self.high_contrast {
            Rgba::rgb(0x101010)
        } else {
            Rgba::rgb(0xffffff)
        }
    }

    fn outline(&self) -> Rgba {
        if self.light && !self.high_contrast {
            Rgba::rgb(0xffffff)
        } else {
            Rgba::rgb(0x000000)
        }
    }

    fn gold(&self) -> Rgba {
        match (self.high_contrast, self.light) {
            (true, _) => self.fg(),
            (false, true) => Rgba::rgb(0x8a5a00),
            (false, false) => Rgba::rgb(0xffd23c),
        }
    }

    fn good(&self) -> Rgba {
        match (self.high_contrast, self.light) {
            (true, _) => self.fg(),
            (false, true) => Rgba::rgb(0x1f7a1f),
            (false, false) => Rgba::rgb(0x6fe06f),
        }
    }

    fn bad(&self) -> Rgba {
        match (self.high_contrast, self.light) {
            (true, _) => self.fg(),
            (false, true) => Rgba::rgb(0xa02020),
            (false, false) => Rgba::rgb(0xe06f6f),
        }
    }

    fn palette(&self, team: Team) -> SpritePalette {
        let team_colors = if self.high_contrast {
            match team {
                Team::Player => TeamColors {
                    main: Rgba::rgb(0xffffff),
                    shade: Rgba::rgb(0xc0c0c0),
                },
                Team::Opponent => TeamColors {
                    main: Rgba::rgb(0x808080),
                    shade: Rgba::rgb(0x505050),
                },
            }
        } else {
            match team {
                Team::Player => TeamColors {
                    main: Rgba::rgb(0xb83a2e),
                    shade: Rgba::rgb(0x7c2219),
                },
                Team::Opponent => TeamColors {
                    main: Rgba::rgb(0x3a5cb8),
                    shade: Rgba::rgb(0x223a7c),
                },
            }
        };
        SpritePalette {
            outline: Rgba::rgb(0x161210),
            skin: Rgba::rgb(0xd9a273),
            skin_shade: Rgba::rgb(0xa8704a),
            metal: Rgba::rgb(0xc4c6cc),
            metal_shade: Rgba::rgb(0x74767e),
            leather: Rgba::rgb(0x8c5a2b),
            belt: Rgba::rgb(0x55361a),
            team: team_colors,
        }
    }
}

/// Lado.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Team {
    /// Jogador (esquerda).
    Player,
    /// Oponente (direita).
    Opponent,
}

/// Dados de um lutador para exibição.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FighterView {
    /// Nome curto.
    pub name: String,
    /// Nível.
    pub level: u8,
    /// HP.
    pub hp: i32,
    /// HP máximo.
    pub hp_max: i32,
    /// Fôlego.
    pub stamina: i32,
    /// Fôlego máximo atual.
    pub stamina_max: i32,
    /// Casa na arena (pode ser fracionária durante a animação: ‰ de casa).
    pub pos_pm: i32,
    /// Quadro.
    pub frame: Frame,
    /// Pisca em branco (golpe recebido).
    pub flash: bool,
}

/// Estado de um botão.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonState {
    /// Clicável.
    Enabled,
    /// Desabilitado.
    Disabled,
}

/// Conteúdo da faixa.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StripView {
    /// Luta em andamento.
    Fight {
        /// Jogador.
        player: FighterView,
        /// Oponente.
        opponent: FighterView,
        /// Número de casas da arena.
        cells: u8,
        /// Botões de ação (modo manual) ou `None` (modo automático).
        buttons: Option<[ButtonState; 8]>,
        /// Modo automático pausado.
        paused: bool,
        /// Texto curto opcional sobre a arena (ex.: "CRÍTICO!").
        banner: Option<String>,
    },
    /// Fora de luta.
    Hub {
        /// Nome do jogador.
        name: String,
        /// Nível.
        level: u8,
        /// Ouro.
        gold: u32,
        /// Pontos de atributo livres.
        unspent: u16,
        /// Rótulo do próximo desafio (já traduzido).
        next_label: String,
        /// Rótulo do botão de luta (já traduzido).
        fight_label: String,
    },
    /// Resultado de luta.
    Result {
        /// Título (já traduzido, ex.: "VITÓRIA!").
        title: String,
        /// Detalhe (ex.: "+45 XP  +30 OURO").
        detail: String,
        /// Vitória (cor do título).
        won: bool,
        /// Rótulo do botão de continuar.
        ok_label: String,
    },
}

/// Escala inteira e deslocamento vertical para uma altura física de faixa.
pub fn strip_scale(height_px: i32) -> (i32, i32) {
    let s = (height_px / STRIP_BASE_HEIGHT).max(1);
    let oy = ((height_px - STRIP_BASE_HEIGHT * s) / 2).max(0);
    (s, oy)
}

/// Desenha a faixa inteira. `hits` é limpo e preenchido com as regiões clicáveis.
pub fn draw_strip(s: &mut Surface, view: &StripView, theme: Theme, hits: &mut Vec<Hit>) {
    hits.clear();
    s.clear(Rgba::TRANSPARENT);
    let (sc, oy) = strip_scale(s.height() as i32);
    let ctx = Ctx {
        sc,
        ox: 0,
        oy,
        theme,
    };
    match view {
        StripView::Fight {
            player,
            opponent,
            cells,
            buttons,
            paused,
            banner,
        } => {
            ctx.fighter_panel(s, 1, player, Team::Player);
            let arena_x = 30;
            let cell_w = 10;
            let arena_w = i32::from(*cells) * cell_w;
            ctx.arena(s, arena_x, arena_w, *cells, cell_w, player, opponent);
            ctx.fighter_panel(s, arena_x + arena_w + 2, opponent, Team::Opponent);
            if let Some(b) = banner {
                let w = draw::text_width(b);
                let x = arena_x + (arena_w - w) / 2;
                ctx.text(s, x, 1, b, ctx.theme.gold());
            }
            let bx = arena_x + arena_w + 32;
            match buttons {
                Some(states) => {
                    for (i, (icon, st)) in ACTION_ICONS.iter().zip(states.iter()).enumerate() {
                        let x = bx + i as i32 * 10;
                        let enabled = *st == ButtonState::Enabled;
                        ctx.button(s, x, 7, *icon, enabled, HitId::Action(i as u8), hits);
                    }
                    ctx.button(s, bx + 80, 7, Icon::Menu, true, HitId::OpenPanel, hits);
                }
                None => {
                    let icon = if *paused { Icon::Play } else { Icon::Pause };
                    ctx.button(s, bx, 7, icon, true, HitId::TogglePause, hits);
                    ctx.text(s, bx + 12, 9, "AUTO", ctx.theme.fg());
                    ctx.button(s, bx + 80, 7, Icon::Menu, true, HitId::OpenPanel, hits);
                }
            }
        }
        StripView::Hub {
            name,
            level,
            gold,
            unspent,
            next_label,
            fight_label,
        } => {
            ctx.button(s, 1, 7, Icon::Menu, true, HitId::OpenPanel, hits);
            let fg = ctx.theme.fg();
            let mut buf = NumBuf::new();
            ctx.text_max(s, 13, 3, name, 12, fg);
            let x = 13 + ctx.text(s, 13, 13, "NV ", fg) / sc;
            ctx.text(s, x, 13, buf.fmt(u32::from(*level)), fg);
            ctx.text(s, 70, 3, buf.fmt(*gold), ctx.theme.gold());
            if *unspent > 0 {
                let w = ctx.text(s, 70, 13, "+", ctx.theme.good()) / sc;
                ctx.text(
                    s,
                    70 + w + 1,
                    13,
                    buf.fmt(u32::from(*unspent)),
                    ctx.theme.good(),
                );
            }
            ctx.text_max(s, 108, 8, next_label, 26, fg);
            let bw = draw::text_width(fight_label) + 6;
            ctx.label_button(
                s,
                STRIP_WIDTH - bw - 2,
                5,
                bw,
                13,
                fight_label,
                HitId::Fight,
                hits,
            );
        }
        StripView::Result {
            title,
            detail,
            won,
            ok_label,
        } => {
            let color = if *won {
                ctx.theme.good()
            } else {
                ctx.theme.bad()
            };
            ctx.text_max(s, 4, 3, title, 20, color);
            ctx.text_max(s, 4, 13, detail, 40, ctx.theme.fg());
            let bw = draw::text_width(ok_label) + 6;
            ctx.label_button(
                s,
                STRIP_WIDTH - bw - 2,
                5,
                bw,
                13,
                ok_label,
                HitId::Continue,
                hits,
            );
        }
    }
}

/// Formata números sem alocar (buffer na pilha).
struct NumBuf {
    buf: [u8; 12],
    len: usize,
}

impl NumBuf {
    fn new() -> Self {
        Self {
            buf: [0; 12],
            len: 0,
        }
    }

    fn fmt(&mut self, mut v: u32) -> &str {
        let mut tmp = [0u8; 12];
        let mut n = 0;
        loop {
            tmp[n] = b'0' + (v % 10) as u8;
            n += 1;
            v /= 10;
            if v == 0 || n == tmp.len() {
                break;
            }
        }
        for i in 0..n {
            self.buf[i] = tmp[n - 1 - i];
        }
        self.len = n;
        std::str::from_utf8(&self.buf[..self.len]).unwrap_or("")
    }
}

/// Contexto de desenho: escala, deslocamento e tema.
struct Ctx {
    sc: i32,
    ox: i32,
    oy: i32,
    theme: Theme,
}

impl Ctx {
    fn px(&self, x: i32) -> i32 {
        self.ox + x * self.sc
    }

    fn py(&self, y: i32) -> i32 {
        self.oy + y * self.sc
    }

    fn rect(&self, s: &mut Surface, x: i32, y: i32, w: i32, h: i32, c: Rgba) {
        s.fill_rect(self.px(x), self.py(y), w * self.sc, h * self.sc, c);
    }

    fn text(&self, s: &mut Surface, x: i32, y: i32, t: &str, c: Rgba) -> i32 {
        draw::text(
            s,
            self.px(x),
            self.py(y),
            self.sc,
            t,
            c,
            Some(self.theme.outline()),
        )
    }

    fn text_max(&self, s: &mut Surface, x: i32, y: i32, t: &str, max: usize, c: Rgba) -> i32 {
        draw::text_max(
            s,
            self.px(x),
            self.py(y),
            self.sc,
            t,
            max,
            c,
            Some(self.theme.outline()),
        )
    }

    fn fighter_panel(&self, s: &mut Surface, x: i32, f: &FighterView, team: Team) {
        let fg = self.theme.fg();
        self.text_max(s, x, 1, &f.name, 6, fg);
        // HP: barra cheia (posição superior). Fôlego: segmentos (posição inferior). As duas se
        // distinguem pela forma e posição, não só pela cor.
        let bw = 26;
        self.rect(s, x, 9, bw, 4, Rgba::rgb(0x000000).with_alpha(200));
        let hp_w = if f.hp_max > 0 {
            (i64::from(f.hp.max(0)) * i64::from(bw - 2) / i64::from(f.hp_max)) as i32
        } else {
            0
        };
        let hp_color = match team {
            Team::Player => Rgba::rgb(0xe8573c),
            Team::Opponent => Rgba::rgb(0xe8573c),
        };
        let hp_color = if self.theme.high_contrast {
            Rgba::rgb(0xffffff)
        } else {
            hp_color
        };
        self.rect(s, x + 1, 10, hp_w, 2, hp_color);
        let segs = 8;
        let filled = if f.stamina_max > 0 {
            (i64::from(f.stamina.max(0)) * i64::from(segs) / i64::from(f.stamina_max)) as i32
        } else {
            0
        };
        for i in 0..segs {
            let c = if i < filled {
                if self.theme.high_contrast {
                    Rgba::rgb(0xffffff)
                } else {
                    Rgba::rgb(0xf2c53d)
                }
            } else {
                Rgba::rgb(0x000000).with_alpha(160)
            };
            self.rect(s, x + i * 3 + 1, 14, 2, 2, c);
        }
        let mut buf = NumBuf::new();
        self.text(s, x, 17, buf.fmt(f.hp.max(0) as u32), fg);
    }

    #[allow(clippy::too_many_arguments)]
    fn arena(
        &self,
        s: &mut Surface,
        x: i32,
        w: i32,
        cells: u8,
        cell_w: i32,
        p: &FighterView,
        o: &FighterView,
    ) {
        let line = self.theme.fg().with_alpha(110);
        self.rect(s, x, 22, w, 1, line);
        for c in 0..=i32::from(cells) {
            self.rect(s, x + c * cell_w, 21, 1, 1, line);
        }
        let (fw, _) = draw::fighter_size();
        for (f, team, mirror) in [(p, Team::Player, false), (o, Team::Opponent, true)] {
            let center = x * 1000 + f.pos_pm * cell_w + cell_w * 500;
            let left = center / 1000 - fw / 2;
            draw::fighter(
                s,
                self.px(left),
                self.py(2),
                self.sc,
                f.frame,
                &self.theme.palette(team),
                mirror,
                f.flash,
            );
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn button(
        &self,
        s: &mut Surface,
        x: i32,
        y: i32,
        icon: Icon,
        enabled: bool,
        id: HitId,
        hits: &mut Vec<Hit>,
    ) {
        let bg = if enabled {
            Rgba::rgb(0x2a2a33).with_alpha(230)
        } else {
            Rgba::rgb(0x2a2a33).with_alpha(110)
        };
        self.rect(s, x, y, 9, 9, bg);
        let fg = if enabled {
            Rgba::rgb(0xf0f0f0)
        } else {
            Rgba::rgb(0x8a8a8a)
        };
        draw::icon(s, self.px(x + 1), self.py(y + 1), self.sc, icon, fg);
        if enabled {
            hits.push(Hit {
                x: self.px(x),
                y: self.py(y),
                w: 9 * self.sc,
                h: 9 * self.sc,
                id,
            });
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn label_button(
        &self,
        s: &mut Surface,
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        label: &str,
        id: HitId,
        hits: &mut Vec<Hit>,
    ) {
        self.rect(s, x, y, w, h, Rgba::rgb(0x2a2a33).with_alpha(235));
        self.rect(s, x, y, w, 1, Rgba::rgb(0xffffff).with_alpha(60));
        draw::text(
            s,
            self.px(x + 3),
            self.py(y + (h - 6) / 2),
            self.sc,
            label,
            Rgba::rgb(0xf0f0f0),
            None,
        );
        hits.push(Hit {
            x: self.px(x),
            y: self.py(y),
            w: w * self.sc,
            h: h * self.sc,
            id,
        });
    }
}

/// Linha do painel.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PanelRow {
    /// Texto à esquerda.
    pub text: String,
    /// Texto à direita (antes do botão).
    pub value: String,
    /// Botão opcional: rótulo e se está habilitado.
    pub button: Option<(String, bool)>,
    /// Destaque (item equipado, próximo oponente).
    pub highlight: bool,
}

/// Conteúdo do painel.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PanelView {
    /// Rótulos das abas.
    pub tabs: Vec<String>,
    /// Aba ativa.
    pub active_tab: usize,
    /// Cabeçalho da aba (linha de resumo).
    pub header: String,
    /// Linhas.
    pub rows: Vec<PanelRow>,
    /// Rodapé (mensagem).
    pub footer: String,
}

/// Desenha o painel. A escala é `largura_física / PANEL_WIDTH`.
pub fn draw_panel(s: &mut Surface, view: &PanelView, hits: &mut Vec<Hit>) {
    hits.clear();
    let sc = (s.width() as i32 / PANEL_WIDTH).max(1);
    let ctx = Ctx {
        sc,
        ox: 0,
        oy: 0,
        theme: Theme {
            light: false,
            high_contrast: false,
        },
    };
    s.clear(Rgba::rgb(0x1c1c22));
    ctx.rect(s, 0, 0, PANEL_WIDTH, 1, Rgba::rgb(0x50505c));
    ctx.rect(s, 0, PANEL_HEIGHT - 1, PANEL_WIDTH, 1, Rgba::rgb(0x50505c));
    ctx.rect(s, 0, 0, 1, PANEL_HEIGHT, Rgba::rgb(0x50505c));
    ctx.rect(s, PANEL_WIDTH - 1, 0, 1, PANEL_HEIGHT, Rgba::rgb(0x50505c));

    // Abas.
    let mut x = 3;
    for (i, t) in view.tabs.iter().enumerate() {
        let w = draw::text_width(t) + 6;
        let active = i == view.active_tab;
        let bg = if active {
            Rgba::rgb(0x3a3a48)
        } else {
            Rgba::rgb(0x26262e)
        };
        ctx.rect(s, x, 3, w, 10, bg);
        draw::text(
            s,
            ctx.px(x + 3),
            ctx.py(5),
            sc,
            t,
            if active {
                Rgba::rgb(0xffd23c)
            } else {
                Rgba::rgb(0xc8c8c8)
            },
            None,
        );
        hits.push(Hit {
            x: ctx.px(x),
            y: ctx.py(3),
            w: w * sc,
            h: 10 * sc,
            id: HitId::Tab(i as u8),
        });
        x += w + 2;
    }
    // Fechar.
    ctx.rect(s, PANEL_WIDTH - 12, 3, 9, 9, Rgba::rgb(0x3a2a2a));
    draw::text(
        s,
        ctx.px(PANEL_WIDTH - 10),
        ctx.py(5),
        sc,
        "X",
        Rgba::rgb(0xf0f0f0),
        None,
    );
    hits.push(Hit {
        x: ctx.px(PANEL_WIDTH - 12),
        y: ctx.py(3),
        w: 9 * sc,
        h: 9 * sc,
        id: HitId::ClosePanel,
    });

    let fg = Rgba::rgb(0xe8e8e8);
    draw::text_max(
        s,
        ctx.px(4),
        ctx.py(17),
        sc,
        &view.header,
        48,
        Rgba::rgb(0xffd23c),
        None,
    );
    let top = 27;
    let row_h = 9;
    let max_rows = ((PANEL_HEIGHT - top - 12) / row_h) as usize;
    for (i, row) in view.rows.iter().take(max_rows).enumerate() {
        let y = top + i as i32 * row_h;
        if row.highlight {
            ctx.rect(s, 2, y - 1, PANEL_WIDTH - 4, row_h, Rgba::rgb(0x2c2c3a));
        }
        draw::text_max(s, ctx.px(4), ctx.py(y + 1), sc, &row.text, 24, fg, None);
        let mut right = PANEL_WIDTH - 4;
        if let Some((label, enabled)) = &row.button {
            let w = draw::text_width(label) + 6;
            right -= w;
            let bg = if *enabled {
                Rgba::rgb(0x2f5a2f)
            } else {
                Rgba::rgb(0x33333a)
            };
            ctx.rect(s, right, y - 1, w, row_h - 1, bg);
            draw::text(
                s,
                ctx.px(right + 3),
                ctx.py(y + 1),
                sc,
                label,
                if *enabled {
                    Rgba::rgb(0xffffff)
                } else {
                    Rgba::rgb(0x808080)
                },
                None,
            );
            if *enabled {
                hits.push(Hit {
                    x: ctx.px(right),
                    y: ctx.py(y - 1),
                    w: w * sc,
                    h: (row_h - 1) * sc,
                    id: HitId::Row(i as u16),
                });
            }
            right -= 3;
        }
        let vw = draw::text_width(&row.value);
        draw::text(
            s,
            ctx.px(right - vw),
            ctx.py(y + 1),
            sc,
            &row.value,
            Rgba::rgb(0xb8b8c8),
            None,
        );
    }
    draw::text_max(
        s,
        ctx.px(4),
        ctx.py(PANEL_HEIGHT - 10),
        sc,
        &view.footer,
        48,
        Rgba::rgb(0xa0a0b0),
        None,
    );
}
