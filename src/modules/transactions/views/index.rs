//! Écran Transactions.
//!
//! Un tableau dense : en-tête de colonnes, lignes groupées par jour, filets de
//! séparation, survol. Conçu pour parcourir beaucoup de lignes, pas pour
//! empiler des cartes.

use std::collections::BTreeMap;

use chrono::NaiveDate;
use iced::widget::{column, row, Space};
use iced::{Element, Length};

use crate::app::message::{Message, Screen};
use crate::app::state::AppState;
use crate::core::utils;
use crate::domaine::transaction::Transaction;
use crate::modules::transactions::composants::filtres;
use crate::modules::transactions::composants::ligne_transaction as ligne;
use crate::ui::composants::bouton::{Bouton, Variante};
use crate::ui::composants::carte;
use crate::ui::composants::en_tete_ecran::en_tete as en_tete_generique;
use crate::ui::composants::entete_colonnes::{entete, Colonne};
use crate::ui::composants::etat::Etat;
use crate::ui::composants::icone::Icone;
use crate::ui::composants::selecteur_mois::{selecteur_mois, Actions};
use crate::ui::composants::separateur;
use crate::ui::theme::espacements::Esp;
use crate::ui::theme::palette::Palette;
use crate::ui::theme::typographie::{texte_colore, Role};

pub fn en_tete(state: &AppState) -> Element<'_, Message> {
    let palette = Palette::pour(state.theme_mode);

    en_tete_generique(
        Screen::Transactions.display_name(),
        Some(Screen::Transactions.precision().into()),
        Some(selecteur_mois(
            state.current_year,
            state.current_month,
            state.is_current_month(),
            Actions {
                precedent: Message::PreviousMonth,
                suivant: Message::NextMonth,
                aujourdhui: Message::GoToCurrentMonth,
            },
            palette,
        )),
        vec![
            Bouton::nouveau("Revenu", palette)
                .variante(Variante::Secondaire)
                .avec_icone(Icone::FlecheEntrante)
                .sur_clic(Message::OpenAddIncome)
                .vue(),
            Bouton::nouveau("Dépense", palette)
                .variante(Variante::Principal)
                .avec_icone(Icone::Plus)
                .sur_clic(Message::OpenAddExpense)
                .vue(),
        ],
        palette,
    )
}

pub fn corps(state: &AppState) -> Element<'_, Message> {
    let palette = Palette::pour(state.theme_mode);

    let liste: Element<'_, Message> = if state.transactions.is_empty() {
        carte::carte(etat_vide(state, palette), carte::Variante::Plate, palette).into()
    } else {
        tableau(state, palette)
    };

    column![
        filtres::barre(state),
        row![
            filtres::resume(state.transactions.len(), state.filtres_actifs(), palette),
            Space::with_width(Length::Fill),
            total_affiche(state, palette),
        ],
        liste,
        Space::with_height(Length::Fixed(f32::from(Esp::XL))),
    ]
    .spacing(Esp::LG)
    .padding([Esp::LG, 0])
    .width(Length::Fill)
    .into()
}

/// Somme signée des lignes actuellement affichées : donne du sens au filtrage.
fn total_affiche(state: &AppState, palette: Palette) -> Element<'_, Message> {
    if state.transactions.is_empty() {
        return Space::new(0, 0).into();
    }

    let total = crate::modules::transactions::service::solde_des(&state.transactions);
    let couleur = if total.is_negative() {
        palette.depense
    } else {
        palette.revenu
    };

    row![
        texte_colore("Total affiché ", Role::Legende, palette.texte_doux),
        texte_colore(total.format_fr_signed(), Role::Legende, couleur),
    ]
    .into()
}

fn tableau(state: &AppState, palette: Palette) -> Element<'_, Message> {
    let colonne_statut = state.mise_en_page.colonne_statut_visible();

    let mut colonnes = vec![
        Colonne::flexible("Transaction", ligne::PORTION_LIBELLE),
        Colonne::flexible("Catégorie", ligne::PORTION_CATEGORIE),
        Colonne::fixe("Date", ligne::LARGEUR_DATE),
    ];
    if colonne_statut {
        colonnes.push(Colonne::fixe("Statut", ligne::LARGEUR_STATUT));
    }
    colonnes.push(Colonne::fixe("Montant", ligne::LARGEUR_MONTANT).a_droite());
    colonnes.push(Colonne::fixe("", ligne::LARGEUR_ACTIONS));

    let mut corps = column![].width(Length::Fill);
    let mut premier_groupe = true;

    for (jour, transactions) in grouper_par_jour(&state.transactions) {
        if !premier_groupe {
            corps = corps.push(separateur::horizontal(palette));
        }
        premier_groupe = false;

        corps = corps.push(intitule_de_jour(jour, palette));

        for (index, transaction) in transactions.iter().enumerate() {
            if index > 0 {
                corps = corps.push(separateur::horizontal(palette));
            }
            corps = corps.push(ligne::ligne(
                transaction,
                &state.categories,
                colonne_statut,
                palette,
            ));
        }
    }

    carte::carte_liste(
        column![
            entete(colonnes, palette),
            separateur::horizontal(palette),
            corps,
        ],
        palette,
    )
    .into()
}

