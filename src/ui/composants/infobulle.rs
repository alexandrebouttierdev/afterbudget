//! Infobulles.
//!
//! Donnent leur intitulé aux boutons réduits à une icône, et l'unité ou le
//! détail d'un chiffre condensé.

use std::borrow::Cow;

use iced::widget::{container, text, tooltip};
use iced::Element;

use crate::ui::theme::espacements::{Esp, Rayon};
use crate::ui::theme::palette::Palette;
use crate::ui::theme::styles::{self, Elevation};
use crate::ui::theme::typographie::{police, Role};

pub use iced::widget::tooltip::Position;

/// Enveloppe un élément d'une infobulle.
pub fn infobulle<'a, Message: 'a>(
    contenu: impl Into<Element<'a, Message>>,
    intitule: impl Into<Cow<'a, str>>,
    position: Position,
    palette: Palette,
) -> Element<'a, Message> {
    let bulle = container(
        text(intitule.into())
            .size(Role::Legende.taille())
            .font(police(Role::Legende.graisse()))
            .color(palette.texte_fort),
    )
    .padding([Esp::XS + 1, Esp::SM])
    .style(move |_theme| styles::surface(palette, Elevation::Flottante, Rayon::SM));

    tooltip(contenu, bulle, position)
        .gap(Esp::XS)
        .snap_within_viewport(true)
        .into()
}
