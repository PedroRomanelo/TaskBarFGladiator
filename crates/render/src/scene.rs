//! Cenas: o palco acima da barra de tarefas e o painel expandido.
//!
//! As cenas recebem "visões" (dados prontos para exibir, sem depender do crate `game`) e devolvem as
//! regiões clicáveis. Todas as medidas abaixo são em pixels lógicos da arte; a escala inteira é
//! aplicada no desenho.
//!
//! Organização do palco (de cima para baixo):
//!
//! ```text
//!  0.. 7  legenda da etapa (o que está acontecendo / o que fazer agora)
//!  8..47  arena: os dois lutadores e, no modo manual, os botões de ação em volta do jogador
//! 48      chão (areia, com uma marca por casa)
//! 49..57  faixa de status, encostada na barra de tarefas: vida e fôlego na luta; nível, ouro e o
//!         botão principal (LUTAR / CONTINUAR) fora dela
//! ```

use crate::draw::{self, SpritePalette, TeamColors};
use crate::surface::{Rgba, Surface};
use assets::{CANVAS, CENTER_X, Frame, Icon, Look};

/// Largura lógica do palco.
pub const STAGE_WIDTH: i32 = 132;
/// Altura lógica do palco.
pub const STAGE_HEIGHT: i32 = 58;
/// Largura lógica do painel.
pub const PANEL_WIDTH: i32 = 200;
/// Altura lógica do painel.
pub const PANEL_HEIGHT: i32 = 150;

/// Linha do topo da tela dos lutadores.
const FIGHTER_Y: i32 = 16;
/// Linha do chão.
const FLOOR_Y: i32 = 48;
/// Primeira linha da faixa de status.
const BAND_Y: i32 = 49;
/// Altura da faixa de status.
const BAND_H: i32 = 9;
/// Coluna onde começa a arena.
const ARENA_X: i32 = 15;
/// Largura de uma casa da arena.
const CELL_W: i32 = 10;
/// Diâmetro de um botão de ação.
const BUTTON_D: i32 = 9;
/// Distância vertical entre botões de ação.
const BUTTON_PITCH: i32 = 10;
/// Linha do primeiro botão de ação.
const BUTTONS_Y: i32 = 9;
/// Distância entre o eixo do lutador e a coluna de botões ao lado dele.
const BUTTON_GAP: i32 = 11;

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

/// Lado do jogador em que cada botão de ação aparece e sua linha na coluna, na ordem das ações:
/// para trás ficam recuar e as ações de preparo; para a frente, avançar e os golpes.
const ACTION_SLOTS: [(bool, i32); 8] = [
    (true, 0),
    (false, 0),
    (true, 1),
    (true, 2),
    (true, 3),
    (false, 1),
    (false, 2),
    (false, 3),
];

/// Tema visual.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Theme {
    /// Barra de tarefas clara (faixa de status clara com texto escuro).
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
        Rgba::rgb(0x000000)
    }

    /// Fundo da faixa de status (acompanha a cor da barra de tarefas).
    fn band(&self) -> Rgba {
        match (self.high_contrast, self.light) {
            (true, _) => Rgba::rgb(0x000000),
            (false, true) => Rgba::rgb(0xeeeef2).with_alpha(240),
            (false, false) => Rgba::rgb(0x1c1c22).with_alpha(240),
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

    fn team(&self, team: Team) -> TeamColors {
        if self.high_contrast {
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
                    main: Rgba::rgb(0xc8402f),
                    shade: Rgba::rgb(0x862419),
                },
                Team::Opponent => TeamColors {
                    main: Rgba::rgb(0x3a66c8),
                    shade: Rgba::rgb(0x223f86),
                },
            }
        }
    }
}

/// Cores do texto da legenda (sempre sobre fundo escuro).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Tone {
    /// Informação neutra.
    #[default]
    Normal,
    /// Pede uma ação do jogador.
    Prompt,
    /// Algo bom para o jogador.
    Good,
    /// Algo ruim para o jogador.
    Bad,
}

