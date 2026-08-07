//! Formulaire de transaction.
//!
//! Le montant est traité à part : c'est la donnée que l'utilisateur vient
//! réellement saisir, elle reçoit sa propre taille, sa propre police et le
//! focus à l'ouverture. Les erreurs apparaissent sous le champ concerné.

use iced::widget::{column, row, Space};
use iced::{Alignment, Element, Length};

use crate::app::message::Message;
use crate::app::state::AppState;
use crate::domaine::categorie::Category;
use crate::domaine::transaction::{TransactionKind, TransactionStatus};
use crate::modules::categories::composants::pastille::{self, Taille as TaillePastille};
use crate::ui::composants::bouton::{Bouton, Taille as TailleBouton, Variante};
use crate::ui::composants::calendrier::{calendrier, Actions as ActionsCalendrier};
use crate::ui::composants::champ;
use crate::ui::composants::icone::Icone;
use crate::ui::composants::modale::{Largeur, Modale};
use crate::ui::theme::espacements::Esp;
use crate::ui::theme::palette::Palette;

// ── Options des sélecteurs ────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
struct OptionCategorie {
    identifiant: String,
    nom: String,
}

impl std::fmt::Display for OptionCategorie {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.nom)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct OptionStatut(TransactionStatus);

impl std::fmt::Display for OptionStatut {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0.display_name())
    }
}

/// Intitulé de la modale selon le contexte.
pub fn titre(edition: bool, genre: TransactionKind) -> &'static str {
    match (edition, genre) {
        (true, TransactionKind::Income) => "Modifier le revenu",
        (true, TransactionKind::Expense) => "Modifier la dépense",
        (false, TransactionKind::Income) => "Nouveau revenu",
        (false, TransactionKind::Expense) => "Nouvelle dépense",
    }
}

/// Le formulaire, posé en modale sur la fenêtre.
pub fn modale<'a>(state: &'a AppState, fenetre: Element<'a, Message>) -> Element<'a, Message> {
    let palette = Palette::pour(state.theme_mode);
    let formulaire = &state.transaction_form;

    let mut modale = Modale::nouvelle(
        titre(formulaire.is_edit, formulaire.kind),
        corps(state, palette),
        Message::CloseTransactionForm,
        palette,
        state.mise_en_page,
    )
    .sous_titre("Les montants en attente alimentent ta prévision de fin de mois.")
    .largeur(Largeur::Moyenne);

    if formulaire.is_edit {
        if let Some(identifiant) = formulaire.edit_id.clone() {
            modale = modale.action_a_gauche(
                Bouton::nouveau("Supprimer", palette)
                    .variante(Variante::Destructif)
                    .avec_icone(Icone::Corbeille)
                    .sur_clic(Message::DeleteTransaction(identifiant))
                    .vue(),
            );
        }
    }

    modale
        .action(
            Bouton::nouveau("Annuler", palette)
                .variante(Variante::Discret)
                .sur_clic(Message::CloseTransactionForm)
                .vue(),
        )
        .action(
            Bouton::nouveau(
                if formulaire.recurrent && !formulaire.is_edit {
                    "Créer la récurrence"
                } else {
                    "Enregistrer"
                },
                palette,
            )
            .variante(Variante::Principal)
            .avec_icone(Icone::Coche)
            .sur_clic(Message::SubmitTransactionForm)
            .vue(),
        )
        .poser_sur(fenetre)
}

fn corps<'a>(state: &'a AppState, palette: Palette) -> Element<'a, Message> {
    let formulaire = &state.transaction_form;

    let categories: Vec<OptionCategorie> = state
        .categories
        .iter()
        .filter(|c| c.kind == formulaire.kind && c.is_active)
        .map(|c| OptionCategorie {
            identifiant: c.id.clone(),
            nom: c.name.clone(),
        })
        .collect();

    let categorie_choisie = categories
        .iter()
        .find(|option| option.identifiant == formulaire.category_id)
        .cloned();

    let categorie_courante: Option<&Category> =
        pastille::trouver(&state.categories, &formulaire.category_id);

    column![
        selecteur_de_sens(formulaire.kind, palette),
        champ::champ_montant(
            "Montant",
            &formulaire.amount_str,
            state.symbole_devise(),
            Message::SetFormAmount,
            Message::SubmitTransactionForm,
            formulaire.amount_error.as_deref(),
            palette,
        ),
        champ::champ(
            "Libellé",
            champ::saisie(
                &formulaire.label,
                "Ex : Courses au supermarché",
                Message::SetFormLabel,
                formulaire.label_error.is_some(),
                palette,
            ),
            None,
            formulaire.label_error.as_deref(),
            palette,
        ),
        row![
            champ::champ(
                "Date",
                row![
                    champ::saisie(
                        &formulaire.date_str,
                        "JJ/MM/AAAA",
                        Message::SetFormDate,
                        formulaire.date_error.is_some(),
                        palette,
                    )
                    .width(Length::Fill),
                    Bouton::icone(Icone::Calendrier, palette)
                        .sur_clic(Message::ToggleFormCalendar)
                        .vue(),
                ]
                .spacing(Esp::XS)
                .align_y(Alignment::Center),
                Some("JJ/MM/AAAA"),
                formulaire.date_error.as_deref(),
                palette,
            ),
            champ::champ(
                "Statut",
                champ::selecteur(
                    vec![
                        OptionStatut(TransactionStatus::Pending),
                        OptionStatut(TransactionStatus::Completed),
                    ],
                    Some(OptionStatut(formulaire.status)),
                    |choix: OptionStatut| Message::SetFormStatus(choix.0),
                    palette,
                )
                .width(Length::Fill),
                Some(aide_statut(formulaire.status)),
                None,
                palette,
            ),
        ]
        .spacing(Esp::MD),
        selecteur_de_date(formulaire, palette),
        champ::champ(
            "Catégorie",
            row![
                pastille::pastille(categorie_courante, TaillePastille::Grande, palette),
                champ::selecteur(
                    categories,
                    categorie_choisie,
                    |choix: OptionCategorie| Message::SetFormCategory(choix.identifiant),
                    palette,
                )
                .width(Length::Fill),
            ]
            .spacing(Esp::SM)
            .align_y(Alignment::Center),
            None,
            formulaire.category_error.as_deref(),
            palette,
        ),
        recurrence(formulaire, palette),
        champ::champ(
            "Note",
            champ::saisie(
                &formulaire.note,
                "Facultative",
                Message::SetFormNote,
                false,
                palette,
            ),
            None,
            None,
            palette,
        ),
    ]
    .spacing(Esp::LG)
    .width(Length::Fill)
    .into()
}

