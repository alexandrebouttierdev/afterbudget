use crate::core::db::pool::DatabasePool;
use crate::domaine::categorie::Category;
use crate::modules::categories::service;

pub fn lister(pool: &DatabasePool) -> Result<Vec<Category>, String> {
    service::lister_categories(pool)
}

pub fn lister_toutes(pool: &DatabasePool) -> Result<Vec<Category>, String> {
    service::lister_toutes_categories(pool)
}
