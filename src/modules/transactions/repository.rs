use rusqlite::params;

use crate::core::db::modeles::TransactionRow;
use crate::core::db::pool::DatabasePool;
use crate::core::utils::last_day_of_month;
use crate::domaine::transaction::Transaction;

fn row_to_transaction(row: &TransactionRow) -> Result<Transaction, String> {
    use crate::domaine::argent::Money;
    use crate::domaine::transaction::{TransactionKind, TransactionStatus};

    let kind = TransactionKind::from_str(&row.kind)
        .ok_or_else(|| format!("Transaction {} : type inconnu « {} ».", row.id, row.kind))?;
    let transaction_date = chrono::NaiveDate::parse_from_str(&row.transaction_date, "%Y-%m-%d")
        .map_err(|_| {
            format!(
                "Transaction {} : date invalide « {} ».",
                row.id, row.transaction_date
            )
        })?;
    let status = TransactionStatus::from_str(&row.status).ok_or_else(|| {
        format!(
            "Transaction {} : statut inconnu « {} ».",
            row.id, row.status
        )
    })?;
    let parse_horodatage = |brut: &str| {
        chrono::DateTime::parse_from_rfc3339(brut)
            .map(|d| d.with_timezone(&chrono::Utc))
            .map_err(|_| format!("Transaction {} : horodatage invalide « {} ».", row.id, brut))
    };

    Ok(Transaction {
        id: row.id.clone(),
        kind,
        label: row.label.clone(),
        amount: Money::from_cents(row.amount_cents),
        transaction_date,
        status,
        category_id: row.category_id.clone(),
        note: row.note.clone(),
        recurring_rule_id: row.recurring_rule_id.clone(),
        created_at: parse_horodatage(&row.created_at)?,
        updated_at: parse_horodatage(&row.updated_at)?,
    })
}

pub fn find_by_month(
    pool: &DatabasePool,
    year: i32,
    month: u32,
    kind_filter: Option<&str>,
    status_filter: Option<&str>,
    category_filter: Option<&str>,
    search: Option<&str>,
) -> Result<Vec<Transaction>, String> {
    let start_date = format!("{:04}-{:02}-01", year, month);
    let end_day = last_day_of_month(year, month);
    let end_date = format!("{:04}-{:02}-{:02}", year, month, end_day);

    let mut sql = String::from(
        "SELECT id, kind, label, amount_cents, transaction_date, status, category_id, note, recurring_rule_id, created_at, updated_at
         FROM transactions
         WHERE transaction_date >= ?1 AND transaction_date <= ?2",
    );

    let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> =
        vec![Box::new(start_date), Box::new(end_date)];

    if let Some(kind) = kind_filter {
        sql.push_str(" AND kind = ?");
        sql.push_str(&(param_values.len() + 1).to_string());
        param_values.push(Box::new(kind.to_string()));
    }

    if let Some(status) = status_filter {
        sql.push_str(" AND status = ?");
        sql.push_str(&(param_values.len() + 1).to_string());
        param_values.push(Box::new(status.to_string()));
    }

    if let Some(cat) = category_filter {
        sql.push_str(" AND category_id = ?");
        sql.push_str(&(param_values.len() + 1).to_string());
        param_values.push(Box::new(cat.to_string()));
    }

    if let Some(q) = search {
        let like = format!("%{}%", q);
        let idx1 = param_values.len() + 1;
        let idx2 = param_values.len() + 2;
        sql.push_str(&format!(
            " AND (label LIKE ?{} OR note LIKE ?{})",
            idx1, idx2
        ));
        param_values.push(Box::new(like.clone()));
        param_values.push(Box::new(like));
    }

    sql.push_str(" ORDER BY transaction_date DESC, created_at DESC");

    let param_refs: Vec<&dyn rusqlite::types::ToSql> =
        param_values.iter().map(|p| p.as_ref()).collect();

    let mut stmt = pool
        .conn
        .prepare(&sql)
        .map_err(|e| format!("Erreur de préparation : {}", e))?;

    let rows = stmt
        .query_map(param_refs.as_slice(), |row| {
            Ok(TransactionRow {
                id: row.get(0)?,
                kind: row.get(1)?,
                label: row.get(2)?,
                amount_cents: row.get(3)?,
                transaction_date: row.get(4)?,
                status: row.get(5)?,
                category_id: row.get(6)?,
                note: row.get(7)?,
                recurring_rule_id: row.get(8)?,
                created_at: row.get(9)?,
                updated_at: row.get(10)?,
            })
        })
        .map_err(|e| format!("Erreur de requête : {}", e))?;

    let mut transactions = Vec::new();
    for row in rows {
        let row = row.map_err(|e| format!("Erreur de lecture : {}", e))?;
        transactions.push(row_to_transaction(&row)?);
    }

    Ok(transactions)
}

