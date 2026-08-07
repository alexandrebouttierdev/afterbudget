//! Section de paramètres.
//!
//! Un titre, une explication, puis des réglages posés en lignes « intitulé à
//! gauche, contrôle à droite ». Évite de multiplier les cartes sans raison :
//! une section, une carte.

use std::borrow::Cow;

use iced::widget::{column, container, row, Space};
use iced::{Alignment, Element, Length};

use crate::ui::composants::carte;
use crate::ui::composants::icone::{icone, Icone, Taille as TailleIcone};
use crate::ui::composants::separateur;
use crate::ui::theme::espacements::{Esp, Rayon};
use crate::ui::theme::palette::Palette;
use crate::ui::theme::styles;
use crate::ui::theme::typographie::{texte_colore, Role};

/// Registre visuel d'une section.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Registre {
    /// Réglage courant.
    Courant,
    /// Zone dangereuse : actions irréversibles, clairement séparées.
    Dangereux,
}

/// Section complète.
pub fn section<'a, Message: 'a>(
    symbole: Icone,
    titre: impl Into<Cow<'a, str>>,
    explication: impl Into<Cow<'a, str>>,
    reglages: Vec<Element<'a, Message>>,
    registre: Registre,
    palette: Palette,
) -> Element<'a, Message> {
    let teinte = match registre {
        Registre::Courant => palette.accent,
        Registre::Dangereux => palette.danger,
    };

    let entete = row![
        container(icone(symbole, TailleIcone::Normale, teinte))
            .padding(Esp::SM)
            .style(move |_theme| {
                styles::surface_nue(palette, palette.teinter_surface(teinte), Rayon::MD)
            }),
        column![
            texte_colore(titre.into(), Role::TitreSection, palette.texte_fort),
            texte_colore(explication.into(), Role::Legende, palette.texte_doux),
        ]
        .spacing(Esp::XXS),
    ]
    .spacing(Esp::MD)
    .align_y(Alignment::Center);

    let mut contenu = column![entete].spacing(Esp::LG).width(Length::Fill);

    for (index, reglage) in reglages.into_iter().enumerate() {
        if index == 0 {
            contenu = contenu.push(separateur::horizontal(palette));
        }
        contenu = contenu.push(reglage);
    }

    carte::carte(contenu, carte::Variante::Plate, palette).into()
}

/// Ligne de réglage : intitulé et aide à gauche, contrôle à droite.
pub fn reglage<'a, Message: 'a>(
    intitule: impl Into<Cow<'a, str>>,
    aide: impl Into<Cow<'a, str>>,
    controle: impl Into<Element<'a, Message>>,
    palette: Palette,
) -> Element<'a, Message> {
    row![
        column![
            texte_colore(intitule.into(), Role::CorpsFort, palette.texte_fort),
            texte_colore(aide.into(), Role::Legende, palette.texte_doux),
        ]
        .spacing(Esp::XXS)
        .width(Length::FillPortion(3)),
        Space::with_width(Length::Fixed(f32::from(Esp::LG))),
        container(controle.into())
            .width(Length::FillPortion(2))
            .align_x(Alignment::End),
    ]
    .align_y(Alignment::Center)
    .width(Length::Fill)
    .into()
}

/// Réglage occupant toute la largeur : le contrôle passe sous l'intitulé.
pub fn reglage_large<'a, Message: 'a>(
    intitule: impl Into<Cow<'a, str>>,
    aide: impl Into<Cow<'a, str>>,
    controle: impl Into<Element<'a, Message>>,
    palette: Palette,
) -> Element<'a, Message> {
    column![
        texte_colore(intitule.into(), Role::CorpsFort, palette.texte_fort),
        texte_colore(aide.into(), Role::Legende, palette.texte_doux),
        Space::with_height(Length::Fixed(f32::from(Esp::XS))),
        controle.into(),
    ]
    .spacing(Esp::XXS)
    .width(Length::Fill)
    .into()
}
