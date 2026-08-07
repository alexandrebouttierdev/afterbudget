//! Badges d'état.
//!
//! Un badge porte toujours **une pastille et un libellé** : l'information n'est
//! jamais transmise par la seule couleur.

use std::borrow::Cow;

use iced::widget::{container, row, text, Space};
use iced::{Alignment, Color, Element, Length};

use crate::ui::composants::icone::{icone, Icone, Taille as TailleIcone};
use crate::ui::theme::espacements::{Esp, Rayon};
use crate::ui::theme::palette::Palette;
use crate::ui::theme::styles;
use crate::ui::theme::typographie::{police, Role};

/// Registre sémantique d'un badge.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ton {
    Neutre,
    Succes,
    Attention,
    Danger,
    Accent,
}

impl Ton {
    /// Couleur du texte et de la pastille.
    pub fn teinte(self, palette: Palette) -> Color {
        match self {
            // `texte_doux` ne passe pas le seuil de contraste sur
            // `surface_basse` en thème clair : le badge neutre utilise `texte`.
            Self::Neutre => palette.texte,
            Self::Succes => palette.succes,
            Self::Attention => palette.attention,
            Self::Danger => palette.danger,
            Self::Accent => palette.accent,
        }
    }

    /// Fond du badge.
    pub fn fond(self, palette: Palette) -> Color {
        match self {
            Self::Neutre => palette.surface_basse,
            Self::Succes => palette.succes_surface,
            Self::Attention => palette.attention_surface,
            Self::Danger => palette.danger_surface,
            Self::Accent => palette.accent_surface,
        }
    }
}

/// Badge « pastille + libellé ».
pub fn badge<'a, Message: 'a>(
    libelle: impl Into<Cow<'a, str>>,
    ton: Ton,
    palette: Palette,
) -> Element<'a, Message> {
    let teinte = ton.teinte(palette);

    let pastille = container(Space::new(Length::Fixed(0.0), Length::Fixed(0.0)))
        .width(Length::Fixed(6.0))
        .height(Length::Fixed(6.0))
        .style(move |_theme| styles::surface_nue(palette, teinte, Rayon::PLEIN));

    let contenu = row![
        pastille,
        text(libelle.into())
            .size(Role::Micro.taille())
            .font(police(Role::Micro.graisse()))
            .color(teinte),
    ]
    .spacing(Esp::SM - 2)
    .align_y(Alignment::Center);

    container(contenu)
        .padding([Esp::XS - 1, Esp::SM])
        .style(move |_theme| {
            styles::surface_semantique(palette, ton.fond(palette), teinte, Rayon::SM)
        })
        .into()
}

/// Badge portant une icône plutôt qu'une pastille, pour les alertes.
pub fn badge_icone<'a, Message: 'a>(
    libelle: impl Into<Cow<'a, str>>,
    symbole: Icone,
    ton: Ton,
    palette: Palette,
) -> Element<'a, Message> {
    let teinte = ton.teinte(palette);

    let contenu = row![
        icone(symbole, TailleIcone::Petite, teinte),
        text(libelle.into())
            .size(Role::Micro.taille())
            .font(police(Role::Micro.graisse()))
            .color(teinte),
    ]
    .spacing(Esp::XS + 1)
    .align_y(Alignment::Center);

    container(contenu)
        .padding([Esp::XS - 1, Esp::SM])
        .style(move |_theme| {
            styles::surface_semantique(palette, ton.fond(palette), teinte, Rayon::SM)
        })
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::theme::palette::{contraste, ThemeMode};

    /// Le texte d'un badge doit rester lisible sur son propre fond : c'était le
    /// défaut principal des badges précédents (1,9:1 pour « En attente »).
    #[test]
    fn le_texte_reste_lisible_sur_le_fond_du_badge() {
        for mode in [ThemeMode::Light, ThemeMode::Dark] {
            let p = Palette::pour(mode);
            for ton in [
                Ton::Neutre,
                Ton::Succes,
                Ton::Attention,
                Ton::Danger,
                Ton::Accent,
            ] {
                let ratio = contraste(ton.teinte(p), ton.fond(p));
                assert!(
                    ratio >= 4.5,
                    "{ton:?} : contraste {ratio:.2} insuffisant ({mode:?})"
                );
            }
        }
    }

    /// Deux tons différents doivent aussi se distinguer par leur fond, sinon
    /// seul le libellé porte l'information.
    #[test]
    fn les_tons_se_distinguent_par_leur_fond() {
        for mode in [ThemeMode::Light, ThemeMode::Dark] {
            let p = Palette::pour(mode);
            let tons = [
                Ton::Neutre,
                Ton::Succes,
                Ton::Attention,
                Ton::Danger,
                Ton::Accent,
            ];
            for (i, a) in tons.iter().enumerate() {
                for b in &tons[i + 1..] {
                    assert_ne!(a.fond(p), b.fond(p), "{a:?} et {b:?} ont le même fond");
                }
            }
        }
    }
}
