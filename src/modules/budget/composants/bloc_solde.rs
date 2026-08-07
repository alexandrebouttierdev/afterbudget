//! Bloc du solde prévisionnel.
//!
//! C'est **la** donnée centrale d'AfterBudget : elle occupe seule le haut de
//! l'écran, dans le seul rôle typographique hors échelle, accompagnée de son
//! état, de sa formule et d'une jauge de découvert.

use iced::widget::{column, container, row, Space};
use iced::{Alignment, Element, Length};

use crate::domaine::argent::Money;
use crate::domaine::budget::{BudgetSummary, FinancialStatus};
use crate::ui::composants::badge::{badge_icone, Ton};
use crate::ui::composants::icone::Icone;
use crate::ui::composants::jauge;
use crate::ui::theme::espacements::Esp;
use crate::ui::theme::palette::Palette;
use crate::ui::theme::typographie::{texte_colore, Role};

/// Traduction d'un état financier en variante visuelle.
///
/// Isolée et testée : c'est le pont entre la règle métier et l'apparence.
pub fn ton_de_letat(etat: FinancialStatus) -> Ton {
    match etat {
        FinancialStatus::Healthy => Ton::Succes,
        FinancialStatus::Warning => Ton::Attention,
        FinancialStatus::Danger => Ton::Danger,
    }
}

/// Icône associée à un état financier : le sens ne dépend pas de la couleur.
pub fn symbole_de_letat(etat: FinancialStatus) -> Icone {
    match etat {
        FinancialStatus::Healthy => Icone::Succes,
        FinancialStatus::Warning => Icone::Alerte,
        FinancialStatus::Danger => Icone::Danger,
    }
}

/// Couleur du montant prévisionnel.
pub fn couleur_du_montant(budget: &BudgetSummary, palette: Palette) -> iced::Color {
    if budget.projected_balance.is_negative() {
        palette.depense
    } else {
        palette.texte_fort
    }
}

/// Part du découvert autorisé qui serait consommée, entre 0 et 1.
///
/// Renvoie `None` lorsqu'aucun découvert n'est configuré ou que le solde reste
/// positif : la jauge n'a alors rien à montrer.
pub fn consommation_du_decouvert(budget: &BudgetSummary) -> Option<f32> {
    if budget.overdraft_limit.cents <= 0 || !budget.projected_balance.is_negative() {
        return None;
    }
    let utilise = -budget.projected_balance.cents as f32;
    let autorise = budget.overdraft_limit.cents as f32;
    Some((utilise / autorise).clamp(0.0, 1.0))
}

/// Intitulé de la jauge de découvert.
pub fn libelle_du_decouvert(budget: &BudgetSummary) -> String {
    if budget.remaining_overdraft_margin.is_negative() {
        format!(
            "Dépassement de {}",
            Money::from_cents(-budget.remaining_overdraft_margin.cents).format_fr()
        )
    } else {
        format!(
            "{} de découvert encore disponible",
            budget.remaining_overdraft_margin.format_fr()
        )
    }
}

/// Bloc complet.
pub fn bloc_solde<'a, Message: 'a>(
    budget: &BudgetSummary,
    palette: Palette,
) -> Element<'a, Message> {
    let ton = ton_de_letat(budget.financial_status);
    let teinte = ton.teinte(palette);

    let entete = row![
        texte_colore(
            "Ce qu'il te restera à la fin du mois",
            Role::Legende,
            palette.texte_doux,
        ),
        Space::with_width(Length::Fill),
        badge_icone(
            budget.financial_status.display_name(),
            symbole_de_letat(budget.financial_status),
            ton,
            palette,
        ),
    ]
    .align_y(Alignment::Center);

    let montant = texte_colore(
        budget.projected_balance.format_fr(),
        Role::MontantHeros,
        couleur_du_montant(budget, palette),
    );

    let formule = row![
        element_de_formule(
            "Solde actuel",
            budget.current_balance,
            palette.texte,
            palette
        ),
        signe("+", palette),
        element_de_formule("À recevoir", budget.pending_income, palette.revenu, palette),
        signe("\u{2212}", palette),
        element_de_formule("À payer", budget.pending_expenses, palette.depense, palette),
    ]
    .spacing(Esp::MD)
    .align_y(Alignment::Center);

    let mut bloc = column![
        entete,
        montant,
        texte_colore(
            budget
                .financial_status
                .display_message(budget.remaining_overdraft_margin),
            Role::Corps,
            palette.texte,
        ),
        Space::with_height(Length::Fixed(f32::from(Esp::XS))),
        formule,
    ]
    .spacing(Esp::SM)
    .width(Length::Fill);

    if let Some(consommation) = consommation_du_decouvert(budget) {
        bloc = bloc.push(Space::with_height(Length::Fixed(f32::from(Esp::SM))));
        bloc = bloc.push(
            column![
                jauge::jauge(consommation, teinte, palette),
                texte_colore(libelle_du_decouvert(budget), Role::Legende, teinte),
            ]
            .spacing(Esp::XS + 2),
        );
    }

    bloc.into()
}

