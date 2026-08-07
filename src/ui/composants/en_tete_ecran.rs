//! En-tête d'écran.
//!
//! Fixe en haut de la zone de contenu : le titre reste visible quand le
//! contenu défile. Porte le titre, une précision facultative et les actions
//! propres à l'écran.

use std::borrow::Cow;

use iced::widget::{column, container, row, Space};
use iced::{Alignment, Element, Length};

use crate::ui::theme::espacements::Esp;
use crate::ui::theme::palette::Palette;
use crate::ui::theme::styles;
use crate::ui::theme::typographie::{texte_colore, Role};

/// En-tête d'écran.
pub fn en_tete<'a, Message: 'a>(
    titre: impl Into<Cow<'a, str>>,
    precision: Option<Cow<'a, str>>,
    milieu: Option<Element<'a, Message>>,
    actions: Vec<Element<'a, Message>>,
    palette: Palette,
) -> Element<'a, Message> {
    let mut identite = column![texte_colore(
        titre.into(),
        Role::TitreEcran,
        palette.texte_fort
    )]
    .spacing(Esp::XXS);
    if let Some(precision) = precision {
        identite = identite.push(texte_colore(precision, Role::Legende, palette.texte_doux));
    }

    let mut ligne = row![identite].align_y(Alignment::Center).spacing(Esp::LG);

    if let Some(milieu) = milieu {
        ligne = ligne.push(Space::with_width(Length::Fill));
        ligne = ligne.push(milieu);
    }

    ligne = ligne.push(Space::with_width(Length::Fill));
    let mut zone_actions = row![].spacing(Esp::SM).align_y(Alignment::Center);
    for action in actions {
        zone_actions = zone_actions.push(action);
    }
    ligne = ligne.push(zone_actions);

    container(ligne)
        .width(Length::Fill)
        .padding([Esp::LG, Esp::XL])
        .style(move |_theme| styles::surface_nue(palette, palette.fond, 0.0))
        .into()
}
