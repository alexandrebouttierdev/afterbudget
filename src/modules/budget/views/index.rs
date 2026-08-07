//! Tableau de bord.
//!
//! Hiérarchie voulue, de haut en bas et de gauche à droite :
//! 1. le solde prévisionnel, seul et hors échelle ;
//! 2. ce qui reste à encaisser et à payer, en indicateurs secondaires ;
//! 3. les derniers mouvements, en liste dense ;
//! 4. le récapitulatif du mois, en colonne latérale.
//!
//! Ce n'est délibérément pas une grille de cartes de poids égal.

use iced::widget::{column, container, row, Space};
use iced::{Element, Length};

use crate::app::message::{Message, Screen};
use crate::app::state::AppState;
use crate::domaine::budget::BudgetSummary;
use crate::domaine::transaction::{TransactionKind, TransactionStatus};
use crate::modules::budget::composants::bloc_solde::bloc_solde;
use crate::modules::budget::composants::indicateur::{indicateur, precision_de_comptage};
use crate::modules::transactions::composants::ligne_transaction as ligne;
use crate::ui::composants::bouton::{Bouton, Taille as TailleBouton, Variante};
use crate::ui::composants::carte;
use crate::ui::composants::champ;
use crate::ui::composants::en_tete_ecran::en_tete as en_tete_generique;
use crate::ui::composants::etat::{chargement, Etat};
use crate::ui::composants::icone::Icone;
use crate::ui::composants::selecteur_mois::{selecteur_mois, Actions};
use crate::ui::composants::separateur;
use crate::ui::theme::espacements::Esp;
use crate::ui::theme::palette::Palette;
use crate::ui::theme::typographie::{montant_aligne, texte_colore, Role};

pub fn en_tete(state: &AppState) -> Element<'_, Message> {
    let palette = Palette::pour(state.theme_mode);

    en_tete_generique(
        Screen::Dashboard.display_name(),
        Some(Screen::Dashboard.precision().into()),
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

    let Some(budget) = state.budget.as_ref() else {
        return chargement("Calcul de ton budget…", state.phase_chargement, palette);
    };

    let principal = column![
        carte::carte(
            column![
                bloc_solde(budget, palette),
                edition_rapide_du_solde(state, palette),
            ]
            .spacing(Esp::LG),
            carte::Variante::Heros,
            palette,
        ),
        indicateurs(state, budget, palette),
        derniers_mouvements(state, palette),
    ]
    .spacing(Esp::LG)
    .width(Length::Fill);

    let lateral = column![
        recapitulatif(budget, palette),
        repartition_rapide(state, palette)
    ]
    .spacing(Esp::LG)
    .width(Length::Fill);

    let contenu: Element<'_, Message> = if state.mise_en_page.tableau_de_bord_deux_colonnes() {
        row![
            container(principal).width(Length::FillPortion(64)),
            container(lateral).width(Length::FillPortion(36)),
        ]
        .spacing(Esp::LG)
        .into()
    } else {
        column![principal, lateral].spacing(Esp::LG).into()
    };

    column![
        contenu,
        Space::with_height(Length::Fixed(f32::from(Esp::XL))),
    ]
    .padding([Esp::LG, 0])
    .width(Length::Fill)
    .into()
}

/// Ce qui reste à encaisser et à payer d'ici la fin du mois.
fn indicateurs<'a>(
    state: &'a AppState,
    budget: &BudgetSummary,
    palette: Palette,
) -> Element<'a, Message> {
    let attendus = compter(state, TransactionKind::Income);
    let a_payer = compter(state, TransactionKind::Expense);

    let mut ligne = row![
        indicateur(
            "Reste à encaisser",
            budget.pending_income.format_fr(),
            precision_de_comptage(attendus),
            Icone::FlecheEntrante,
            palette.revenu,
            palette,
        ),
        indicateur(
            "Reste à payer",
            budget.pending_expenses.format_fr(),
            precision_de_comptage(a_payer),
            Icone::FlecheSortante,
            palette.depense,
            palette,
        ),
    ]
    .spacing(Esp::LG)
    .width(Length::Fill);

    // Le découvert n'est affiché que s'il en existe un : sinon l'indicateur
    // serait un zéro permanent sans information.
    if budget.overdraft_limit.is_positive() {
        let depassement = budget.depassement_du_decouvert();
        let (montant, precision, teinte) = if depassement.is_positive() {
            (
                depassement.format_fr(),
                "Découvert autorisé dépassé".to_string(),
                palette.danger,
            )
        } else {
            (
                budget.decouvert_restant().format_fr(),
                format!("{} déjà entamés", budget.decouvert_utilise().format_fr()),
                palette.attention,
            )
        };

        ligne = ligne.push(indicateur(
            if depassement.is_positive() {
                "Dépassement"
            } else {
                "Reste en découvert"
            },
            montant,
            precision,
            if depassement.is_positive() {
                Icone::Danger
            } else {
                Icone::Bouclier
            },
            teinte,
            palette,
        ));
    }

    ligne.into()
}

