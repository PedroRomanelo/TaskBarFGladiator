//! Textos da interface em português do Brasil e inglês. Nenhum texto visível fica fora daqui.

use serde::{Deserialize, Serialize};

/// Idioma.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Lang {
    /// Português do Brasil.
    Pt,
    /// Inglês.
    En,
}

impl Lang {
    /// O outro idioma.
    pub fn toggle(self) -> Lang {
        match self {
            Lang::Pt => Lang::En,
            Lang::En => Lang::Pt,
        }
    }
}

/// Chave de texto.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(missing_docs)]
pub enum Key {
    Fight,
    Next,
    Training,
    LeagueDone,
    Victory,
    Defeat,
    Ok,
    Critical,
    Miss,
    Auto,
    TabSheet,
    TabShop,
    TabLeague,
    TabConfig,
    Level,
    Gold,
    Points,
    Xp,
    Strength,
    Technique,
    Agility,
    Defense,
    Vitality,
    Stamina,
    Mode,
    Manual,
    Change,
    AutoAllocate,
    On,
    Off,
    Buy,
    Equip,
    Equipped,
    Beaten,
    Locked,
    Language,
    ReduceMotion,
    StartWithWindows,
    Reposition,
    Pace,
    PaceSlow,
    PaceNormal,
    PaceFast,
    Quit,
    Toggle,
    ClickOutside,
    NotInFight,
    TrayShow,
    TrayHide,
    TrayPause,
    TrayResume,
    TrayPanel,
    TrayReposition,
    TrayQuit,
    Aggressive,
    Defensive,
    Opportunist,
    Balanced,
    Champion,
    NotEnoughGold,
    InventoryFull,
}

/// Texto traduzido.
pub fn t(lang: Lang, key: Key) -> &'static str {
    use Key::*;
    match lang {
        Lang::Pt => match key {
            Fight => "LUTAR",
            Next => "PRÓXIMO",
            Training => "TREINO",
            LeagueDone => "LIGA CONCLUÍDA!",
            Victory => "VITÓRIA!",
            Defeat => "DERROTA",
            Ok => "OK",
            Critical => "CRÍTICO!",
            Miss => "ERROU",
            Auto => "AUTO",
            TabSheet => "FICHA",
            TabShop => "LOJA",
            TabLeague => "LIGA",
            TabConfig => "CONFIG",
            Level => "NV",
            Gold => "OURO",
            Points => "PONTOS",
            Xp => "XP",
            Strength => "FORÇA",
            Technique => "TÉCNICA",
            Agility => "AGILIDADE",
            Defense => "DEFESA",
            Vitality => "VITALIDADE",
            Stamina => "FÔLEGO",
            Mode => "MODO",
            Manual => "MANUAL",
            Change => "TROCAR",
            AutoAllocate => "DISTRIBUIR PONTOS SOZINHO",
            On => "LIGADO",
            Off => "DESLIGADO",
            Buy => "COMPRAR",
            Equip => "EQUIPAR",
            Equipped => "EQUIPADO",
            Beaten => "VENCIDO",
            Locked => "BLOQUEADO",
            Language => "IDIOMA",
            ReduceMotion => "REDUZIR ANIMAÇÕES",
            StartWithWindows => "INICIAR COM O WINDOWS",
            Reposition => "REPOSICIONAR FAIXA",
            Pace => "RITMO DO AUTOMÁTICO",
            PaceSlow => "LENTO",
            PaceNormal => "NORMAL",
            PaceFast => "RÁPIDO",
            Quit => "SAIR DO JOGO",
            Toggle => "TROCAR",
            ClickOutside => "ESC OU CLIQUE FORA PARA FECHAR",
            NotInFight => "INDISPONÍVEL DURANTE A LUTA",
            TrayShow => "Mostrar faixa",
            TrayHide => "Ocultar faixa",
            TrayPause => "Pausar",
            TrayResume => "Continuar",
            TrayPanel => "Painel",
            TrayReposition => "Reposicionar",
            TrayQuit => "Sair",
            Aggressive => "AGRESSIVO",
            Defensive => "DEFENSIVO",
            Opportunist => "OPORTUNISTA",
            Balanced => "EQUILIBRADO",
            Champion => "CAMPEÃO",
            NotEnoughGold => "OURO INSUFICIENTE",
            InventoryFull => "INVENTÁRIO CHEIO",
        },
        Lang::En => match key {
            Fight => "FIGHT",
            Next => "NEXT",
            Training => "TRAINING",
            LeagueDone => "LEAGUE COMPLETE!",
            Victory => "VICTORY!",
            Defeat => "DEFEAT",
            Ok => "OK",
            Critical => "CRITICAL!",
            Miss => "MISS",
            Auto => "AUTO",
            TabSheet => "SHEET",
            TabShop => "SHOP",
            TabLeague => "LEAGUE",
            TabConfig => "SETTINGS",
            Level => "LV",
            Gold => "GOLD",
            Points => "POINTS",
            Xp => "XP",
            Strength => "STRENGTH",
            Technique => "TECHNIQUE",
            Agility => "AGILITY",
            Defense => "DEFENSE",
            Vitality => "VITALITY",
            Stamina => "STAMINA",
            Mode => "MODE",
            Manual => "MANUAL",
            Change => "CHANGE",
            AutoAllocate => "AUTO-ASSIGN POINTS",
            On => "ON",
            Off => "OFF",
            Buy => "BUY",
            Equip => "EQUIP",
            Equipped => "EQUIPPED",
            Beaten => "BEATEN",
            Locked => "LOCKED",
            Language => "LANGUAGE",
            ReduceMotion => "REDUCE MOTION",
            StartWithWindows => "START WITH WINDOWS",
            Reposition => "RESET STRIP POSITION",
            Pace => "AUTO MODE PACE",
            PaceSlow => "SLOW",
            PaceNormal => "NORMAL",
            PaceFast => "FAST",
            Quit => "QUIT GAME",
            Toggle => "TOGGLE",
            ClickOutside => "ESC OR CLICK OUTSIDE TO CLOSE",
            NotInFight => "UNAVAILABLE DURING A FIGHT",
            TrayShow => "Show strip",
            TrayHide => "Hide strip",
            TrayPause => "Pause",
            TrayResume => "Resume",
            TrayPanel => "Panel",
            TrayReposition => "Reset position",
            TrayQuit => "Quit",
            Aggressive => "AGGRESSIVE",
            Defensive => "DEFENSIVE",
            Opportunist => "OPPORTUNIST",
            Balanced => "BALANCED",
            Champion => "CHAMPION",
            NotEnoughGold => "NOT ENOUGH GOLD",
            InventoryFull => "INVENTORY FULL",
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_texts_fit_the_font() {
        // Todo texto da faixa precisa ter glifo na fonte (sem cair no '?').
        for lang in [Lang::Pt, Lang::En] {
            for key in [
                Key::Fight,
                Key::Victory,
                Key::Defeat,
                Key::Critical,
                Key::Next,
                Key::Auto,
            ] {
                for c in t(lang, key).chars() {
                    if c != '?' {
                        assert_ne!(assets::glyph(c), assets::glyph('?'), "{c:?} em {key:?}");
                    }
                }
            }
        }
    }
}