pub fn find_all_by_month(
    pool: &DatabasePool,
    year: i32,
    month: u32,
) -> Result<Vec<Transaction>, String> {
    find_by_month(pool, year, month, None, None, None, None)
}

pub fn find_recent_by_month(
    pool: &DatabasePool,
    year: i32,
    month: u32,
    limit: usize,
) -> Result<Vec<Transaction>, String> {
    let start_date = format!("{:04}-{:02}-01", year, month);
    let end_day = last_day_of_month(year, month);
    let end_date = format!("{:04}-{:02}-{:02}", year, month, end_day);

    let mut stmt = pool
        .conn
        .prepare(
            "SELECT id, kind, label, amount_cents, transaction_date, status, category_id, note, recurring_rule_id, created_at, updated_at
             FROM transactions
             WHERE transaction_date >= ?1 AND transaction_date <= ?2
             ORDER BY transaction_date DESC, created_at DESC
             LIMIT ?3",
        )
        .map_err(|e| format!("Erreur de préparation : {}", e))?;

    let rows = stmt
        .query_map(params![start_date, end_date, limit as i64], |row| {
            Ok(TransactionRow {
                id: row.get(0)?,
                kind: row.get(1)?,
                label: row.get(2)?,
                amount_cents: row.get(3)?,
                transaction_date: row.get(4)?,
                status: row.get(5)?,
                category_id: row.get(6)?,
                note: row.get(7)?,
                recurring_rule_id: row.get(8)?,
                created_at: row.get(9)?,
                updated_at: row.get(10)?,
            })
        })
        .map_err(|e| format!("Erreur de requête : {}", e))?;

    let mut transactions = Vec::new();
    for row in rows {
        let row = row.map_err(|e| format!("Erreur de lecture : {}", e))?;
        transactions.push(row_to_transaction(&row)?);
    }

    Ok(transactions)
}

pub fn find_by_id(pool: &DatabasePool, id: &str) -> Result<Option<Transaction>, String> {
    let mut stmt = pool
        .conn
        .prepare(
            "SELECT id, kind, label, amount_cents, transaction_date, status, category_id, note, recurring_rule_id, created_at, updated_at
             FROM transactions WHERE id = ?1",
        )
        .map_err(|e| format!("Erreur de préparation : {}", e))?;

    let mut rows = stmt
        .query_map(params![id], |row| {
            Ok(TransactionRow {
                id: row.get(0)?,
                kind: row.get(1)?,
                label: row.get(2)?,
                amount_cents: row.get(3)?,
                transaction_date: row.get(4)?,
                status: row.get(5)?,
                category_id: row.get(6)?,
                note: row.get(7)?,
                recurring_rule_id: row.get(8)?,
                created_at: row.get(9)?,
                updated_at: row.get(10)?,
            })
        })
        .map_err(|e| format!("Erreur de requête : {}", e))?;

    match rows.next() {
        Some(Ok(row)) => Ok(Some(row_to_transaction(&row)?)),
        Some(Err(e)) => Err(format!("Erreur de lecture : {}", e)),
        None => Ok(None),
    }
}

/// Insère une occurrence produite par une règle récurrente.
///
/// Le rattachement à la règle est ce qui permet, au rechargement du mois, de
/// savoir que l'occurrence existe déjà et de ne pas la recréer.
#[allow(clippy::too_many_arguments)]
pub fn insert_from_rule(
    pool: &DatabasePool,
    identifiant_regle: &str,
    kind: crate::domaine::transaction::TransactionKind,
    label: &str,
    amount: crate::domaine::argent::Money,
    date: chrono::NaiveDate,
    category_id: &str,
    status: crate::domaine::transaction::TransactionStatus,
    note: Option<String>,
) -> Result<(), String> {
    let maintenant = chrono::Utc::now().to_rfc3339();
    pool.conn
        .execute(
            "INSERT INTO transactions (id, kind, label, amount_cents, transaction_date, status,
                category_id, note, recurring_rule_id, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?10)",
            params![
                uuid::Uuid::new_v4().to_string(),
                kind.as_str(),
                label,
                amount.cents,
                date.format("%Y-%m-%d").to_string(),
                status.as_str(),
                category_id,
                note,
                identifiant_regle,
                &maintenant,
            ],
        )
        .map_err(|e| format!("Insertion d'une occurrence récurrente : {e}"))?;
    Ok(())
}

pub fn insert(pool: &DatabasePool, t: &Transaction) -> Result<(), String> {
    let now = chrono::Utc::now().to_rfc3339();
    pool.conn
        .execute(
            "INSERT INTO transactions (id, kind, label, amount_cents, transaction_date, status, category_id, note, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                t.id,
                t.kind.as_str(),
                t.label,
                t.amount.cents,
                t.transaction_date.format("%Y-%m-%d").to_string(),
                t.status.as_str(),
                t.category_id,
                t.note,
                &now,
                &now,
            ],
        )
        .map_err(|e| format!("Erreur d'insertion : {}", e))?;
    Ok(())
}

