use afterbudget::domaine::argent::Money;
use afterbudget::domaine::transaction::{Transaction, TransactionKind, TransactionStatus};
use chrono::NaiveDate;

#[allow(dead_code)]
pub fn transaction_test(
    label: &str,
    amount_cents: i64,
    kind: TransactionKind,
    status: TransactionStatus,
    date: &str,
    category_id: &str,
) -> Transaction {
    let now = chrono::Utc::now();
    Transaction {
        id: uuid::Uuid::new_v4().to_string(),
        kind,
        label: label.to_string(),
        amount: Money::from_cents(amount_cents),
        transaction_date: NaiveDate::parse_from_str(date, "%Y-%m-%d").unwrap(),
        status,
        category_id: category_id.to_string(),
        note: None,
        recurring_rule_id: None,
        created_at: now,
        updated_at: now,
    }
}
