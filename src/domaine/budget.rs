use super::argent::Money;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FinancialStatus {
    Healthy,
    Warning,
    Danger,
}

impl FinancialStatus {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Healthy => "En forme",
            Self::Warning => "Attention",
            Self::Danger => "Danger",
        }
    }

    pub fn display_message(&self, margin: Money) -> String {
        match self {
            Self::Healthy => "Tu devrais terminer le mois avec un solde positif.".into(),
            Self::Warning => format!(
                "Tu devrais utiliser une partie de ton découvert. Il te reste {} de marge.",
                margin.format_fr()
            ),
            Self::Danger => {
                let over = Money::from_cents(-margin.cents);
                format!(
                    "Tu risques de dépasser ton découvert autorisé de {}.",
                    over.format_fr()
                )
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct BudgetSummary {
    pub current_balance: Money,
    pub pending_income: Money,
    pub pending_expenses: Money,
    pub projected_balance: Money,
    pub overdraft_limit: Money,
    pub remaining_overdraft_margin: Money,
    pub financial_status: FinancialStatus,
    pub total_income: Money,
    pub total_expenses: Money,
    pub completed_income: Money,
    pub completed_expenses: Money,
}

impl BudgetSummary {
    #[allow(clippy::too_many_arguments)]
    pub fn compute(
        current_balance: Money,
        overdraft_limit: Money,
        pending_income: Money,
        pending_expenses: Money,
        total_income: Money,
        total_expenses: Money,
        completed_income: Money,
        completed_expenses: Money,
    ) -> Result<Self, String> {
        let err = || "Dépassement de montant dans le calcul du budget.".to_string();
        let projected_balance = current_balance
            .checked_add(pending_income)
            .ok_or_else(err)?
            .checked_sub(pending_expenses)
            .ok_or_else(err)?;
        let remaining_overdraft_margin = projected_balance
            .checked_add(overdraft_limit)
            .ok_or_else(err)?;

        let financial_status = if projected_balance >= Money::ZERO {
            FinancialStatus::Healthy
        } else if projected_balance >= Money::from_cents(overdraft_limit.cents.saturating_neg()) {
            FinancialStatus::Warning
        } else {
            FinancialStatus::Danger
        };

        Ok(Self {
            current_balance,
            pending_income,
            pending_expenses,
            projected_balance,
            overdraft_limit,
            remaining_overdraft_margin,
            financial_status,
            total_income,
            total_expenses,
            completed_income,
            completed_expenses,
        })
    }
}

impl BudgetSummary {
    /// Part du découvert autorisé encore disponible en fin de mois.
    ///
    /// Tant que le solde prévisionnel reste positif, la totalité du découvert
    /// est disponible. Dès qu'il passe sous zéro, la part consommée est
    /// déduite. En cas de dépassement, le reste est nul — jamais négatif :
    /// l'ampleur du dépassement est portée par `depassement_du_decouvert`.
    pub fn decouvert_restant(&self) -> Money {
        if self.projected_balance.is_negative() {
            Money::from_cents(
                self.overdraft_limit
                    .cents
                    .saturating_add(self.projected_balance.cents)
                    .max(0),
            )
        } else {
            self.overdraft_limit
        }
    }

    /// Part du découvert autorisé qui serait consommée, plafonnée à la limite.
    pub fn decouvert_utilise(&self) -> Money {
        if self.projected_balance.is_negative() {
            Money::from_cents(
                self.projected_balance
                    .cents
                    .saturating_neg()
                    .min(self.overdraft_limit.cents),
            )
        } else {
            Money::ZERO
        }
    }

    /// Montant dépassant le découvert autorisé. Nul tant qu'on reste dedans.
    pub fn depassement_du_decouvert(&self) -> Money {
        Money::from_cents(
            self.remaining_overdraft_margin
                .cents
                .saturating_neg()
                .max(0),
        )
    }
}

#[derive(Debug, Clone)]
pub struct CategoryStats {
    pub category_id: String,
    pub category_name: String,
    pub category_color: String,
    pub total: Money,
    pub percentage: f64,
    pub count: usize,
}

pub struct MonthlyStatistics {
    pub total_income: Money,
    pub total_expenses: Money,
    pub balance: Money,
    pub completed_income: Money,
    pub pending_income: Money,
    pub completed_expenses: Money,
    pub pending_expenses: Money,
    pub transaction_count: usize,
    pub income_count: usize,
    pub expense_count: usize,
    pub expenses_by_category: Vec<CategoryStats>,
    pub income_by_category: Vec<CategoryStats>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domaine::argent::Money;

    #[test]
    fn test_solde_positif() {
        let summary = BudgetSummary::compute(
            Money::from_cents(30000),  // solde actuel
            Money::from_cents(20000),  // découvert
            Money::from_cents(150000), // revenus en attente
            Money::from_cents(120000), // dépenses en attente
            Money::from_cents(150000), // total revenus
            Money::from_cents(120000), // total dépenses
            Money::ZERO,               // complétés income
            Money::ZERO,               // complétés expenses
        )
        .unwrap();
        assert_eq!(summary.projected_balance.cents, 60000); // 300 + 1500 - 1200 = 600
        assert_eq!(summary.financial_status, FinancialStatus::Healthy);
    }

    #[test]
    fn test_utilisation_decouvert() {
        let summary = BudgetSummary::compute(
            Money::from_cents(-36000),
            Money::from_cents(50000),
            Money::from_cents(120700),
            Money::from_cents(132500),
            Money::from_cents(120700),
            Money::from_cents(132500),
            Money::ZERO,
            Money::ZERO,
        )
        .unwrap();
        assert_eq!(summary.projected_balance.cents, -47800); // -360 + 1207 - 1325 = -478
        assert_eq!(summary.financial_status, FinancialStatus::Warning);
        assert_eq!(summary.remaining_overdraft_margin.cents, 2200); // -478 + 500 = 22
    }

    #[test]
    fn test_depassement_decouvert() {
        let summary = BudgetSummary::compute(
            Money::from_cents(-40000),
            Money::from_cents(50000),
            Money::from_cents(20000),
            Money::from_cents(50000),
            Money::from_cents(20000),
            Money::from_cents(50000),
            Money::ZERO,
            Money::ZERO,
        )
        .unwrap();
        assert_eq!(summary.projected_balance.cents, -70000); // -400 + 200 - 500 = -700
        assert_eq!(summary.financial_status, FinancialStatus::Danger);
    }

    #[test]
    fn test_aucune_transaction() {
        let summary = BudgetSummary::compute(
            Money::ZERO,
            Money::ZERO,
            Money::ZERO,
            Money::ZERO,
            Money::ZERO,
            Money::ZERO,
            Money::ZERO,
            Money::ZERO,
        )
        .unwrap();
        assert_eq!(summary.projected_balance, Money::ZERO);
        assert_eq!(summary.financial_status, FinancialStatus::Healthy);
    }

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
        .unwrap()
    }

    /// Solde prévisionnel positif : tout le découvert reste disponible.
    #[test]
    fn un_solde_positif_laisse_le_decouvert_intact() {
        let b = budget(100_000, 50_000, 0, 0);
        assert_eq!(b.decouvert_restant(), Money::from_cents(50_000));
        assert_eq!(b.decouvert_utilise(), Money::ZERO);
        assert_eq!(b.depassement_du_decouvert(), Money::ZERO);
    }

    /// Solde prévisionnel négatif : la part consommée est déduite.
    #[test]
    fn un_solde_negatif_entame_le_decouvert() {
        // −200 € prévus sur 500 € autorisés.
        let b = budget(0, 50_000, 0, 20_000);
        assert_eq!(b.decouvert_utilise(), Money::from_cents(20_000));
        assert_eq!(b.decouvert_restant(), Money::from_cents(30_000));
        assert_eq!(b.depassement_du_decouvert(), Money::ZERO);
    }

    /// Dépassement : le reste tombe à zéro, jamais en dessous, et l'ampleur du
    /// dépassement est exposée séparément.
    #[test]
    fn un_depassement_ne_produit_jamais_de_reste_negatif() {
        // −700 € prévus sur 500 € autorisés.
        let b = budget(0, 50_000, 0, 70_000);
        assert_eq!(b.decouvert_restant(), Money::ZERO);
        assert_eq!(b.decouvert_utilise(), Money::from_cents(50_000));
        assert_eq!(b.depassement_du_decouvert(), Money::from_cents(20_000));
        assert_eq!(b.financial_status, FinancialStatus::Danger);
    }

    /// Un compte déjà à découvert au départ doit être calculé correctement :
    /// c'est le cas d'usage qui motivait l'acceptation d'un solde négatif.
    #[test]
    fn un_compte_deja_a_decouvert_est_calcule_correctement() {
        // Solde de départ −360 €, 1 207 € à recevoir, 1 325 € à payer.
        let b = budget(-36_000, 50_000, 120_700, 132_500);
        assert_eq!(b.projected_balance, Money::from_cents(-47_800));
        assert_eq!(b.financial_status, FinancialStatus::Warning);
        assert_eq!(b.decouvert_utilise(), Money::from_cents(47_800));
        assert_eq!(b.decouvert_restant(), Money::from_cents(2_200));
        assert_eq!(b.depassement_du_decouvert(), Money::ZERO);
    }

    /// Sans découvert autorisé, il n'y a jamais de reste à afficher.
    #[test]
    fn sans_decouvert_autorise_le_reste_est_nul() {
        let b = budget(-10_000, 0, 0, 0);
        assert_eq!(b.decouvert_restant(), Money::ZERO);
        assert_eq!(b.decouvert_utilise(), Money::ZERO);
        assert_eq!(b.depassement_du_decouvert(), Money::from_cents(10_000));
    }

    /// Le reste et la part utilisée se complètent toujours pour former la
    /// limite, tant qu'il n'y a pas de dépassement.
    #[test]
    fn le_reste_et_lutilise_reconstituent_la_limite() {
        for sortant in [0, 10_000, 25_000, 50_000] {
            let b = budget(0, 50_000, 0, sortant);
            assert_eq!(
                b.decouvert_restant().cents + b.decouvert_utilise().cents,
                50_000
            );
        }
    }

    #[test]
    fn test_exclusion_transactions_realisees() {
        let summary = BudgetSummary::compute(
            Money::from_cents(10000),
            Money::from_cents(10000),
            Money::ZERO,
            Money::ZERO,
            Money::from_cents(50000),
            Money::from_cents(30000),
            Money::from_cents(50000),
            Money::from_cents(30000),
        )
        .unwrap();
        assert_eq!(summary.projected_balance.cents, 10000);
    }
}
