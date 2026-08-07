//! Aiguillage vers l'écran actif.
//!
//! Chaque écran fournit séparément son en-tête — fixe — et son corps — défilant.

use crate::app::message::{Message, Screen};
use crate::app::state::AppState;
use crate::modules::budget::views::index as accueil;
use crate::modules::parametres::views::index as parametres;
use crate::modules::statistiques::views::index as statistiques;
use crate::modules::transactions::views::index as transactions;
use iced::Element;

/// En-tête et corps de l'écran actif.
pub fn vue_ecran_actif(state: &AppState) -> (Element<'_, Message>, Element<'_, Message>) {
    match state.screen {
        Screen::Dashboard => (accueil::en_tete(state), accueil::corps(state)),
        Screen::Transactions => (transactions::en_tete(state), transactions::corps(state)),
        Screen::Statistics => (statistiques::en_tete(state), statistiques::corps(state)),
        Screen::Settings => (parametres::en_tete(state), parametres::corps(state)),
        // L'onboarding occupe toute la fenêtre et ne passe pas par le shell.
        Screen::Onboarding => (accueil::en_tete(state), accueil::corps(state)),
    }
}
