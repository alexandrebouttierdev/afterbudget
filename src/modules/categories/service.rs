use crate::core::db::pool::DatabasePool;
use crate::domaine::categorie::Category;
use crate::domaine::transaction::TransactionKind;
use crate::modules::categories::repository as repo;

pub fn lister_categories(pool: &DatabasePool) -> Result<Vec<Category>, String> {
    repo::find_all_active(pool)
}

pub fn lister_toutes_categories(pool: &DatabasePool) -> Result<Vec<Category>, String> {
    repo::find_all(pool)
}

pub fn categories_par_type(pool: &DatabasePool, kind: &str) -> Result<Vec<Category>, String> {
    repo::find_by_kind(pool, kind)
}

pub fn trouver_categorie(pool: &DatabasePool, id: &str) -> Result<Option<Category>, String> {
    repo::find_by_id(pool, id)
}

/// Vérifie qu'une catégorie peut porter une transaction ou une règle du sens
/// donné : existante, active, et du même type (AB-009).
pub fn verifier_categorie(
    pool: &DatabasePool,
    kind: TransactionKind,
    category_id: &str,
) -> Result<(), String> {
    let cat = crate::modules::categories::repository::find_by_id(pool, category_id)?
        .ok_or_else(|| format!("Catégorie inconnue : « {} ».", category_id))?;
    if !cat.is_active {
        return Err(format!("La catégorie « {} » est désactivée.", cat.name));
    }
    if cat.kind != kind {
        return Err(format!(
            "La catégorie « {} » est une catégorie de {}, pas de {}.",
            cat.name,
            cat.kind.display_name(),
            kind.display_name()
        ));
    }
    Ok(())
}