impl Tone {
    fn color(self, high_contrast: bool) -> Rgba {
        if high_contrast {
            return Rgba::rgb(0xffffff);
        }
        match self {
            Tone::Normal => Rgba::rgb(0xf0f0f0),
            Tone::Prompt => Rgba::rgb(0xffd23c),
            Tone::Good => Rgba::rgb(0x6fe06f),
            Tone::Bad => Rgba::rgb(0xff7a6a),
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

/// Texto que sobe acima da cabeça de um lutador.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FloatText {
    /// Dano recebido.
    Damage {
        /// Valor.
        amount: i32,
        /// Golpe crítico.
        critical: bool,
    },
    /// Fôlego recuperado.
    Stamina(i32),
    /// Palavra curta já traduzida (ex.: "ERROU").
    Label(&'static str),
}

/// Texto flutuante e quanto ele já subiu.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Floater {
    /// Conteúdo.
    pub text: FloatText,
    /// Linhas lógicas já percorridas (0 = recém-criado).
    pub rise: i32,
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
    /// Aparência e equipamento.
    pub look: Look,
    /// Texto flutuante acima da cabeça.
    pub float: Option<Floater>,
}

/// Estado de um botão.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonState {
    /// Clicável.
    Enabled,
    /// Desabilitado.
    Disabled,
}

/// Etapa do jogo mostrada no palco.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Phase {
    /// Fora de luta: o jogador se prepara e escolhe quando lutar.
    Hub {
        /// Ouro.
        gold: u32,
        /// Pontos de atributo livres.
        unspent: u16,
        /// Rótulo curto de nível (já traduzido, ex.: "NV").
        level_label: String,
        /// Rótulo curto de pontos (já traduzido, ex.: "PTS").
        points_label: String,
        /// Rótulo do botão de luta (já traduzido).
        fight_label: String,
    },
    /// Luta em andamento.
    Fight {
        /// Botões de ação (modo manual) ou `None` (modo automático).
        buttons: Option<[ButtonState; 8]>,
        /// Modo automático pausado.
        paused: bool,
    },
    /// Resultado de luta.
    Result {
        /// Recompensa (ex.: "+45 XP  +30 OURO").
        detail: String,
        /// Rótulo do botão de continuar.
        ok_label: String,
    },
}

/// Conteúdo do palco.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StageView {
    /// Jogador.
    pub player: FighterView,
    /// Oponente (na preparação, o próximo desafio).
    pub opponent: FighterView,
    /// Número de casas da arena.
    pub cells: u8,
    /// Legenda da etapa: o que está acontecendo ou o que fazer agora.
    pub caption: String,
    /// Cor da legenda.
    pub tone: Tone,
    /// Segunda linha opcional (dica), abaixo da legenda.
    pub hint: String,
    /// Etapa.
    pub phase: Phase,
}

/// Escala inteira para uma altura física de palco.
pub fn stage_scale(height_px: i32) -> i32 {
    (height_px / STAGE_HEIGHT).max(1)
}

