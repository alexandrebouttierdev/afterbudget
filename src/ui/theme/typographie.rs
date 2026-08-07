//! Rôles typographiques.
//!
//! Les vues ne choisissent jamais une taille ni une graisse : elles choisissent
//! un rôle. Iced n'embarque pas de fonte sur les plateformes natives, la
//! graisse dépend donc de la famille système — la hiérarchie repose en priorité
//! sur la taille, la couleur et l'espace, la graisse ne fait que la renforcer.

use iced::font::{Family, Weight};
use iced::widget::{text, Text};
use iced::{Color, Font};

/// Rôle typographique. L'ordre de la liste suit l'ordre d'importance.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    /// Le solde prévisionnel. Un seul par écran.
    MontantHeros,
    /// Titre d'écran.
    TitreEcran,
    /// Montant d'indicateur secondaire.
    MontantFort,
    /// Titre de bloc.
    TitreSection,
    /// Libellé de transaction, valeur mise en avant.
    CorpsFort,
    /// Texte courant.
    Corps,
    /// Étiquette de champ.
    Libelle,
    /// Métadonnée, aide, sous-titre.
    Legende,
    /// En-tête de colonne, badge.
    Micro,
}

impl Role {
    pub const fn taille(self) -> u16 {
        match self {
            Self::MontantHeros => 46,
            Self::TitreEcran => 24,
            Self::MontantFort => 22,
            Self::TitreSection => 16,
            Self::CorpsFort | Self::Corps => 14,
            Self::Libelle => 13,
            Self::Legende => 12,
            Self::Micro => 11,
        }
    }

    pub const fn graisse(self) -> Weight {
        match self {
            Self::MontantHeros => Weight::Light,
            Self::TitreEcran | Self::TitreSection => Weight::Semibold,
            Self::MontantFort | Self::CorpsFort | Self::Libelle | Self::Micro => Weight::Medium,
            Self::Corps | Self::Legende => Weight::Normal,
        }
    }
}

/// Fonte d'interface pour une graisse donnée.
pub const fn police(graisse: Weight) -> Font {
    Font {
        weight: graisse,
        ..Font::DEFAULT
    }
}

/// Fonte à chasse fixe, utilisée pour les colonnes de montants afin que les
/// chiffres restent alignés d'une ligne à l'autre.
pub const fn police_chiffres(graisse: Weight) -> Font {
    Font {
        family: Family::Monospace,
        weight: graisse,
        ..Font::DEFAULT
    }
}

/// Construit un texte dans un rôle donné.
pub fn texte<'a>(contenu: impl text::IntoFragment<'a>, role: Role) -> Text<'a> {
    text(contenu)
        .size(role.taille())
        .font(police(role.graisse()))
}

/// Idem, avec une couleur explicite.
pub fn texte_colore<'a>(
    contenu: impl text::IntoFragment<'a>,
    role: Role,
    couleur: Color,
) -> Text<'a> {
    texte(contenu, role).color(couleur)
}

/// Montant destiné à une colonne alignée : chasse fixe, couleur explicite.
pub fn montant_aligne<'a>(
    contenu: impl text::IntoFragment<'a>,
    role: Role,
    couleur: Color,
) -> Text<'a> {
    text(contenu)
        .size(role.taille())
        .font(police_chiffres(role.graisse()))
        .color(couleur)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// L'échelle doit rester strictement décroissante : c'est ce qui garantit
    /// qu'un rôle plus important est toujours visuellement plus fort.
    #[test]
    fn lechelle_est_decroissante() {
        let ordre = [
            Role::MontantHeros,
            Role::TitreEcran,
            Role::MontantFort,
            Role::TitreSection,
            Role::CorpsFort,
            Role::Libelle,
            Role::Legende,
            Role::Micro,
        ];
        for paire in ordre.windows(2) {
            assert!(
                paire[0].taille() > paire[1].taille(),
                "{:?} devrait être plus grand que {:?}",
                paire[0],
                paire[1]
            );
        }
    }

    /// Aucun texte ne descend sous 11 px, seuil de lisibilité retenu.
    #[test]
    fn aucune_taille_sous_le_seuil_de_lisibilite() {
        for role in [
            Role::MontantHeros,
            Role::TitreEcran,
            Role::MontantFort,
            Role::TitreSection,
            Role::CorpsFort,
            Role::Corps,
            Role::Libelle,
            Role::Legende,
            Role::Micro,
        ] {
            assert!(role.taille() >= 11, "{role:?} est trop petit");
        }
    }

    #[test]
    fn les_chiffres_alignes_utilisent_une_chasse_fixe() {
        assert_eq!(police_chiffres(Weight::Normal).family, Family::Monospace);
        assert_ne!(police(Weight::Normal).family, Family::Monospace);
    }
}
