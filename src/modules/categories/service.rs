use crate::core::db::pool::DatabasePool;
use crate::domaine::categorie::Category;
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
