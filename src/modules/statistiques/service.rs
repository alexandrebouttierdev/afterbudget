use crate::core::db::pool::DatabasePool;
use crate::domaine::argent::Money;
use crate::domaine::budget::{CategoryStats, MonthlyStatistics};
use crate::domaine::transaction::TransactionKind;
use crate::modules::categories::repository as categories;
use crate::modules::transactions::repository as transactions;

pub fn calculer_statistiques_mensuelles(
    pool: &DatabasePool,
    year: i32,
    month: u32,
) -> Result<MonthlyStatistics, String> {
    let pending_income_cents = transactions::sum_pending_by_kind(pool, year, month, "income")?;
    let pending_expenses_cents = transactions::sum_pending_by_kind(pool, year, month, "expense")?;
    let total_income_cents = transactions::sum_by_kind(pool, year, month, "income")?;
    let total_expenses_cents = transactions::sum_by_kind(pool, year, month, "expense")?;

    let total_income = Money::from_cents(total_income_cents);
    let total_expenses = Money::from_cents(total_expenses_cents);
    let pending_income = Money::from_cents(pending_income_cents);
    let pending_expenses = Money::from_cents(pending_expenses_cents);
    let completed_income = total_income - pending_income;
    let completed_expenses = total_expenses - pending_expenses;

    let balance = total_income - total_expenses;

    let txs = transactions::find_by_month(pool, year, month, None, None, None, None)?;

    let income_count = txs
        .iter()
        .filter(|t| t.kind == TransactionKind::Income)
        .count();
    let expense_count = txs
        .iter()
        .filter(|t| t.kind == TransactionKind::Expense)
        .count();

    let expenses_by_category =
        statistiques_par_categorie(pool, year, month, TransactionKind::Expense, total_expenses)?;
    let income_by_category =
        statistiques_par_categorie(pool, year, month, TransactionKind::Income, total_income)?;

    Ok(MonthlyStatistics {
        total_income,
        total_expenses,
        balance,
        completed_income,
        pending_income,
        completed_expenses,
        pending_expenses,
        transaction_count: txs.len(),
        income_count,
        expense_count,
        expenses_by_category,
        income_by_category,
    })
}

fn statistiques_par_categorie(
    pool: &DatabasePool,
    year: i32,
    month: u32,
    kind: TransactionKind,
    grand_total: Money,
) -> Result<Vec<CategoryStats>, String> {
    let sums = transactions::sum_by_category(pool, year, month, kind.as_str())?;
    let cats = categories::find_all(pool)?;

    let grand_total_cents = grand_total.cents as f64;

    let mut stats: Vec<CategoryStats> = sums
        .into_iter()
        .filter_map(|(cat_id, total_cents, count)| {
            let cat = cats.iter().find(|c| c.id == cat_id)?;
            let total = Money::from_cents(total_cents);
            let percentage = if grand_total_cents > 0.0 {
                (total_cents as f64 / grand_total_cents) * 100.0
            } else {
                0.0
            };
            Some(CategoryStats {
                category_id: cat_id,
                category_name: cat.name.clone(),
                category_color: cat.color.clone(),
                total,
                percentage,
                count,
            })
        })
        .collect();

    stats.sort_by_key(|b| std::cmp::Reverse(b.total.cents));

    Ok(stats)
}
