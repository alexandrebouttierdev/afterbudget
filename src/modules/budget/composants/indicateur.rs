//! Carte d'indicateur secondaire.
//!
//! Volontairement plus discrète que le bloc du solde : un filet coloré, un
//! intitulé, un montant, une précision. Jamais le même poids visuel que la
//! donnée centrale.

use std::borrow::Cow;

use iced::widget::{column, container, row, Space};
use iced::{Alignment, Color, Element, Length};

use crate::ui::composants::icone::{icone, Icone, Taille as TailleIcone};
use crate::ui::theme::espacements::{Esp, Rayon};
use crate::ui::theme::palette::Palette;
use crate::ui::theme::styles::{self, Elevation};
use crate::ui::theme::typographie::{texte_colore, Role};

/// Hauteur du filet coloré, calée sur les trois lignes de texte de la carte.
const HAUTEUR_FILET: f32 = 56.0;

/// Indicateur : intitulé, montant, précision, teinte et icône.
pub fn indicateur<'a, Message: 'a>(
    intitule: impl Into<Cow<'a, str>>,
    montant: impl Into<Cow<'a, str>>,
    precision: impl Into<Cow<'a, str>>,
    symbole: Icone,
    teinte: Color,
    palette: Palette,
) -> Element<'a, Message> {
    // Hauteur fixe : un filet extensible ferait remplir l'axe vertical au
    // contenu défilant, ce qu'Iced refuse dans un `scrollable`.
    let filet = container(Space::new(Length::Fixed(0.0), Length::Fixed(HAUTEUR_FILET)))
        .width(Length::Fixed(3.0))
        .height(Length::Fixed(HAUTEUR_FILET))
        .style(move |_theme| styles::surface_nue(palette, teinte, Rayon::PLEIN));

    let corps = column![
        row![
            icone(symbole, TailleIcone::Petite, teinte),
            texte_colore(intitule.into(), Role::Micro, palette.texte_doux),
        ]
        .spacing(Esp::XS + 1)
        .align_y(Alignment::Center),
        texte_colore(montant.into(), Role::MontantFort, palette.texte_fort),
        texte_colore(precision.into(), Role::Legende, palette.texte_doux),
    ]
    .spacing(Esp::XS);

    container(
        row![filet, corps]
            .spacing(Esp::MD)
            .align_y(Alignment::Center),
    )
    .padding(Esp::LG)
    .width(Length::Fill)
    .style(move |_theme| styles::surface(palette, Elevation::Plate, Rayon::LG))
    .into()
}

/// Accord du décompte de transactions affiché sous un indicateur.
pub fn precision_de_comptage(nombre: usize) -> String {
    match nombre {
        0 => "Rien en attente".to_string(),
        1 => "1 transaction en attente".to_string(),
        n => format!("{n} transactions en attente"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn le_comptage_est_accorde() {
        assert_eq!(precision_de_comptage(0), "Rien en attente");
        assert_eq!(precision_de_comptage(1), "1 transaction en attente");
        assert_eq!(precision_de_comptage(4), "4 transactions en attente");
    }
}