/// Desenha o palco inteiro. `hover` é a região sob o mouse (realce). `hits` é limpo e preenchido
/// com as regiões clicáveis.
pub fn draw_stage(
    s: &mut Surface,
    view: &StageView,
    theme: Theme,
    hover: Option<HitId>,
    hits: &mut Vec<Hit>,
) {
    hits.clear();
    s.clear(Rgba::TRANSPARENT);
    let sc = stage_scale(s.height() as i32);
    // O chão fica sempre encostado na base da janela (que encosta na barra de tarefas).
    let ctx = Ctx {
        sc,
        ox: 0,
        oy: (s.height() as i32 - STAGE_HEIGHT * sc).max(0),
        theme,
        hover,
    };
    ctx.floor(s, view.cells);
    ctx.fighters(s, &view.player, &view.opponent);
    match &view.phase {
        Phase::Hub {
            gold,
            unspent,
            level_label,
            points_label,
            fight_label,
        } => {
            ctx.band_button(s, 0, Icon::Menu, HitId::OpenPanel, hits);
            let mut buf = NumBuf::new();
            let fg = ctx.theme.fg();
            let mut x = 12;
            x += ctx.band_text(s, x, level_label, 3, fg) + 2;
            x += ctx.band_text(s, x, buf.fmt(u32::from(view.player.level)), 3, fg) + 5;
            x += ctx.band_text(s, x, buf.fmt(*gold), 7, ctx.theme.gold());
            x += ctx.band_text(s, x, "$", 1, ctx.theme.gold()) + 5;
            if *unspent > 0 {
                let good = ctx.theme.good();
                x += ctx.band_text(s, x, "+", 1, good);
                x += ctx.band_text(s, x, buf.fmt(u32::from(*unspent)), 3, good) + 2;
                ctx.band_text(s, x, points_label, 4, good);
            }
            ctx.band_label_button(s, fight_label, HitId::Fight, hits);
        }
        Phase::Fight { buttons, paused } => {
            ctx.hud(s, &view.player, Team::Player);
            ctx.hud(s, &view.opponent, Team::Opponent);
            ctx.band_button(s, 57, Icon::Menu, HitId::OpenPanel, hits);
            match buttons {
                Some(states) => ctx.action_buttons(s, &view.player, &view.opponent, states, hits),
                None => {
                    let icon = if *paused { Icon::Play } else { Icon::Pause };
                    ctx.band_button(s, 67, icon, HitId::TogglePause, hits);
                }
            }
        }
        Phase::Result { detail, ok_label } => {
            ctx.band_button(s, 0, Icon::Menu, HitId::OpenPanel, hits);
            ctx.band_text(s, 12, detail, 20, ctx.theme.fg());
            ctx.band_label_button(s, ok_label, HitId::Continue, hits);
        }
    }
    ctx.caption(s, &view.caption, view.tone);
    if !view.hint.is_empty() {
        let w = draw::text_width(&view.hint).min(STAGE_WIDTH - 2);
        ctx.text(s, (STAGE_WIDTH - w) / 2, 9, &view.hint, ctx.theme_air_fg());
    }
    ctx.floater(s, &view.player);
    ctx.floater(s, &view.opponent);
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

    fn fmt(&mut self, v: u32) -> &str {
        self.fmt_signed(i64::from(v), false)
    }

    /// Número com sinal opcional na frente (`+` para positivos quando `plus` é verdadeiro).
    fn fmt_signed(&mut self, value: i64, plus: bool) -> &str {
        let mut v = value.unsigned_abs();
        let mut tmp = [0u8; 11];
        let mut n = 0;
        loop {
            tmp[n] = b'0' + (v % 10) as u8;
            n += 1;
            v /= 10;
            if v == 0 || n == tmp.len() {
                break;
            }
        }
        let mut len = 0;
        if value < 0 {
            self.buf[0] = b'-';
            len = 1;
        } else if plus {
            self.buf[0] = b'+';
            len = 1;
        }
        for i in 0..n {
            self.buf[len + i] = tmp[n - 1 - i];
        }
        self.len = len + n;
        std::str::from_utf8(&self.buf[..self.len]).unwrap_or("")
    }
}

/// Contexto de desenho: escala, deslocamento, tema e região sob o mouse.
struct Ctx {
    sc: i32,
    ox: i32,
    oy: i32,
    theme: Theme,
    hover: Option<HitId>,
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

    /// Cor do texto solto no ar (sem fundo próprio): sempre claro com contorno escuro, legível
    /// sobre qualquer janela que esteja atrás.
    fn theme_air_fg(&self) -> Rgba {
        Rgba::rgb(0xffffff)
    }

    /// Texto com contorno (para o que fica solto no ar).
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

    /// Texto na faixa de status; devolve a largura lógica desenhada.
    fn band_text(&self, s: &mut Surface, x: i32, t: &str, max: usize, c: Rgba) -> i32 {
        draw::text_max(s, self.px(x), self.py(BAND_Y + 1), self.sc, t, max, c, None) / self.sc
    }

    fn hit(&self, x: i32, y: i32, w: i32, h: i32, id: HitId) -> Hit {
        Hit {
            x: self.px(x),
            y: self.py(y),
            w: w * self.sc,
            h: h * self.sc,
            id,
        }
    }

    /// Centro de um lutador em milésimos de pixel lógico.
    fn center_milli(f: &FighterView) -> i32 {
        ARENA_X * 1000 + f.pos_pm * CELL_W + CELL_W * 500
    }

    /// Centro de um lutador arredondado para a casa mais próxima (pixels lógicos).
    fn center_cell(f: &FighterView) -> i32 {
        let cell = (f.pos_pm + 500).div_euclid(1000);
        ARENA_X + cell * CELL_W + CELL_W / 2
    }

    fn floor(&self, s: &mut Surface, cells: u8) {
        let hc = self.theme.high_contrast;
        self.rect(s, 0, BAND_Y, STAGE_WIDTH, BAND_H, self.theme.band());
        let sand = if hc {
            Rgba::rgb(0xffffff)
        } else {
            Rgba::rgb(0xd9b36a)
        };
        let mark = if hc {
            Rgba::rgb(0x000000)
        } else {
            Rgba::rgb(0x8a6a34)
        };
        self.rect(s, 0, FLOOR_Y, STAGE_WIDTH, 1, sand);
        for c in 0..=i32::from(cells) {
            self.rect(s, ARENA_X + c * CELL_W, FLOOR_Y, 1, 1, mark);
        }
    }

