use crate::core::db::pool::DatabasePool;
use crate::domaine::argent::Money;
use crate::domaine::budget::BudgetSummary;
use crate::modules::parametres::repository as settings_repo;
use crate::modules::transactions::repository as tx_repo;

pub fn calculer_resume_budget(
    pool: &DatabasePool,
    year: i32,
    month: u32,
) -> Result<BudgetSummary, String> {
    let settings = settings_repo::get_settings(pool)?.unwrap_or_default();

    let pending_income =
        Money::from_cents(tx_repo::sum_pending_by_kind(pool, year, month, "income")?);
    let pending_expenses =
        Money::from_cents(tx_repo::sum_pending_by_kind(pool, year, month, "expense")?);
    let total_income = Money::from_cents(tx_repo::sum_by_kind(pool, year, month, "income")?);
    let total_expenses = Money::from_cents(tx_repo::sum_by_kind(pool, year, month, "expense")?);
    let completed_income = total_income - pending_income;
    let completed_expenses = total_expenses - pending_expenses;

    Ok(BudgetSummary::compute(
        settings.current_balance,
        settings.overdraft_limit,
        pending_income,
        pending_expenses,
        total_income,
        total_expenses,
        completed_income,
        completed_expenses,
    ))
}
