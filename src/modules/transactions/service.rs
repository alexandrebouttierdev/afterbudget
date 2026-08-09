use chrono::NaiveDate;

use crate::core::db::pool::DatabasePool;
use crate::domaine::argent::Money;
use crate::domaine::transaction::{Transaction, TransactionKind, TransactionStatus};
use crate::modules::transactions::repository as repo;

#[allow(clippy::too_many_arguments)]
pub fn creer_transaction(
    pool: &DatabasePool,
    kind: TransactionKind,
    label: &str,
    amount: Money,
    date: NaiveDate,
    category_id: &str,
    status: TransactionStatus,
    note: Option<String>,
) -> Result<Transaction, String> {
    let now = chrono::Utc::now();
    let tx = Transaction {
        id: uuid::Uuid::new_v4().to_string(),
        kind,
        label: label.to_string(),
        amount,
        transaction_date: date,
        status,
        category_id: category_id.to_string(),
        note,
        recurring_rule_id: None,
        created_at: now,
        updated_at: now,
    };
    repo::insert(pool, &tx)?;
    Ok(tx)
}

pub fn modifier_transaction(pool: &DatabasePool, tx: &Transaction) -> Result<(), String> {
    repo::update(pool, tx)
}

pub fn supprimer_transaction(pool: &DatabasePool, id: &str) -> Result<(), String> {
    repo::delete_by_id(pool, id)
}

pub fn changer_statut(pool: &DatabasePool, id: &str) -> Result<Transaction, String> {
    let mut tx =
        repo::find_by_id(pool, id)?.ok_or_else(|| "Transaction introuvable.".to_string())?;
    tx.status = match tx.status {
        TransactionStatus::Pending => TransactionStatus::Completed,
        TransactionStatus::Completed => TransactionStatus::Pending,
    };
    tx.updated_at = chrono::Utc::now();
    repo::update(pool, &tx)?;
    Ok(tx)
}

pub fn lister_transactions(
    pool: &DatabasePool,
    year: i32,
    month: u32,
    kind_filter: Option<&str>,
    status_filter: Option<&str>,
    category_filter: Option<&str>,
    search: Option<&str>,
) -> Result<Vec<Transaction>, String> {
    repo::find_by_month(
        pool,
        year,
        month,
        kind_filter,
        status_filter,
        category_filter,
        search,
    )
}

pub fn transactions_recentes(
    pool: &DatabasePool,
    year: i32,
    month: u32,
    limit: usize,
) -> Result<Vec<Transaction>, String> {
    repo::find_recent_by_month(pool, year, month, limit)
}

/// Solde signé d'un ensemble de transactions : revenus moins dépenses.
///
/// Utilisé pour afficher le total des lignes réellement visibles après
/// filtrage, ce qui donne un sens immédiat au filtre appliqué.
pub fn solde_des(transactions: &[Transaction]) -> Result<Money, String> {
    let mut total = Money::ZERO;
    for transaction in transactions {
        let signe = transaction.signed_amount()?;
        total = total
            .checked_add(signe)
            .ok_or_else(|| "Dépassement de montant dans le total des lignes.".to_string())?;
    }
    Ok(total)
}