    fn fighters(&self, s: &mut Surface, p: &FighterView, o: &FighterView) {
        // Quem está golpeando fica por cima.
        let order = if o.frame == Frame::Attack {
            [(p, Team::Player), (o, Team::Opponent)]
        } else {
            [(o, Team::Opponent), (p, Team::Player)]
        };
        for (f, team) in order {
            let mirror = team == Team::Opponent;
            let axis = if mirror {
                CANVAS - 1 - CENTER_X
            } else {
                CENTER_X
            };
            // A posição horizontal usa pixels físicos: o movimento entre casas fica suave.
            let x = self.ox + Self::center_milli(f) * self.sc / 1000 - axis * self.sc;
            let palette = SpritePalette::new(&f.look, self.theme.team(team));
            draw::fighter(
                s,
                x,
                self.py(FIGHTER_Y),
                self.sc,
                f.frame,
                &f.look,
                &palette,
                mirror,
                f.flash,
            );
        }
    }

    /// Nome, vida e fôlego de um lutador na faixa de status. Vida e fôlego se distinguem pela forma
    /// e posição (barra cheia em cima, segmentos embaixo), não só pela cor.
    fn hud(&self, s: &mut Surface, f: &FighterView, team: Team) {
        let hc = self.theme.high_contrast;
        let fg = self.theme.fg();
        let bar_w = 28;
        let name_w = draw::text_width(&f.name).min(6 * draw::CHAR_ADVANCE - 1);
        let (name_x, bar_x) = match team {
            Team::Player => (2, 28),
            Team::Opponent => (STAGE_WIDTH - 2 - name_w, STAGE_WIDTH - 28 - bar_w),
        };
        self.band_text(s, name_x, &f.name, 6, fg);
        let empty = if self.theme.light && !hc {
            Rgba::rgb(0x000000).with_alpha(70)
        } else {
            Rgba::rgb(0xffffff).with_alpha(50)
        };
        self.rect(s, bar_x, BAND_Y + 2, bar_w, 3, empty);
        let hp_w = if f.hp_max > 0 {
            (i64::from(f.hp.max(0)) * i64::from(bar_w) / i64::from(f.hp_max)) as i32
        } else {
            0
        };
        let hp_color = if hc { fg } else { Rgba::rgb(0xe8573c) };
        // A vida do oponente esvazia em direção ao centro do palco, como a do jogador.
        let hp_x = match team {
            Team::Player => bar_x,
            Team::Opponent => bar_x + bar_w - hp_w,
        };
        self.rect(s, hp_x, BAND_Y + 2, hp_w, 3, hp_color);
        let segs = 7;
        let filled = if f.stamina_max > 0 {
            (i64::from(f.stamina.max(0)) * i64::from(segs) / i64::from(f.stamina_max)) as i32
        } else {
            0
        };
        let st_color = if hc { fg } else { Rgba::rgb(0xe0a92a) };
        for i in 0..segs {
            let lit = match team {
                Team::Player => i < filled,
                Team::Opponent => i >= segs - filled,
            };
            self.rect(
                s,
                bar_x + i * 4,
                BAND_Y + 6,
                3,
                2,
                if lit { st_color } else { empty },
            );
        }
    }

    /// Botões de ação em volta do jogador: os de recuo e preparo atrás dele, o avanço e os golpes
    /// à frente. A coluna da frente passa para depois do oponente quando os dois estão próximos,
    /// para nunca cobrir a luta.
    fn action_buttons(
        &self,
        s: &mut Surface,
        p: &FighterView,
        o: &FighterView,
        states: &[ButtonState; 8],
        hits: &mut Vec<Hit>,
    ) {
        let pc = Self::center_cell(p);
        let oc = Self::center_cell(o);
        let back_x = (pc - BUTTON_GAP - BUTTON_D).max(0);
        let between = pc + BUTTON_GAP;
        let front_x = if between + BUTTON_D <= oc - BUTTON_GAP {
            between
        } else {
            (oc + BUTTON_GAP).min(STAGE_WIDTH - BUTTON_D)
        };
        for (i, (icon, st)) in ACTION_ICONS.iter().zip(states.iter()).enumerate() {
            let (front, row) = ACTION_SLOTS[i];
            let x = if front { front_x } else { back_x };
            let y = BUTTONS_Y + row * BUTTON_PITCH;
            let id = HitId::Action(i as u8);
            let enabled = *st == ButtonState::Enabled;
            let hovered = self.hover == Some(id);
            let (bg, fg) = self.action_colors(*icon, enabled, hovered);
            draw::disc(s, self.px(x), self.py(y), BUTTON_D, self.sc, bg);
            draw::icon(s, self.px(x + 1), self.py(y + 1), self.sc, *icon, fg);
            // Botões desabilitados também têm região: passar o mouse explica o motivo.
            hits.push(self.hit(x, y, BUTTON_D, BUTTON_D, id));
        }
    }

