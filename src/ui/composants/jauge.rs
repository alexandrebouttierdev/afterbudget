//! Jauge de progression.
//!
//! Utilisée pour la consommation du découvert et la part d'une catégorie. Une
//! valeur non nulle reste toujours visible : c'était un défaut de l'ancienne
//! barre, où une catégorie à 0,4 % disparaissait complètement.

use iced::widget::{container, row, Space};
use iced::{Color, Element, Length};

use crate::ui::theme::espacements::Rayon;
use crate::ui::theme::palette::Palette;
use crate::ui::theme::styles;

/// Épaisseur du filet.
pub const EPAISSEUR: f32 = 6.0;

/// Portion remplie, exprimée en millièmes, avec un plancher visible.
///
/// Séparée du rendu pour être testable : c'est la règle de présentation, pas
/// une décoration.
pub fn portions(fraction: f32) -> (u16, u16) {
    let fraction = if fraction.is_finite() {
        fraction.clamp(0.0, 1.0)
    } else {
        0.0
    };
    let remplie = (fraction * 1000.0).round() as u16;
    // Toute valeur strictement positive occupe au moins 1,2 % de la piste.
    let remplie = if fraction > 0.0 { remplie.max(12) } else { 0 };
    (remplie, 1000 - remplie)
}

/// Jauge simple.
pub fn jauge<'a, Message: 'a>(
    fraction: f32,
    couleur: Color,
    palette: Palette,
) -> Element<'a, Message> {
    let (remplie, reste) = portions(fraction);

    let mut piste = row![];
    if remplie > 0 {
        piste = piste.push(
            container(Space::new(Length::Fill, Length::Fixed(EPAISSEUR)))
                .width(Length::FillPortion(remplie))
                .height(Length::Fixed(EPAISSEUR))
                .style(move |_theme| styles::surface_nue(palette, couleur, Rayon::PLEIN)),
        );
    }
    if reste > 0 {
        piste = piste.push(Space::new(
            Length::FillPortion(reste),
            Length::Fixed(EPAISSEUR),
        ));
    }

    container(piste)
        .width(Length::Fill)
        .height(Length::Fixed(EPAISSEUR))
        .style(move |_theme| styles::surface_nue(palette, palette.surface_basse, Rayon::PLEIN))
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn une_valeur_nulle_ne_remplit_rien() {
        assert_eq!(portions(0.0), (0, 1000));
    }

    #[test]
    fn une_valeur_pleine_remplit_tout() {
        assert_eq!(portions(1.0), (1000, 0));
    }

    /// Une part très faible mais réelle doit rester visible.
    #[test]
    fn une_part_minuscule_reste_visible() {
        let (remplie, _) = portions(0.001);
        assert!(remplie >= 12, "une part de 0,1 % disparaîtrait");
    }

    /// Les valeurs aberrantes ne doivent pas produire de mise en page invalide.
    #[test]
    fn les_valeurs_aberrantes_sont_bornees() {
        assert_eq!(portions(-4.0), (0, 1000));
        assert_eq!(portions(12.0), (1000, 0));
        assert_eq!(portions(f32::NAN), (0, 1000));
        assert_eq!(portions(f32::INFINITY), (0, 1000));
    }

    /// Les deux portions couvrent toujours exactement la piste.
    #[test]
    fn les_portions_couvrent_toute_la_piste() {
        for millieme in 0..=1000 {
            let (a, b) = portions(millieme as f32 / 1000.0);
            assert_eq!(a + b, 1000);
        }
    }
}
