//! Pastille de catégorie.
//!
//! Une catégorie est toujours représentée par **son icône dans un disque
//! teinté**, jamais par sa seule couleur : deux catégories de teintes voisines
//! restent distinguables, et une catégorie reste identifiable en nuances de gris.

use iced::widget::{container, row};
use iced::{Alignment, Element, Length};

use crate::domaine::categorie::Category;
use crate::ui::composants::icone::{icone, Icone, Taille as TailleIcone};
use crate::ui::theme::espacements::{Esp, Rayon};
use crate::ui::theme::palette::{couleur_categorie, Palette};
use crate::ui::theme::styles;
use crate::ui::theme::typographie::{texte_colore, Role};

/// Taille de la pastille.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Taille {
    /// Dans une ligne de liste.
    Ligne,
    /// Dans un formulaire ou une légende.
    Grande,
}

impl Taille {
    fn cote(self) -> f32 {
        match self {
            Self::Ligne => 32.0,
            Self::Grande => 40.0,
        }
    }

    fn icone(self) -> TailleIcone {
        match self {
            Self::Ligne => TailleIcone::Petite,
            Self::Grande => TailleIcone::Normale,
        }
    }
}

/// Disque teinté portant l'icône de la catégorie.
pub fn pastille<'a, Message: 'a>(
    categorie: Option<&Category>,
    taille: Taille,
    palette: Palette,
) -> Element<'a, Message> {
    let (symbole, teinte) = match categorie {
        Some(categorie) => (
            Icone::depuis_nom(&categorie.icon),
            couleur_categorie(&categorie.color, palette),
        ),
        None => (Icone::Points, palette.texte_doux),
    };

    container(icone(symbole, taille.icone(), teinte))
        .width(Length::Fixed(taille.cote()))
        .height(Length::Fixed(taille.cote()))
        .align_x(Alignment::Center)
        .align_y(Alignment::Center)
        .style(move |_theme| {
            styles::surface_nue(palette, palette.teinter_surface(teinte), Rayon::MD)
        })
        .into()
}

/// Pastille suivie du nom de la catégorie.
pub fn pastille_nommee<'a, Message: 'a>(
    categorie: Option<&Category>,
    taille: Taille,
    palette: Palette,
) -> Element<'a, Message> {
    let nom = categorie
        .map(|c| c.name.clone())
        .unwrap_or_else(|| "Sans catégorie".to_string());

    row![
        pastille(categorie, taille, palette),
        // Sans retour à la ligne : un nom long est coupé plutôt que de faire
        // varier la hauteur de la ligne et de casser l'alignement des colonnes.
        texte_colore(nom, Role::Corps, palette.texte).wrapping(iced::widget::text::Wrapping::None),
    ]
    .spacing(Esp::SM)
    .align_y(Alignment::Center)
    .into()
}

/// Retrouve une catégorie par identifiant.
pub fn trouver<'a>(categories: &'a [Category], identifiant: &str) -> Option<&'a Category> {
    categories.iter().find(|c| c.id == identifiant)
}

/// Nom affichable d'une catégorie, avec repli explicite.
pub fn nom(categories: &[Category], identifiant: &str) -> String {
    trouver(categories, identifiant)
        .map(|c| c.name.clone())
        .unwrap_or_else(|| "Sans catégorie".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domaine::transaction::TransactionKind;

    fn categorie(identifiant: &str, nom: &str) -> Category {
        Category {
            id: identifiant.into(),
            kind: TransactionKind::Expense,
            name: nom.into(),
            icon: "Home".into(),
            color: "#3B82F6".into(),
            sort_order: 0,
            is_default: true,
            is_active: true,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }

    #[test]
    fn une_categorie_connue_est_retrouvee() {
        let categories = vec![categorie("logement", "Logement")];
        assert_eq!(
            trouver(&categories, "logement").map(|c| c.name.as_str()),
            Some("Logement")
        );
        assert_eq!(nom(&categories, "logement"), "Logement");
    }

    /// Une transaction dont la catégorie a été supprimée doit rester lisible.
    #[test]
    fn une_categorie_absente_reste_lisible() {
        let categories: Vec<Category> = Vec::new();
        assert!(trouver(&categories, "inconnue").is_none());
        assert_eq!(nom(&categories, "inconnue"), "Sans catégorie");
    }

    #[test]
    fn les_tailles_sont_ordonnees() {
        assert!(Taille::Ligne.cote() < Taille::Grande.cote());
        assert!(Taille::Ligne.cote() >= 32.0, "cible cliquable minimale");
    }
}
