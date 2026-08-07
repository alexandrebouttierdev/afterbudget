//! Échelles d'espacement, de rayon, d'ombre et de densité.
//!
//! Aucune de ces valeurs ne doit être réécrite en littéral dans une vue.

use iced::{Shadow, Vector};

use super::palette::{avec_alpha, Palette};

/// Échelle d'espacement, pas de 4 px.
pub struct Esp;

impl Esp {
    /// Écart à l'intérieur d'une même ligne.
    pub const XXS: u16 = 2;
    /// Libellé et sa valeur.
    pub const XS: u16 = 4;
    /// Éléments d'un même groupe.
    pub const SM: u16 = 8;
    /// Padding d'une ligne dense, écart entre champs.
    pub const MD: u16 = 12;
    /// Padding de carte, écart entre blocs.
    pub const LG: u16 = 16;
    /// Padding d'écran, écart entre sections.
    pub const XL: u16 = 24;
    /// Respiration d'une modale.
    pub const XXL: u16 = 32;
    /// État vide, onboarding.
    pub const XXXL: u16 = 48;
}

/// Rayons de bordure.
pub struct Rayon;

impl Rayon {
    /// Pastille, jauge.
    pub const XS: f32 = 4.0;
    /// Badge, bouton compact.
    pub const SM: f32 = 6.0;
    /// Bouton, champ.
    pub const MD: f32 = 8.0;
    /// Carte, panneau.
    pub const LG: f32 = 12.0;
    /// Carte héros, modale.
    pub const XL: f32 = 16.0;
    /// Disque, indicateur.
    pub const PLEIN: f32 = 999.0;
}

/// Hauteur des lignes de liste.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Densite {
    Compacte,
    Normale,
    Confortable,
}

impl Densite {
    pub const fn hauteur(self) -> f32 {
        match self {
            Self::Compacte => 32.0,
            Self::Normale => 44.0,
            Self::Confortable => 56.0,
        }
    }

    /// Padding vertical correspondant, en gardant les paddings horizontaux
    /// à la charge de l'appelant.
    pub const fn padding_vertical(self) -> u16 {
        match self {
            Self::Compacte => Esp::XS,
            Self::Normale => Esp::SM,
            Self::Confortable => Esp::MD,
        }
    }
}

/// Trois niveaux d'ombre seulement, teintés par la palette pour rester
/// crédibles en thème sombre où une ombre noire pure disparaît.
pub struct Ombre;

impl Ombre {
    /// Décollement discret d'une carte.
    pub fn carte(palette: Palette) -> Shadow {
        Shadow {
            color: avec_alpha(palette.ombre, palette.ombre.a * 0.6),
            offset: Vector::new(0.0, 1.0),
            blur_radius: 3.0,
        }
    }

    /// Élément flottant : menu, infobulle, notification.
    pub fn flottant(palette: Palette) -> Shadow {
        Shadow {
            color: palette.ombre,
            offset: Vector::new(0.0, 6.0),
            blur_radius: 18.0,
        }
    }

    /// Modale posée au-dessus du voile.
    pub fn modale(palette: Palette) -> Shadow {
        Shadow {
            color: avec_alpha(palette.ombre, (palette.ombre.a * 1.6).min(1.0)),
            offset: Vector::new(0.0, 16.0),
            blur_radius: 40.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::theme::palette::ThemeMode;

    #[test]
    fn lechelle_despacement_est_croissante() {
        let echelle = [
            Esp::XXS,
            Esp::XS,
            Esp::SM,
            Esp::MD,
            Esp::LG,
            Esp::XL,
            Esp::XXL,
            Esp::XXXL,
        ];
        for paire in echelle.windows(2) {
            assert!(paire[0] < paire[1]);
        }
    }

    #[test]
    fn lechelle_de_rayons_est_croissante() {
        let echelle = [Rayon::XS, Rayon::SM, Rayon::MD, Rayon::LG, Rayon::XL];
        for paire in echelle.windows(2) {
            assert!(paire[0] < paire[1]);
        }
    }

    /// Toute ligne doit rester au-dessus de la cible cliquable minimale.
    #[test]
    fn les_densites_respectent_la_cible_minimale() {
        for densite in [Densite::Compacte, Densite::Normale, Densite::Confortable] {
            assert!(densite.hauteur() >= 32.0);
        }
        assert!(Densite::Compacte.hauteur() < Densite::Normale.hauteur());
        assert!(Densite::Normale.hauteur() < Densite::Confortable.hauteur());
    }

    /// L'ombre doit grandir avec le niveau d'élévation, dans les deux thèmes.
    #[test]
    fn les_ombres_croissent_avec_lelevation() {
        for mode in [ThemeMode::Light, ThemeMode::Dark] {
            let p = Palette::pour(mode);
            assert!(Ombre::carte(p).blur_radius < Ombre::flottant(p).blur_radius);
            assert!(Ombre::flottant(p).blur_radius < Ombre::modale(p).blur_radius);
        }
    }
}