/// Bascule « répéter chaque mois ».
///
/// Absente en édition : modifier une occurrence ne doit pas transformer
/// rétroactivement l'historique en règle.
fn recurrence<'a>(
    formulaire: &'a crate::app::state::TransactionFormState,
    palette: Palette,
) -> Element<'a, Message> {
    if formulaire.is_edit {
        return Space::new(0, 0).into();
    }

    champ::interrupteur(
        "Répéter chaque mois",
        "Une occurrence sera créée automatiquement à l'ouverture de chaque mois.",
        formulaire.recurrent,
        Message::SetFormRecurrent,
        palette,
    )
}

/// Calendrier, déplié sous les champs date et statut.
fn selecteur_de_date<'a>(
    formulaire: &'a crate::app::state::TransactionFormState,
    palette: Palette,
) -> Element<'a, Message> {
    if !formulaire.calendrier_ouvert {
        return Space::new(0, 0).into();
    }

    calendrier(
        formulaire.calendrier_annee,
        formulaire.calendrier_mois,
        formulaire.date(),
        chrono::Utc::now().date_naive(),
        ActionsCalendrier {
            mois_precedent: Message::FormCalendarPreviousMonth,
            mois_suivant: Message::FormCalendarNextMonth,
            choisir: Message::PickFormDate,
        },
        palette,
    )
}

/// Bascule revenu / dépense, présentée comme deux choix explicites plutôt que
/// comme un champ parmi d'autres : c'est la première décision du formulaire.
fn selecteur_de_sens<'a>(genre: TransactionKind, palette: Palette) -> Element<'a, Message> {
    let bouton = |cible: TransactionKind, intitule: &'a str, symbole: Icone| {
        Bouton::nouveau(intitule, palette)
            .variante(if genre == cible {
                Variante::Principal
            } else {
                Variante::Secondaire
            })
            .taille(TailleBouton::Compacte)
            .avec_icone(symbole)
            .sur_clic(Message::SetFormKind(cible))
            .vue()
    };

    row![
        bouton(TransactionKind::Expense, "Dépense", Icone::FlecheSortante),
        bouton(TransactionKind::Income, "Revenu", Icone::FlecheEntrante),
        Space::with_width(Length::Fill),
    ]
    .spacing(Esp::SM)
    .align_y(Alignment::Center)
    .into()
}

/// Explication du statut : c'est lui qui décide si le montant entre ou non dans
/// la prévision de fin de mois.
pub fn aide_statut(statut: TransactionStatus) -> &'static str {
    match statut {
        TransactionStatus::Pending => "Compté dans la prévision",
        TransactionStatus::Completed => "Déjà sur le compte",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Chaque combinaison contexte/sens a son intitulé propre : l'utilisateur
    /// sait toujours ce qu'il est en train de faire.
    #[test]
    fn le_titre_depend_du_contexte_et_du_sens() {
        assert_eq!(titre(false, TransactionKind::Expense), "Nouvelle dépense");
        assert_eq!(titre(false, TransactionKind::Income), "Nouveau revenu");
        assert_eq!(titre(true, TransactionKind::Expense), "Modifier la dépense");
        assert_eq!(titre(true, TransactionKind::Income), "Modifier le revenu");

        let intitules = [
            titre(false, TransactionKind::Expense),
            titre(false, TransactionKind::Income),
            titre(true, TransactionKind::Expense),
            titre(true, TransactionKind::Income),
        ];
        let uniques: std::collections::HashSet<_> = intitules.iter().collect();
        assert_eq!(uniques.len(), intitules.len());
    }

    /// Les deux statuts doivent être expliqués différemment, sinon le champ
    /// reste incompréhensible.
    #[test]
    fn chaque_statut_est_explique() {
        assert_ne!(
            aide_statut(TransactionStatus::Pending),
            aide_statut(TransactionStatus::Completed)
        );
        assert!(!aide_statut(TransactionStatus::Pending).is_empty());
    }
}