/// Édition du solde actuel sans quitter l'accueil.
///
/// Le solde est le point de départ de toute la prévision : devoir passer par
/// les paramètres pour le corriger après un relevé bancaire était un détour
/// injustifié. Ouvert à la demande pour ne pas alourdir l'écran au repos.
fn edition_rapide_du_solde(state: &AppState, palette: Palette) -> Element<'_, Message> {
    if !state.edition_solde_ouverte {
        return row![
            Bouton::nouveau("Corriger mon solde", palette)
                .variante(Variante::Discret)
                .taille(TailleBouton::Compacte)
                .avec_icone(Icone::Crayon)
                .sur_clic(Message::OpenBalanceEdit)
                .vue(),
            Space::with_width(Length::Fill),
        ]
        .into();
    }

    let mut bloc = column![
        texte_colore("Nouveau solde du compte", Role::Libelle, palette.texte),
        row![
            champ::saisie_montant(
                &state.settings_balance_str,
                "Ex : 1 240,80",
                Message::SetCurrentBalance,
                Message::UpdateBalance,
                state.settings_error.is_some(),
                palette,
            )
            .width(Length::Fixed(200.0)),
            Bouton::nouveau("Enregistrer", palette)
                .variante(Variante::Principal)
                .taille(TailleBouton::Compacte)
                .avec_icone(Icone::Coche)
                .sur_clic(Message::UpdateBalance)
                .vue(),
            Bouton::nouveau("Annuler", palette)
                .variante(Variante::Discret)
                .taille(TailleBouton::Compacte)
                .sur_clic(Message::CancelBalanceEdit)
                .vue(),
            Space::with_width(Length::Fill),
        ]
        .spacing(Esp::SM)
        .align_y(iced::Alignment::Center),
    ]
    .spacing(Esp::XS + 2);

    if let Some(erreur) = state.settings_error.as_deref() {
        bloc = bloc.push(champ::message_derreur(erreur, palette));
    } else {
        bloc = bloc.push(texte_colore(
            "Un solde négatif est accepté si tu es déjà à découvert.",
            Role::Legende,
            palette.texte_doux,
        ));
    }

    bloc.into()
}

/// Nombre de transactions encore en attente pour un sens donné.
pub fn compter(state: &AppState, genre: TransactionKind) -> usize {
    state
        .transactions
        .iter()
        .filter(|t| t.kind == genre && t.status == TransactionStatus::Pending)
        .count()
}

fn derniers_mouvements(state: &AppState, palette: Palette) -> Element<'_, Message> {
    let entete = row![
        texte_colore(
            "Derniers mouvements",
            Role::TitreSection,
            palette.texte_fort
        ),
        Space::with_width(Length::Fill),
        Bouton::nouveau("Tout voir", palette)
            .variante(Variante::Discret)
            .taille(TailleBouton::Compacte)
            .avec_icone(Icone::ChevronDroit)
            .sur_clic(Message::NavigateTo(Screen::Transactions))
            .vue(),
    ]
    .align_y(iced::Alignment::Center);

    if state.recent_transactions.is_empty() {
        return carte::carte(
            column![
                entete,
                Etat::nouveau(
                    Icone::Transactions,
                    "Aucun mouvement ce mois-ci",
                    "Ajoute tes revenus et tes charges prévues : la prévision de fin de mois se met à jour immédiatement.",
                    palette,
                )
                .avec_action(
                    Bouton::nouveau("Ajouter une dépense", palette)
                        .variante(Variante::Principal)
                        .avec_icone(Icone::Plus)
                        .sur_clic(Message::OpenAddExpense)
                        .vue(),
                )
                .vue(),
            ]
            .spacing(Esp::SM),
            carte::Variante::Plate,
            palette,
        )
        .into();
    }

    let mut lignes = column![].width(Length::Fill);
    for (index, transaction) in state.recent_transactions.iter().enumerate() {
        if index > 0 {
            lignes = lignes.push(separateur::horizontal(palette));
        }
        // L'aperçu de l'accueil vit dans une colonne étroite : le statut est
        // replié sous le libellé plutôt que de comprimer la colonne principale.
        lignes = lignes.push(ligne::ligne(transaction, &state.categories, false, palette));
    }

    carte::carte_liste(
        column![
            container(entete).padding([Esp::LG, Esp::LG]),
            separateur::horizontal(palette),
            lignes,
        ],
        palette,
    )
    .into()
}

