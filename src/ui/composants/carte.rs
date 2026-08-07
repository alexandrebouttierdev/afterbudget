//! Cartes et panneaux.
//!
//! Le poids d'un bloc se lit à son élévation et à son format, jamais à une
//! bordure décorative ajoutée au cas par cas.

use iced::widget::{container, Container};
use iced::{Element, Length};

use crate::ui::theme::espacements::{Esp, Rayon};
use crate::ui::theme::palette::Palette;
use crate::ui::theme::styles::{self, Elevation};

/// Variante de carte.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Variante {
    /// Bloc courant, posé sur le fond de page.
    Plate,
    /// Bloc principal d'un écran : plus de rayon, plus de padding.
    Heros,
    /// Encoche : plus basse que la surface qui la porte.
    Encoche,
    /// Bloc détaché : menu, infobulle, notification.
    Flottante,
}

impl Variante {
    fn elevation(self) -> Elevation {
        match self {
            Self::Plate | Self::Heros => Elevation::Plate,
            Self::Encoche => Elevation::Basse,
            Self::Flottante => Elevation::Flottante,
        }
    }

    fn rayon(self) -> f32 {
        match self {
            Self::Heros => Rayon::XL,
            Self::Plate | Self::Flottante => Rayon::LG,
            Self::Encoche => Rayon::MD,
        }
    }

    fn padding(self) -> u16 {
        match self {
            Self::Heros => Esp::XL,
            Self::Plate | Self::Flottante => Esp::LG,
            Self::Encoche => Esp::MD,
        }
    }
}

/// Carte enveloppant un contenu quelconque.
pub fn carte<'a, Message: 'a>(
    contenu: impl Into<Element<'a, Message>>,
    variante: Variante,
    palette: Palette,
) -> Container<'a, Message> {
    container(contenu)
        .padding(variante.padding())
        .width(Length::Fill)
        .style(move |_theme| styles::surface(palette, variante.elevation(), variante.rayon()))
}

/// Carte sans padding : la liste qu'elle contient gère elle-même ses marges,
/// pour que les séparateurs aillent d'un bord à l'autre.
pub fn carte_liste<'a, Message: 'a>(
    contenu: impl Into<Element<'a, Message>>,
    palette: Palette,
) -> Container<'a, Message> {
    container(contenu)
        .width(Length::Fill)
        .style(move |_theme| styles::surface(palette, Elevation::Plate, Rayon::LG))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// La carte héros doit se distinguer d'une carte courante par son format,
    /// afin que la hiérarchie du tableau de bord soit lisible sans couleur.
    #[test]
    fn la_carte_heros_est_plus_genereuse() {
        assert!(Variante::Heros.padding() > Variante::Plate.padding());
        assert!(Variante::Heros.rayon() > Variante::Plate.rayon());
    }

    /// L'encoche doit descendre d'un niveau, la carte flottante monter.
    #[test]
    fn les_elevations_sont_ordonnees() {
        assert_eq!(Variante::Encoche.elevation(), Elevation::Basse);
        assert_eq!(Variante::Plate.elevation(), Elevation::Plate);
        assert_eq!(Variante::Flottante.elevation(), Elevation::Flottante);
    }
}