    /// Cores de um botão de ação. Cada grupo tem uma cor: movimento, golpes e preparo.
    fn action_colors(&self, icon: Icon, enabled: bool, hovered: bool) -> (Rgba, Rgba) {
        if self.theme.high_contrast {
            let fg = if enabled {
                Rgba::rgb(0xffffff)
            } else {
                Rgba::rgb(0x707070)
            };
            return (Rgba::rgb(0x000000), fg);
        }
        let (base, lit) = match icon {
            Icon::Advance | Icon::Retreat => (0x1f4a78, 0x2f6fb0),
            Icon::Light | Icon::Medium | Icon::Heavy => (0x7a261f, 0xb23a2e),
            _ => (0x2f5a2a, 0x46873e),
        };
        match (enabled, hovered) {
            (true, true) => (Rgba::rgb(lit), Rgba::rgb(0xffffff)),
            (true, false) => (Rgba::rgb(base), Rgba::rgb(0xf4f4f4)),
            (false, _) => (
                Rgba::rgb(0x2a2a33).with_alpha(150),
                Rgba::rgb(0x9a9aa6).with_alpha(170),
            ),
        }
    }

    /// Botão quadrado com ícone na faixa de status.
    fn band_button(&self, s: &mut Surface, x: i32, icon: Icon, id: HitId, hits: &mut Vec<Hit>) {
        let hovered = self.hover == Some(id);
        let bg = match (self.theme.high_contrast, hovered) {
            (true, _) => Rgba::rgb(0x000000),
            (false, true) => Rgba::rgb(0x55556a),
            (false, false) => Rgba::rgb(0x3a3a48),
        };
        self.rect(s, x, BAND_Y, BAND_H, BAND_H, bg);
        draw::icon(
            s,
            self.px(x + 1),
            self.py(BAND_Y + 1),
            self.sc,
            icon,
            Rgba::rgb(0xf0f0f0),
        );
        hits.push(self.hit(x, BAND_Y, BAND_H, BAND_H, id));
    }

    /// Botão principal da etapa, sempre no canto direito da faixa de status.
    fn band_label_button(&self, s: &mut Surface, label: &str, id: HitId, hits: &mut Vec<Hit>) {
        let w = draw::text_width(label) + 8;
        let x = STAGE_WIDTH - w;
        let hovered = self.hover == Some(id);
        let bg = match (self.theme.high_contrast, hovered) {
            (true, _) => Rgba::rgb(0xffffff),
            (false, true) => Rgba::rgb(0x3f9a3f),
            (false, false) => Rgba::rgb(0x2f7a2f),
        };
        let fg = if self.theme.high_contrast {
            Rgba::rgb(0x000000)
        } else {
            Rgba::rgb(0xffffff)
        };
        self.rect(s, x, BAND_Y, w, BAND_H, bg);
        draw::text(
            s,
            self.px(x + 4),
            self.py(BAND_Y + 1),
            self.sc,
            label,
            fg,
            None,
        );
        hits.push(self.hit(x, BAND_Y, w, BAND_H, id));
    }

    /// Legenda da etapa: texto centralizado no topo, sobre uma placa escura.
    fn caption(&self, s: &mut Surface, text: &str, tone: Tone) {
        if text.is_empty() {
            return;
        }
        let max = ((STAGE_WIDTH - 6) / draw::CHAR_ADVANCE) as usize;
        let n = text.chars().count().min(max) as i32;
        let w = n * draw::CHAR_ADVANCE - 1;
        let x = (STAGE_WIDTH - w) / 2;
        let plate = if self.theme.high_contrast {
            Rgba::rgb(0x000000)
        } else {
            Rgba::rgb(0x14141a).with_alpha(225)
        };
        self.rect(s, x - 3, 0, w + 6, 8, plate);
        draw::text_max(
            s,
            self.px(x),
            self.py(1),
            self.sc,
            text,
            max,
            tone.color(self.theme.high_contrast),
            None,
        );
    }