/// Récapitulatif chiffré du mois, en colonne latérale : information secondaire,
/// donc format compact et typographie sobre.
fn recapitulatif<'a>(budget: &BudgetSummary, palette: Palette) -> Element<'a, Message> {
    let lignes: [(&str, String, iced::Color); 5] = [
        (
            "Revenus du mois",
            budget.total_income.format_fr(),
            palette.revenu,
        ),
        (
            "Dépenses du mois",
            budget.total_expenses.format_fr(),
            palette.depense,
        ),
        (
            "Déjà encaissé",
            budget.completed_income.format_fr(),
            palette.texte,
        ),
        (
            "Déjà payé",
            budget.completed_expenses.format_fr(),
            palette.texte,
        ),
        (
            "Découvert autorisé",
            budget.overdraft_limit.format_fr(),
            palette.texte,
        ),
    ];

    let mut bloc = column![texte_colore(
        "Récapitulatif du mois",
        Role::TitreSection,
        palette.texte_fort
    )]
    .spacing(Esp::MD)
    .width(Length::Fill);

    for (index, (intitule, montant, couleur)) in lignes.iter().enumerate() {
        if index == 2 || index == 4 {
            bloc = bloc.push(separateur::horizontal(palette));
        }
        bloc = bloc.push(
            row![
                texte_colore(*intitule, Role::Corps, palette.texte_doux),
                Space::with_width(Length::Fill),
                montant_aligne(montant.clone(), Role::Corps, *couleur),
            ]
            .align_y(iced::Alignment::Center),
        );
    }

    carte::carte(bloc, carte::Variante::Plate, palette).into()
}

/// Trois premières catégories de dépense du mois : un aperçu, pas un graphique.
fn repartition_rapide(state: &AppState, palette: Palette) -> Element<'_, Message> {
    let Some(statistiques) = state.monthly_statistics.as_ref() else {
        return Space::new(0, 0).into();
    };
    if statistiques.expenses_by_category.is_empty() {
        return Space::new(0, 0).into();
    }

    let mut bloc = column![row![
        texte_colore(
            "Principales catégories",
            Role::TitreSection,
            palette.texte_fort
        ),
        Space::with_width(Length::Fill),
        Bouton::nouveau("Détail", palette)
            .variante(Variante::Discret)
            .taille(TailleBouton::Compacte)
            .avec_icone(Icone::ChevronDroit)
            .sur_clic(Message::NavigateTo(Screen::Statistics))
            .vue(),
    ]
    .align_y(iced::Alignment::Center)]
    .spacing(Esp::MD)
    .width(Length::Fill);

    for statistique in statistiques.expenses_by_category.iter().take(3) {
        let couleur =
            crate::ui::theme::palette::couleur_categorie(&statistique.category_color, palette);
        bloc = bloc.push(
            column![
                row![
                    texte_colore(
                        statistique.category_name.as_str(),
                        Role::Corps,
                        palette.texte
                    ),
                    Space::with_width(Length::Fill),
                    montant_aligne(
                        statistique.total.format_fr(),
                        Role::Corps,
                        palette.texte_fort
                    ),
                ]
                .align_y(iced::Alignment::Center),
                crate::ui::composants::jauge::jauge(
                    (statistique.percentage / 100.0) as f32,
                    couleur,
                    palette
                ),
            ]
            .spacing(Esp::XS + 2),
        );
    }

    carte::carte(bloc, carte::Variante::Plate, palette).into()
}
