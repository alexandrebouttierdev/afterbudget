//! Règles d'opérations récurrentes.
//!
//! Une règle décrit une opération qui revient **chaque mois** : loyer, salaire,
//! abonnement. Elle ne remplace pas les transactions — à l'ouverture d'un mois,
//! elle y matérialise une transaction « en attente » que l'utilisateur reste
//! libre de corriger ou de supprimer sans toucher à la règle.

use chrono::NaiveDate;

use super::argent::Money;
use super::transaction::TransactionKind;
use crate::core::utils::jour_borne;

pub type RecurringRuleId = String;

#[derive(Debug, Clone, PartialEq)]
pub struct RecurringRule {
    pub id: RecurringRuleId,
    pub kind: TransactionKind,
    pub label: String,
    pub amount: Money,
    pub category_id: String,
    /// Jour du mois visé, de 1 à 31. Reporté au dernier jour d'un mois plus
    /// court : une règle au 31 tombe le 28 en février.
    pub day_of_month: u32,
    /// Premier mois couvert, inclus.
    pub start: (i32, u32),
    /// Dernier mois couvert, inclus. `None` signifie « sans fin ».
    pub end: Option<(i32, u32)>,
    pub note: Option<String>,
    pub is_active: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Rang absolu d'un mois, pour comparer deux couples (année, mois) sans piège.
fn rang(annee: i32, mois: u32) -> i32 {
    annee * 12 + mois as i32 - 1
}

impl RecurringRule {
    /// Vrai si la règle doit produire une occurrence dans ce mois.
    pub fn couvre(&self, annee: i32, mois: u32) -> bool {
        if !self.is_active || !(1..=12).contains(&mois) {
            return false;
        }
        let cible = rang(annee, mois);
        if cible < rang(self.start.0, self.start.1) {
            return false;
        }
        match self.end {
            Some((fin_annee, fin_mois)) => cible <= rang(fin_annee, fin_mois),
            None => true,
        }
    }

    /// Date de l'occurrence dans ce mois, jour reporté si le mois est plus court.
    pub fn date_pour(&self, annee: i32, mois: u32) -> Option<NaiveDate> {
        if !self.couvre(annee, mois) {
            return None;
        }
        NaiveDate::from_ymd_opt(annee, mois, jour_borne(annee, mois, self.day_of_month))
    }

    /// Intitulé de la périodicité, affiché à côté d'une règle.
    pub fn periodicite(&self) -> String {
        format!("Le {} de chaque mois", self.day_of_month)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn regle(jour: u32, debut: (i32, u32), fin: Option<(i32, u32)>) -> RecurringRule {
        RecurringRule {
            id: "r1".into(),
            kind: TransactionKind::Expense,
            label: "Loyer".into(),
            amount: Money::from_cents(89_000),
            category_id: "logement".into(),
            day_of_month: jour,
            start: debut,
            end: fin,
            note: None,
            is_active: true,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }

    #[test]
    fn une_regle_sans_fin_couvre_tous_les_mois_suivants() {
        let r = regle(3, (2026, 8), None);
        assert!(r.couvre(2026, 8));
        assert!(r.couvre(2026, 12));
        assert!(r.couvre(2030, 1));
    }

    /// Aucune occurrence avant le mois de départ : une règle créée en août ne
    /// doit pas remplir rétroactivement les mois précédents.
    #[test]
    fn une_regle_ne_remonte_pas_avant_son_debut() {
        let r = regle(3, (2026, 8), None);
        assert!(!r.couvre(2026, 7));
        assert!(!r.couvre(2025, 12));
        assert_eq!(r.date_pour(2026, 7), None);
    }

    #[test]
    fn une_regle_bornee_sarrete_a_sa_fin() {
        let r = regle(3, (2026, 8), Some((2026, 10)));
        assert!(r.couvre(2026, 8));
        assert!(r.couvre(2026, 10));
        assert!(!r.couvre(2026, 11));
    }

    /// Le passage d'année doit être géré par la comparaison, pas par le hasard.
    #[test]
    fn les_bornes_traversent_les_annees() {
        let r = regle(3, (2025, 11), Some((2026, 2)));
        assert!(r.couvre(2025, 12));
        assert!(r.couvre(2026, 1));
        assert!(r.couvre(2026, 2));
        assert!(!r.couvre(2026, 3));
        assert!(!r.couvre(2025, 10));
    }

    #[test]
    fn une_regle_inactive_ne_couvre_rien() {
        let mut r = regle(3, (2026, 8), None);
        r.is_active = false;
        assert!(!r.couvre(2026, 8));
        assert_eq!(r.date_pour(2026, 8), None);
    }

    /// Une règle au 31 doit tomber sur le dernier jour des mois plus courts,
    /// et non disparaître.
    #[test]
    fn un_jour_trop_grand_est_reporte_au_dernier_jour_du_mois() {
        let r = regle(31, (2024, 1), None);
        assert_eq!(r.date_pour(2026, 1), NaiveDate::from_ymd_opt(2026, 1, 31));
        assert_eq!(r.date_pour(2026, 2), NaiveDate::from_ymd_opt(2026, 2, 28));
        assert_eq!(r.date_pour(2026, 4), NaiveDate::from_ymd_opt(2026, 4, 30));
        // Année bissextile : le report tombe sur le 29.
        assert_eq!(r.date_pour(2024, 2), NaiveDate::from_ymd_opt(2024, 2, 29));
    }

    /// Toute règle couvrant un mois produit une date, sans exception.
    #[test]
    fn toute_couverture_produit_une_date() {
        let r = regle(31, (2026, 1), None);
        for mois in 1..=12 {
            assert!(r.couvre(2026, mois));
            assert!(r.date_pour(2026, mois).is_some(), "mois {mois}");
        }
    }

    #[test]
    fn un_mois_invalide_ne_couvre_rien() {
        let r = regle(3, (2026, 1), None);
        assert!(!r.couvre(2026, 0));
        assert!(!r.couvre(2026, 13));
    }
}
