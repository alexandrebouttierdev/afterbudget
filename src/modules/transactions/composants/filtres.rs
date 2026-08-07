//! Barre de filtres.
//!
//! Recherche, sens, statut et catégorie. Les options ne sont plus fuitées à
//! chaque rendu : elles sont construites en vecteurs possédés, que `pick_list`
//! accepte directement.

use iced::widget::{column, container, row, Space};
use iced::{Alignment, Element, Length};

use crate::app::message::Message;
use crate::app::state::AppState;
use crate::domaine::categorie::Category;
use crate::domaine::transaction::{TransactionKind, TransactionStatus};
use crate::ui::composants::bouton::{Bouton, Taille as TailleBouton, Variante};
use crate::ui::composants::champ;
use crate::ui::composants::icone::Icone;
use crate::ui::theme::espacements::{Esp, Rayon};
use crate::ui::theme::palette::Palette;
use crate::ui::theme::styles::{self, Elevation};
use crate::ui::theme::typographie::{texte_colore, Role};

// ── Options des sélecteurs ────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OptionSens {
    Tous,
    Revenus,
    Depenses,
}

impl OptionSens {
    fn toutes() -> Vec<Self> {
        vec![Self::Tous, Self::Revenus, Self::Depenses]
    }

    pub fn depuis(filtre: Option<TransactionKind>) -> Self {
        match filtre {
            None => Self::Tous,
            Some(TransactionKind::Income) => Self::Revenus,
            Some(TransactionKind::Expense) => Self::Depenses,
        }
    }

    pub fn vers(&self) -> Option<TransactionKind> {
        match self {
            Self::Tous => None,
            Self::Revenus => Some(TransactionKind::Income),
            Self::Depenses => Some(TransactionKind::Expense),
        }
    }
}

impl std::fmt::Display for OptionSens {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Tous => "Tous les sens",
            Self::Revenus => "Revenus",
            Self::Depenses => "Dépenses",
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OptionStatut {
    Tous,
    EnAttente,
    Realises,
}

impl OptionStatut {
    fn toutes() -> Vec<Self> {
        vec![Self::Tous, Self::EnAttente, Self::Realises]
    }

    pub fn depuis(filtre: Option<TransactionStatus>) -> Self {
        match filtre {
            None => Self::Tous,
            Some(TransactionStatus::Pending) => Self::EnAttente,
            Some(TransactionStatus::Completed) => Self::Realises,
        }
    }

    pub fn vers(&self) -> Option<TransactionStatus> {
        match self {
            Self::Tous => None,
            Self::EnAttente => Some(TransactionStatus::Pending),
            Self::Realises => Some(TransactionStatus::Completed),
        }
    }
}

impl std::fmt::Display for OptionStatut {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Tous => "Tous les statuts",
            Self::EnAttente => "En attente",
            Self::Realises => "Réalisés",
        })
    }
}

#[derive(Debug, Clone)]
pub enum OptionCategorie {
    Toutes,
    Une { identifiant: String, nom: String },
}

impl OptionCategorie {
    pub fn toutes(categories: &[Category]) -> Vec<Self> {
        let mut options = vec![Self::Toutes];
        options.extend(
            categories
                .iter()
                .filter(|c| c.is_active)
                .map(|c| Self::Une {
                    identifiant: c.id.clone(),
                    nom: c.name.clone(),
                }),
        );
        options
    }

    pub fn depuis(categories: &[Category], filtre: Option<&str>) -> Self {
        match filtre {
            None => Self::Toutes,
            Some(identifiant) => categories
                .iter()
                .find(|c| c.id == identifiant)
                .map(|c| Self::Une {
                    identifiant: c.id.clone(),
                    nom: c.name.clone(),
                })
                .unwrap_or(Self::Toutes),
        }
    }

    pub fn vers(&self) -> Option<String> {
        match self {
            Self::Toutes => None,
            Self::Une { identifiant, .. } => Some(identifiant.clone()),
        }
    }
}

impl PartialEq for OptionCategorie {
    fn eq(&self, autre: &Self) -> bool {
        match (self, autre) {
            (Self::Toutes, Self::Toutes) => true,
            (Self::Une { identifiant: a, .. }, Self::Une { identifiant: b, .. }) => a == b,
            _ => false,
        }
    }
}

impl Eq for OptionCategorie {}

impl std::fmt::Display for OptionCategorie {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Toutes => f.write_str("Toutes les catégories"),
            Self::Une { nom, .. } => f.write_str(nom),
        }
    }
}

// ── Rendu ─────────────────────────────────────────────────────────────────

