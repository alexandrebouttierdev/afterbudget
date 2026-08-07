use crate::core::db::pool::DatabasePool;
use crate::domaine::budget::MonthlyStatistics;
use crate::modules::statistiques::service as statistiques_service;

pub fn calculer(pool: &DatabasePool, year: i32, month: u32) -> Result<MonthlyStatistics, String> {
    statistiques_service::calculer_statistiques_mensuelles(pool, year, month)
}
