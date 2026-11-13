//! Gera PNGs de pré-visualização das cenas: `cargo run -p render --example preview -- <pasta>`.

use assets::Frame;
use render::scene::{PANEL_HEIGHT, PANEL_WIDTH, STRIP_WIDTH};
use render::{
    ButtonState, FighterView, PanelRow, PanelView, StripView, Surface, Theme, draw_panel,
    draw_strip, png,
};

fn fighter(name: &str, pos: i32, frame: Frame, hp: i32, flash: bool) -> FighterView {
    FighterView {
        name: name.into(),
        level: 7,
        hp,
        hp_max: 88,
        stamina: 40,
        stamina_max: 60,
        pos_pm: pos * 1000,
        frame,
        flash,
    }
}

fn main() -> std::io::Result<()> {
    let dir = std::env::args().nth(1).unwrap_or_else(|| "preview".into());
    std::fs::create_dir_all(&dir)?;
    let mut hits = Vec::new();
    let dark = Theme {
        light: false,
        high_contrast: false,
    };
    let light = Theme {
        light: true,
        high_contrast: false,
    };
    let views = [
        (
            "luta_manual",
            StripView::Fight {
                player: fighter("ARIA", 3, Frame::Attack, 61, false),
                opponent: fighter("OSSIAN", 4, Frame::Defend, 34, true),
                cells: 10,
                buttons: Some([
                    ButtonState::Disabled,
                    ButtonState::Enabled,
                    ButtonState::Enabled,
                    ButtonState::Enabled,
                    ButtonState::Disabled,
                    ButtonState::Enabled,
                    ButtonState::Enabled,
                    ButtonState::Enabled,
                ]),
                paused: false,
                banner: Some("CRÍTICO!".into()),
            },
        ),
        (
            "luta_auto",
            StripView::Fight {
                player: fighter("ARIA", 2, Frame::Idle, 88, false),
                opponent: fighter("CASTOR", 7, Frame::Down, 0, false),
                cells: 10,
                buttons: None,
                paused: true,
                banner: None,
            },
        ),
        (
            "hub",
            StripView::Hub {
                name: "Aria".into(),
                level: 7,
                gold: 1234,
                unspent: 3,
                next_label: "PRÓXIMO: LIRA PASSO-LEVE (NV 6)".into(),
                fight_label: "LUTAR".into(),
            },
        ),
        (
            "resultado",
            StripView::Result {
                title: "VITÓRIA!".into(),
                detail: "+92 XP   +57 OURO".into(),
                won: true,
                ok_label: "OK".into(),
            },
        ),
    ];
    for (name, view) in &views {
        for (h, theme, tname) in [
            (24, dark, "escuro"),
            (48, dark, "escuro"),
            (48, light, "claro"),
            (72, dark, "escuro"),
        ] {
            let (sc, _) = render::strip_scale(h);
            let mut s = Surface::new((STRIP_WIDTH * sc) as usize, h as usize);
            draw_strip(&mut s, view, theme, &mut hits);
            let bg = if theme.light { 0xe6e6ea } else { 0x202027 };
            std::fs::write(
                format!("{dir}/{name}_{h}px_{tname}.png"),
                png::encode(&s, Some(bg)),
            )?;
        }
    }
    let panel = PanelView {
        tabs: vec![
            "FICHA".into(),
            "LOJA".into(),
            "LIGA".into(),
            "CONFIG".into(),
        ],
        active_tab: 1,
        header: "OURO: 1234".into(),
        rows: vec![
            PanelRow {
                text: "ESPADA CURTA DE BRONZE".into(),
                value: "60".into(),
                button: Some(("EQUIPADO".into(), false)),
                highlight: true,
            },
            PanelRow {
                text: "LANÇA DE FREIXO".into(),
                value: "140".into(),
                button: Some(("COMPRAR".into(), true)),
                highlight: false,
            },
            PanelRow {
                text: "MACHADO DE FERRO".into(),
                value: "300".into(),
                button: Some(("COMPRAR".into(), false)),
                highlight: false,
            },
            PanelRow {
                text: "TÚNICA ACOLCHOADA".into(),
                value: "40".into(),
                button: Some(("EQUIPAR".into(), true)),
                highlight: false,
            },
        ],
        footer: "CLIQUE FORA PARA FECHAR".into(),
    };
    let mut s = Surface::new((PANEL_WIDTH * 2) as usize, (PANEL_HEIGHT * 2) as usize);
    draw_panel(&mut s, &panel, &mut hits);
    std::fs::write(format!("{dir}/painel_loja.png"), png::encode(&s, None))?;
    println!("PNGs gravados em {dir}");
    Ok(())
}