/// Barre de filtres complète. Se replie sur deux lignes lorsque la fenêtre
/// n'est pas assez large, plutôt que de déborder horizontalement.
pub fn barre(state: &AppState) -> Element<'_, Message> {
    let palette = Palette::pour(state.theme_mode);
    let sur_une_ligne = state.mise_en_page.filtres_sur_une_ligne();

    let recherche = champ::champ_recherche(&state.search_query, Message::SetSearchQuery, palette);

    let sens = champ::selecteur(
        OptionSens::toutes(),
        Some(OptionSens::depuis(state.filter_kind)),
        |choix: OptionSens| Message::SetFilterKind(choix.vers()),
        palette,
    )
    .width(Length::Fixed(160.0));

    let statut = champ::selecteur(
        OptionStatut::toutes(),
        Some(OptionStatut::depuis(state.filter_status)),
        |choix: OptionStatut| Message::SetFilterStatus(choix.vers()),
        palette,
    )
    .width(Length::Fixed(160.0));

    let categorie = champ::selecteur(
        OptionCategorie::toutes(&state.categories),
        Some(OptionCategorie::depuis(
            &state.categories,
            state.filter_category.as_deref(),
        )),
        |choix: OptionCategorie| Message::SetFilterCategory(choix.vers()),
        palette,
    )
    .width(Length::Fixed(200.0));

    let mut reinitialiser = Bouton::nouveau("Réinitialiser", palette)
        .variante(Variante::Discret)
        .taille(TailleBouton::Compacte)
        .avec_icone(Icone::Croix);
    if state.filtres_actifs() {
        reinitialiser = reinitialiser.sur_clic(Message::ClearFilters);
    }

    let selecteurs = row![sens, statut, categorie]
        .spacing(Esp::SM)
        .align_y(Alignment::Center);

    let contenu: Element<'_, Message> = if sur_une_ligne {
        row![
            recherche,
            selecteurs,
            Space::with_width(Length::Fill),
            reinitialiser.vue(),
        ]
        .spacing(Esp::MD)
        .align_y(Alignment::Center)
        .into()
    } else {
        column![
            row![
                recherche,
                Space::with_width(Length::Fill),
                reinitialiser.vue()
            ]
            .spacing(Esp::MD)
            .align_y(Alignment::Center),
            selecteurs,
        ]
        .spacing(Esp::SM)
        .into()
    };

    container(contenu)
        .width(Length::Fill)
        .padding(Esp::MD)
        .style(move |_theme| styles::surface(palette, Elevation::Plate, Rayon::LG))
        .into()
}

/// Libellé du décompte de résultats. Distingue « rien à afficher » de « rien
/// ne correspond », et accorde le singulier.
pub fn libelle_resume(nombre: usize, filtres_actifs: bool) -> String {
    match (nombre, filtres_actifs) {
        (0, true) => "Aucun résultat".to_string(),
        (0, false) => "Aucune transaction".to_string(),
        (1, true) => "1 résultat".to_string(),
        (1, false) => "1 transaction".to_string(),
        (n, true) => format!("{n} résultats"),
        (n, false) => format!("{n} transactions"),
    }
}

/// Résumé textuel du nombre de résultats.
pub fn resume(nombre: usize, filtres_actifs: bool, palette: Palette) -> Element<'static, Message> {
    texte_colore(
        libelle_resume(nombre, filtres_actifs),
        Role::Legende,
        palette.texte_doux,
    )
    .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn les_options_de_sens_font_laller_retour() {
        for filtre in [
            None,
            Some(TransactionKind::Income),
            Some(TransactionKind::Expense),
        ] {
            assert_eq!(OptionSens::depuis(filtre).vers(), filtre);
        }
    }

    #[test]
    fn les_options_de_statut_font_laller_retour() {
        for filtre in [
            None,
            Some(TransactionStatus::Pending),
            Some(TransactionStatus::Completed),
        ] {
            assert_eq!(OptionStatut::depuis(filtre).vers(), filtre);
        }
    }

    /// Une catégorie disparue ne doit pas laisser un filtre fantôme dans le
    /// sélecteur.
    #[test]
    fn une_categorie_disparue_revient_a_toutes() {
        let categories: Vec<Category> = Vec::new();
        assert_eq!(
            OptionCategorie::depuis(&categories, Some("supprimee")),
            OptionCategorie::Toutes
        );
        assert_eq!(OptionCategorie::depuis(&categories, None).vers(), None);
    }

    /// Le résumé doit distinguer « rien à afficher » de « rien ne correspond »,
    /// et accorder correctement le singulier.
    #[test]
    fn le_resume_est_explicite_et_accorde() {
        assert_eq!(libelle_resume(0, false), "Aucune transaction");
        assert_eq!(libelle_resume(0, true), "Aucun résultat");
        assert_eq!(libelle_resume(1, false), "1 transaction");
        assert_eq!(libelle_resume(1, true), "1 résultat");
        assert_eq!(libelle_resume(17, false), "17 transactions");
        assert_eq!(libelle_resume(17, true), "17 résultats");
    }
}