fn intitule_de_jour<'a>(jour: NaiveDate, palette: Palette) -> Element<'a, Message> {
    iced::widget::container(texte_colore(
        utils::jour_long(&jour),
        Role::Micro,
        palette.texte_doux,
    ))
    .width(Length::Fill)
    .padding([Esp::SM, Esp::LG])
    .style(move |_theme| crate::ui::theme::styles::surface_nue(palette, palette.surface_basse, 0.0))
    .into()
}

/// Regroupe les transactions par jour, du plus récent au plus ancien.
///
/// Extrait de la vue pour rester testable : c'est la structure de la liste, pas
/// sa décoration.
pub fn grouper_par_jour(transactions: &[Transaction]) -> Vec<(NaiveDate, Vec<&Transaction>)> {
    let mut groupes: BTreeMap<NaiveDate, Vec<&Transaction>> = BTreeMap::new();
    for transaction in transactions {
        groupes
            .entry(transaction.transaction_date)
            .or_default()
            .push(transaction);
    }
    groupes.into_iter().rev().collect()
}

fn etat_vide(state: &AppState, palette: Palette) -> Element<'_, Message> {
    if state.filtres_actifs() {
        Etat::nouveau(
            Icone::Recherche,
            "Aucune transaction ne correspond",
            "Élargis la recherche ou remets les filtres à zéro pour revoir le mois complet.",
            palette,
        )
        .avec_action(
            Bouton::nouveau("Réinitialiser les filtres", palette)
                .variante(Variante::Secondaire)
                .avec_icone(Icone::Croix)
                .sur_clic(Message::ClearFilters)
                .vue(),
        )
        .vue()
    } else {
        Etat::nouveau(
            Icone::Transactions,
            "Ce mois est encore vide",
            "Ajoute une première dépense ou un revenu pour voir apparaître ta prévision de fin de mois.",
            palette,
        )
        .avec_action(
            row![
                Bouton::nouveau("Ajouter un revenu", palette)
                    .variante(Variante::Secondaire)
                    .avec_icone(Icone::FlecheEntrante)
                    .sur_clic(Message::OpenAddIncome)
                    .vue(),
                Bouton::nouveau("Ajouter une dépense", palette)
                    .variante(Variante::Principal)
                    .avec_icone(Icone::Plus)
                    .sur_clic(Message::OpenAddExpense)
                    .vue(),
            ]
            .spacing(Esp::SM),
        )
        .vue()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domaine::argent::Money;
    use crate::domaine::transaction::{TransactionKind, TransactionStatus};

    fn transaction(identifiant: &str, jour: u32) -> Transaction {
        Transaction {
            id: identifiant.into(),
            kind: TransactionKind::Expense,
            label: identifiant.into(),
            amount: Money::from_cents(1000),
            transaction_date: NaiveDate::from_ymd_opt(2026, 8, jour).unwrap(),
            status: TransactionStatus::Pending,
            category_id: "alimentation".into(),
            note: None,
            recurring_rule_id: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }

    #[test]
    fn une_liste_vide_ne_produit_aucun_groupe() {
        assert!(grouper_par_jour(&[]).is_empty());
    }

    /// Les jours doivent apparaître du plus récent au plus ancien, comme dans
    /// un relevé bancaire.
    #[test]
    fn les_jours_sont_ordonnes_du_plus_recent_au_plus_ancien() {
        let transactions = vec![
            transaction("a", 3),
            transaction("b", 17),
            transaction("c", 9),
        ];
        let groupes = grouper_par_jour(&transactions);

        let jours: Vec<u32> = groupes
            .iter()
            .map(|(jour, _)| chrono::Datelike::day(jour))
            .collect();
        assert_eq!(jours, vec![17, 9, 3]);
    }

    /// Les transactions d'un même jour restent regroupées, et aucune n'est
    /// perdue en chemin.
    #[test]
    fn les_transactions_dun_meme_jour_sont_regroupees() {
        let transactions = vec![
            transaction("a", 12),
            transaction("b", 12),
            transaction("c", 5),
        ];
        let groupes = grouper_par_jour(&transactions);

        assert_eq!(groupes.len(), 2);
        assert_eq!(groupes[0].1.len(), 2);
        assert_eq!(groupes[1].1.len(), 1);

        let total: usize = groupes.iter().map(|(_, lignes)| lignes.len()).sum();
        assert_eq!(total, transactions.len());
    }
}
