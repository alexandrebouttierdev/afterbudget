//! Point d'entrée du rendu.
//!
//! Décide entre l'onboarding plein écran, le shell applicatif, et la couche
//! modale éventuellement posée par-dessus.

use crate::app::message::Message;
use crate::app::state::AppState;
use crate::modules::import_export::views as import_export_vues;
use crate::modules::parametres::views as parametres_vues;
use crate::modules::transactions::views as transactions_vues;
use crate::ui::layout;
use iced::Element;

pub fn view(state: &AppState) -> Element<'_, Message> {
    if !state
        .settings
        .as_ref()
        .is_some_and(|s| s.onboarding_completed)
    {
        return crate::modules::onboarding::views::index::view(state);
    }

    let fenetre = layout::fenetre(state);

    if state.show_transaction_form {
        return transactions_vues::formulaire::modale(state, fenetre);
    }

    if state.show_delete_confirm {
        if let Some(transaction) = state.delete_transaction.as_ref() {
            return transactions_vues::suppression::modale(state, transaction, fenetre);
        }
    }

    if state.show_import_confirm {
        return import_export_vues::modale_import(state, fenetre);
    }

    if state.show_reset_confirm {
        return parametres_vues::modale_reinitialisation(state, fenetre);
    }

    fenetre
}
