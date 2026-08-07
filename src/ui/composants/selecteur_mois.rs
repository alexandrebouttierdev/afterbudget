//! Sélecteur de mois.
//!
//! Deux chevrons encadrant le mois affiché, plus un retour explicite au mois
//! courant lorsqu'on s'en est éloigné.

use iced::widget::{container, row};
use iced::{Alignment, Element, Length};

use crate::core::utils;
use crate::ui::composants::bouton::{Bouton, Taille as TailleBouton, Variante};
use crate::ui::composants::icone::Icone;
use crate::ui::composants::infobulle::{infobulle, Position};
use crate::ui::theme::espacements::{Esp, Rayon};
use crate::ui::theme::palette::Palette;
use crate::ui::theme::styles::{self, Elevation};
use crate::ui::theme::typographie::{texte_colore, Role};

/// Messages émis par le sélecteur.
pub struct Actions<Message> {
    pub precedent: Message,
    pub suivant: Message,
    pub aujourdhui: Message,
}

pub fn selecteur_mois<'a, Message: Clone + 'a>(
    annee: i32,
    mois: u32,
    mois_courant: bool,
    actions: Actions<Message>,
    palette: Palette,
) -> Element<'a, Message> {
    let groupe = container(
        row![
            infobulle(
                Bouton::icone(Icone::ChevronGauche, palette)
                    .sur_clic(actions.precedent)
                    .vue(),
                "Mois précédent",
                Position::Bottom,
                palette,
            ),
            container(texte_colore(
                utils::month_year(annee, mois),
                Role::TitreSection,
                palette.texte_fort,
            ))
            .width(Length::Fixed(150.0))
            .align_x(Alignment::Center),
            infobulle(
                Bouton::icone(Icone::ChevronDroit, palette)
                    .sur_clic(actions.suivant)
                    .vue(),
                "Mois suivant",
                Position::Bottom,
                palette,
            ),
        ]
        .align_y(Alignment::Center)
        .spacing(Esp::XS),
    )
    .padding(Esp::XS)
    .style(move |_theme| styles::surface(palette, Elevation::Plate, Rayon::MD));

    let mut ligne = row![groupe].align_y(Alignment::Center).spacing(Esp::SM);

    if !mois_courant {
        ligne = ligne.push(
            Bouton::nouveau("Aujourd'hui", palette)
                .variante(Variante::Discret)
                .taille(TailleBouton::Compacte)
                .sur_clic(actions.aujourdhui)
                .vue(),
        );
    }

    ligne.into()
}
