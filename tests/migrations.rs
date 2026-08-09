use afterbudget::core::db::migrations;
use afterbudget::core::db::pool::DatabasePool;
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

#[test]
fn une_base_interrompue_entre_alter_et_version_est_reparable() {
    let pool = DatabasePool::open_in_memory().unwrap();
    // v1 appliquée, colonne recurring_rule_id ajoutée par un v2 « à moitié
    // appliqué » (ALTER réussi, version jamais enregistrée) : le scénario
    // exact du bug AB-007.
    pool.conn
        .execute_batch(include_str!("../migrations/0001_initial.sql"))
        .unwrap();
    pool.conn
        .execute_batch(
            "CREATE TABLE IF NOT EXISTS schema_migrations (
                version INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                applied_at TEXT NOT NULL
            );",
        )
        .unwrap();
    pool.conn
        .execute(
            "INSERT INTO schema_migrations VALUES (1, 'v1', ?1)",
            rusqlite::params![chrono::Utc::now().to_rfc3339()],
        )
        .unwrap();
    pool.conn
        .execute_batch(
            "ALTER TABLE transactions ADD COLUMN recurring_rule_id TEXT REFERENCES recurring_rules(id);",
        )
        .unwrap();

    assert!(migrations::run_migrations(&pool.conn).is_ok());

    let version: i64 = pool
        .conn
        .query_row("SELECT MAX(version) FROM schema_migrations", [], |r| r.get(0))
        .unwrap();
    assert_eq!(version, migrations::VERSION_COURANTE);

    let colonnes: i64 = pool
        .conn
        .query_row(
            "SELECT COUNT(*) FROM pragma_table_info('transactions') WHERE name = 'recurring_rule_id'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(colonnes, 1);
}

#[test]
fn une_migration_qui_echoue_ne_laisse_rien_derriere() {
    // Test unitaire du mécanisme transactionnel, dans le module migrations.
}
