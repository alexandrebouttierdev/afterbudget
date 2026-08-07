use afterbudget::core::db::migrations;
use afterbudget::core::db::pool::DatabasePool;

pub fn creer_base_test() -> DatabasePool {
    let pool = DatabasePool::open_in_memory().expect("Base mémoire");
    migrations::run_migrations(&pool.conn).expect("Migrations");
    pool
}
