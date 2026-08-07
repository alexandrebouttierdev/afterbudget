//! Confirmation de suppression d'une transaction.
//!
//! L'action sûre reste le bouton discret de gauche ; l'action irréversible est
//! isolée à droite et porte la couleur de danger.

use iced::widget::column;
use iced::Element;

use crate::app::message::Message;
use crate::app::state::AppState;
use crate::domaine::transaction::Transaction;
use crate::modules::categories::composants::pastille;
use crate::modules::transactions::composants::ligne_transaction;
use crate::ui::composants::bouton::{Bouton, Variante};
use crate::ui::composants::icone::Icone;
use crate::ui::composants::modale::{Largeur, Modale};
use crate::ui::theme::espacements::Esp;
use crate::ui::theme::palette::Palette;
use crate::ui::theme::typographie::{texte_colore, Role};

/// Résumé de ce qui va être supprimé : montant, libellé, catégorie et date.
pub fn recapitulatif(transaction: &Transaction, nom_categorie: &str) -> String {
    format!(
        "{} · {} · {}",
        ligne_transaction::montant_signe(transaction),
        nom_categorie,
        crate::core::utils::format_date_fr(&transaction.transaction_date)
    )
}

pub fn modale<'a>(
    state: &'a AppState,
    transaction: &'a Transaction,
    fenetre: Element<'a, Message>,
) -> Element<'a, Message> {
    let palette = Palette::pour(state.theme_mode);
    let nom_categorie = pastille::nom(&state.categories, &transaction.category_id);

    let corps = column![
        texte_colore(
            transaction.label.as_str(),
            Role::TitreSection,
            palette.texte_fort
        ),
        texte_colore(
            recapitulatif(transaction, &nom_categorie),
            Role::Corps,
            palette.texte_doux
        ),
        texte_colore(
            "Cette transaction sera définitivement retirée du mois et de tes statistiques.",
            Role::Corps,
            palette.texte
        ),
    ]
    .spacing(Esp::SM);

    Modale::nouvelle(
        "Supprimer cette transaction ?",
        corps,
        Message::CancelDelete,
        palette,
        state.mise_en_page,
    )
    .sous_titre("Action irréversible")
    .largeur(Largeur::Petite)
    .action(
        Bouton::nouveau("Annuler", palette)
            .variante(Variante::Discret)
            .sur_clic(Message::CancelDelete)
            .vue(),
    )
    .action(
        Bouton::nouveau("Supprimer", palette)
            .variante(Variante::Destructif)
            .avec_icone(Icone::Corbeille)
            .sur_clic(Message::ConfirmDeleteTransaction)
            .vue(),
    )
    .poser_sur(fenetre)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domaine::argent::Money;
    use crate::domaine::transaction::{TransactionKind, TransactionStatus};
    use chrono::NaiveDate;

    #[test]
    fn le_recapitulatif_reprend_montant_categorie_et_date() {
        let transaction = Transaction {
            id: "t1".into(),
            kind: TransactionKind::Expense,
            label: "Loyer".into(),
            amount: Money::from_cents(89000),
            transaction_date: NaiveDate::from_ymd_opt(2026, 8, 3).unwrap(),
            status: TransactionStatus::Completed,
            category_id: "logement".into(),
            note: None,
            recurring_rule_id: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let resume = recapitulatif(&transaction, "Logement");
        assert!(resume.contains("890,00"));
        assert!(resume.contains("Logement"));
        assert!(resume.contains("03/08/2026"));
        assert!(resume.starts_with('\u{2212}'), "{resume}");
    }
}