    fn floater(&self, s: &mut Surface, f: &FighterView) {
        let Some(fl) = f.float else {
            return;
        };
        let hc = self.theme.high_contrast;
        let mut buf = NumBuf::new();
        let (text, color) = match fl.text {
            FloatText::Damage { amount, critical } => (
                buf.fmt_signed(-i64::from(amount.max(0)), false),
                if hc {
                    Rgba::rgb(0xffffff)
                } else if critical {
                    Rgba::rgb(0xffd23c)
                } else {
                    Rgba::rgb(0xff6a5a)
                },
            ),
            FloatText::Stamina(amount) => (
                buf.fmt_signed(i64::from(amount.max(0)), true),
                if hc {
                    Rgba::rgb(0xffffff)
                } else {
                    Rgba::rgb(0xf2c53d)
                },
            ),
            FloatText::Label(l) => (l, Rgba::rgb(0xe0e0e8)),
        };
        let w = draw::text_width(text);
        let cx = Self::center_milli(f) / 1000;
        let x = (cx - w / 2).clamp(1, (STAGE_WIDTH - w - 1).max(1));
        let y = (FIGHTER_Y + 1 - fl.rise).max(0);
        self.text(s, x, y, text, color);
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
    /// Retrato do lutador à direita das linhas (aba de aparência).
    pub preview: Option<Look>,
}

/// Largura lógica reservada ao retrato no painel.
const PREVIEW_W: i32 = 68;

/// Desenha o painel. A escala é `largura_física / PANEL_WIDTH`.
pub fn draw_panel(s: &mut Surface, view: &PanelView, hits: &mut Vec<Hit>) {
    hits.clear();
    let sc = (s.width() as i32 / PANEL_WIDTH).max(1);
    let theme = Theme {
        light: false,
        high_contrast: false,
    };
    let ctx = Ctx {
        sc,
        ox: 0,
        oy: 0,
        theme,
        hover: None,
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
        hits.push(ctx.hit(x, 3, w, 10, HitId::Tab(i as u8)));
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
    hits.push(ctx.hit(PANEL_WIDTH - 12, 3, 9, 9, HitId::ClosePanel));

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
    // Com retrato, as linhas terminam antes dele.
    let rows_right = if view.preview.is_some() {
        PANEL_WIDTH - 4 - PREVIEW_W - 2
    } else {
        PANEL_WIDTH - 4
    };
    if let Some(look) = &view.preview {
        let px = PANEL_WIDTH - 4 - PREVIEW_W;
        ctx.rect(
            s,
            px,
            top - 1,
            PREVIEW_W,
            PREVIEW_W + 2,
            Rgba::rgb(0x50505c),
        );
        ctx.rect(
            s,
            px + 1,
            top,
            PREVIEW_W - 2,
            PREVIEW_W,
            Rgba::rgb(0x2a2a33),
        );
        ctx.rect(
            s,
            px + 1,
            top + PREVIEW_W - 3,
            PREVIEW_W - 2,
            3,
            Rgba::rgb(0xd9b36a),
        );
        // Retrato no dobro da escala do painel.
        let palette = SpritePalette::new(look, theme.team(Team::Player));
        draw::fighter(
            s,
            ctx.px(px + 2),
            ctx.py(top + 1),
            sc * 2,
            Frame::Idle,
            look,
            &palette,
            false,
            false,
        );
    }
    let max_rows = ((PANEL_HEIGHT - top - 12) / row_h) as usize;
    for (i, row) in view.rows.iter().take(max_rows).enumerate() {
        let y = top + i as i32 * row_h;
        if row.highlight {
            ctx.rect(s, 2, y - 1, rows_right - 2, row_h, Rgba::rgb(0x2c2c3a));
        }
        draw::text_max(s, ctx.px(4), ctx.py(y + 1), sc, &row.text, 24, fg, None);
        let mut right = rows_right;
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
                hits.push(ctx.hit(right, y - 1, w, row_h - 1, HitId::Row(i as u16)));
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