pub fn update(pool: &DatabasePool, t: &Transaction) -> Result<(), String> {
    let now = chrono::Utc::now().to_rfc3339();
    let maj = pool
        .conn
        .execute(
            "UPDATE transactions SET kind = ?1, label = ?2, amount_cents = ?3, transaction_date = ?4,
             status = ?5, category_id = ?6, note = ?7, updated_at = ?8
             WHERE id = ?9",
            params![
                t.kind.as_str(),
                t.label,
                t.amount.cents,
                t.transaction_date.format("%Y-%m-%d").to_string(),
                t.status.as_str(),
                t.category_id,
                t.note,
                &now,
                t.id,
            ],
        )
        .map_err(|e| format!("Erreur de mise à jour : {}", e))?;

    if maj == 0 {
        return Err(format!("Transaction introuvable : {}", t.id));
    }
    Ok(())
}

pub fn delete_by_id(pool: &DatabasePool, id: &str) -> Result<(), String> {
    let supprimes = pool
        .conn
        .execute("DELETE FROM transactions WHERE id = ?1", params![id])
        .map_err(|e| format!("Erreur de suppression : {}", e))?;

    if supprimes == 0 {
        return Err(format!("Transaction introuvable : {}", id));
    }
    Ok(())
}

pub fn sum_pending_by_kind(
    pool: &DatabasePool,
    year: i32,
    month: u32,
    kind: &str,
) -> Result<i64, String> {
    let start_date = format!("{:04}-{:02}-01", year, month);
    let end_day = last_day_of_month(year, month);
    let end_date = format!("{:04}-{:02}-{:02}", year, month, end_day);

    let result: Result<i64, _> = pool.conn.query_row(
        "SELECT COALESCE(SUM(amount_cents), 0) FROM transactions
         WHERE transaction_date >= ?1 AND transaction_date <= ?2
         AND kind = ?3 AND status = 'pending'",
        params![start_date, end_date, kind],
        |row| row.get(0),
    );

    result.map_err(|e| format!("Erreur de calcul : {}", e))
}

pub fn sum_by_kind(pool: &DatabasePool, year: i32, month: u32, kind: &str) -> Result<i64, String> {
    let start_date = format!("{:04}-{:02}-01", year, month);
    let end_day = last_day_of_month(year, month);
    let end_date = format!("{:04}-{:02}-{:02}", year, month, end_day);

    let result: Result<i64, _> = pool.conn.query_row(
        "SELECT COALESCE(SUM(amount_cents), 0) FROM transactions
         WHERE transaction_date >= ?1 AND transaction_date <= ?2
         AND kind = ?3",
        params![start_date, end_date, kind],
        |row| row.get(0),
    );

    result.map_err(|e| format!("Erreur de calcul : {}", e))
}

pub fn count_by_kind(
    pool: &DatabasePool,
    year: i32,
    month: u32,
    kind: &str,
) -> Result<i64, String> {
    let start_date = format!("{:04}-{:02}-01", year, month);
    let end_day = last_day_of_month(year, month);
    let end_date = format!("{:04}-{:02}-{:02}", year, month, end_day);

    let result: Result<i64, _> = pool.conn.query_row(
        "SELECT COUNT(*) FROM transactions
         WHERE transaction_date >= ?1 AND transaction_date <= ?2
         AND kind = ?3",
        params![start_date, end_date, kind],
        |row| row.get(0),
    );

    result.map_err(|e| format!("Erreur de comptage : {}", e))
}

pub fn sum_by_category(
    pool: &DatabasePool,
    year: i32,
    month: u32,
    kind: &str,
) -> Result<Vec<(String, i64, usize)>, String> {
    let start_date = format!("{:04}-{:02}-01", year, month);
    let end_day = last_day_of_month(year, month);
    let end_date = format!("{:04}-{:02}-{:02}", year, month, end_day);

    let mut stmt = pool
        .conn
        .prepare(
            "SELECT category_id, SUM(amount_cents), COUNT(*)
             FROM transactions
             WHERE transaction_date >= ?1 AND transaction_date <= ?2
             AND kind = ?3
             GROUP BY category_id
             ORDER BY SUM(amount_cents) DESC",
        )
        .map_err(|e| format!("Erreur de préparation : {}", e))?;

    let rows = stmt
        .query_map(params![start_date, end_date, kind], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        })
        .map_err(|e| format!("Erreur de requête : {}", e))?;

    let mut results = Vec::new();
    for row in rows {
        results.push(row.map_err(|e| format!("Erreur de lecture : {}", e))?);
    }

    Ok(results)
}