fn element_de_formule<'a, Message: 'a>(
    intitule: &'a str,
    montant: Money,
    couleur: iced::Color,
    palette: Palette,
) -> Element<'a, Message> {
    container(
        column![
            texte_colore(intitule, Role::Micro, palette.texte_doux),
            texte_colore(montant.format_fr(), Role::Corps, couleur),
        ]
        .spacing(Esp::XXS),
    )
    .into()
}

fn signe<'a, Message: 'a>(symbole: &'a str, palette: Palette) -> Element<'a, Message> {
    texte_colore(symbole, Role::TitreSection, palette.texte_doux).into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::theme::palette::ThemeMode;

    fn budget(solde: i64, decouvert: i64, entrant: i64, sortant: i64) -> BudgetSummary {
        BudgetSummary::compute(
            Money::from_cents(solde),
            Money::from_cents(decouvert),
            Money::from_cents(entrant),
            Money::from_cents(sortant),
            Money::from_cents(entrant),
            Money::from_cents(sortant),
            Money::ZERO,
            Money::ZERO,
        )
    }

    /// Chaque état financier doit avoir un ton **et** une icône distincts.
    #[test]
    fn chaque_etat_a_son_ton_et_son_icone() {
        let etats = [
            FinancialStatus::Healthy,
            FinancialStatus::Warning,
            FinancialStatus::Danger,
        ];
        for (i, a) in etats.iter().enumerate() {
            for b in &etats[i + 1..] {
                assert_ne!(ton_de_letat(*a), ton_de_letat(*b));
                assert_ne!(symbole_de_letat(*a), symbole_de_letat(*b));
            }
        }
    }

    /// Un solde positif ne doit pas être peint en rouge, et inversement.
    #[test]
    fn la_couleur_du_montant_suit_le_signe() {
        for mode in [ThemeMode::Light, ThemeMode::Dark] {
            let p = Palette::pour(mode);
            assert_eq!(couleur_du_montant(&budget(50000, 0, 0, 0), p), p.texte_fort);
            assert_eq!(
                couleur_du_montant(&budget(0, 50000, 0, 10000), p),
                p.depense
            );
        }
    }

    /// Sans découvert configuré, ou avec un solde positif, la jauge n'a rien à
    /// montrer et ne doit pas s'afficher.
    #[test]
    fn la_jauge_ne_sallume_que_quand_elle_a_du_sens() {
        assert_eq!(
            consommation_du_decouvert(&budget(100000, 50000, 0, 0)),
            None
        );
        assert_eq!(consommation_du_decouvert(&budget(0, 0, 0, 10000)), None);
        assert!(consommation_du_decouvert(&budget(0, 50000, 0, 10000)).is_some());
    }

    /// La consommation est bien la part du découvert utilisée, et elle est
    /// plafonnée à 1 même en cas de dépassement.
    #[test]
    fn la_consommation_est_bornee() {
        // −200 € prévus sur 500 € autorisés = 40 %.
        let quarante = consommation_du_decouvert(&budget(0, 50000, 0, 20000)).unwrap();
        assert!((quarante - 0.4).abs() < 1e-5, "{quarante}");

        // −700 € prévus sur 500 € autorisés : plafonné.
        let depassement = consommation_du_decouvert(&budget(0, 50000, 0, 70000)).unwrap();
        assert_eq!(depassement, 1.0);
    }

    /// Le libellé doit annoncer un dépassement quand la marge est négative.
    #[test]
    fn le_libelle_annonce_le_depassement() {
        let sain = libelle_du_decouvert(&budget(0, 50000, 0, 20000));
        assert!(sain.contains("disponible"), "{sain}");

        let depasse = libelle_du_decouvert(&budget(0, 50000, 0, 70000));
        assert!(depasse.starts_with("Dépassement"), "{depasse}");
        assert!(depasse.contains("200,00"), "{depasse}");
    }
}
