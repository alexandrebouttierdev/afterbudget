use afterbudget::core::db::migrations;
use afterbudget::modules::categories::repository as cat_repo;

mod commun;
use commun::base_temporaire;

#[test]
fn test_migrations_cree_tables() {
    let pool = base_temporaire::creer_base_test();
    let count: i64 = pool
        .conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='transactions'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert!(count > 0);
}

#[test]
fn test_migrations_idempotentes() {
    let pool = base_temporaire::creer_base_test();
    assert!(migrations::run_migrations(&pool.conn).is_ok());
}

#[test]
fn test_categories_initiales() {
    let pool = base_temporaire::creer_base_test();
    let cats = cat_repo::find_all_active(&pool).unwrap();
    assert!(!cats.is_empty());
    assert!(cats.len() >= 35);
}

#[test]
fn test_categories_pas_de_doublons() {
    let pool = base_temporaire::creer_base_test();
    migrations::run_migrations(&pool.conn).unwrap();
    let cats = cat_repo::find_all_active(&pool).unwrap();
    let count = cats.len();
    migrations::run_migrations(&pool.conn).unwrap();
    let cats2 = cat_repo::find_all_active(&pool).unwrap();
    assert_eq!(cats2.len(), count);
}
