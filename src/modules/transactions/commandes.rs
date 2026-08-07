use crate::core::db::pool::DatabasePool;
use crate::domaine::transaction::{Transaction, TransactionKind, TransactionStatus};
use crate::modules::commun;
use crate::modules::transactions::dtos::{CreerTransactionDto, ModifierTransactionDto};
use crate::modules::transactions::service;
use crate::modules::transactions::validateurs as valid;

pub fn creer(pool: &DatabasePool, dto: &CreerTransactionDto) -> Result<Transaction, String> {
    valid::valider_creation(dto).map_err(|errs| {
        errs.iter()
            .map(|(f, e)| format!("{}: {}", f, e))
            .collect::<Vec<_>>()
            .join("\n")
    })?;

    let amount = commun::valider_montant(&dto.montant)?;
    let date = commun::valider_date(&dto.date)?;
    let kind =
        TransactionKind::from_str(&dto.type_transaction).ok_or("Type de transaction invalide.")?;
    let status = TransactionStatus::from_str(&dto.statut).ok_or("Statut invalide.")?;

    service::creer_transaction(
        pool,
        kind,
        &dto.libelle,
        amount,
        date,
        &dto.categorie_id,
        status,
        dto.note.clone(),
    )
}

pub fn modifier(pool: &DatabasePool, dto: &ModifierTransactionDto) -> Result<(), String> {
    valid::valider_modification(dto).map_err(|errs| {
        errs.iter()
            .map(|(f, e)| format!("{}: {}", f, e))
            .collect::<Vec<_>>()
            .join("\n")
    })?;

    let amount = commun::valider_montant(&dto.montant)?;
    let date = commun::valider_date(&dto.date)?;
    let kind = TransactionKind::from_str(&dto.type_transaction).ok_or("Type invalide.")?;
    let status = TransactionStatus::from_str(&dto.statut).ok_or("Statut invalide.")?;
    let now = chrono::Utc::now();

    let tx = Transaction {
        id: dto.id.clone(),
        kind,
        label: dto.libelle.clone(),
        amount,
        transaction_date: date,
        status,
        category_id: dto.categorie_id.clone(),
        note: dto.note.clone(),
        recurring_rule_id: None,
        created_at: now,
        updated_at: now,
    };
    service::modifier_transaction(pool, &tx)
}

pub fn supprimer(pool: &DatabasePool, id: &str) -> Result<(), String> {
    service::supprimer_transaction(pool, id)
}

pub fn changer_statut(pool: &DatabasePool, id: &str) -> Result<Transaction, String> {
    service::changer_statut(pool, id)
}

pub fn lister(
    pool: &DatabasePool,
    year: i32,
    month: u32,
    kind: Option<&str>,
    status: Option<&str>,
    category: Option<&str>,
    search: Option<&str>,
) -> Result<Vec<Transaction>, String> {
    service::lister_transactions(pool, year, month, kind, status, category, search)
}

pub fn recentes(
    pool: &DatabasePool,
    year: i32,
    month: u32,
    limit: usize,
) -> Result<Vec<Transaction>, String> {
    service::transactions_recentes(pool, year, month, limit)
}
