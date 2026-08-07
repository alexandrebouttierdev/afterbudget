use crate::core::db::pool::DatabasePool;
use crate::domaine::budget::BudgetSummary;
use crate::modules::budget::service as budget_service;

pub fn calculer_budget(
    pool: &DatabasePool,
    year: i32,
    month: u32,
) -> Result<BudgetSummary, String> {
    budget_service::calculer_resume_budget(pool, year, month)
}
