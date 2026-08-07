use crate::domaine::categorie::{DEFAULT_EXPENSE_CATEGORIES, DEFAULT_INCOME_CATEGORIES};
use rusqlite::params;

type MigrationFn = Box<dyn Fn(&rusqlite::Connection) -> Result<(), String>>;

pub fn run_migrations(conn: &rusqlite::Connection) -> Result<(), String> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
            version INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            applied_at TEXT NOT NULL
        );",
    )
    .map_err(|e| format!("Erreur schema_migrations : {}", e))?;

    let current = get_current_version(conn)?;
    let migrations: Vec<MigrationFn> = vec![Box::new(migration_v1), Box::new(migration_v2)];

    for (i, m) in migrations.iter().enumerate() {
        let v = (i + 1) as i64;
        if v > current {
            m(conn)?;
            record_migration(conn, v, &format!("v{}", v))?;
        }
    }
    Ok(())
}

fn get_current_version(conn: &rusqlite::Connection) -> Result<i64, String> {
    conn.query_row(
        "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
        [],
        |r| r.get(0),
    )
    .map_err(|e| format!("Erreur version : {}", e))
}

fn record_migration(conn: &rusqlite::Connection, v: i64, name: &str) -> Result<(), String> {
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO schema_migrations VALUES (?1, ?2, ?3)",
        params![v, name, now],
    )
    .map_err(|e| format!("Erreur enregistrement migration : {}", e))?;
    Ok(())
}

fn migration_v1(conn: &rusqlite::Connection) -> Result<(), String> {
    conn.execute_batch(include_str!("../../../migrations/0001_initial.sql"))
        .map_err(|e| format!("Erreur migration v1 : {}", e))?;
    insert_default_categories(conn)
}

fn migration_v2(conn: &rusqlite::Connection) -> Result<(), String> {
    conn.execute_batch(include_str!("../../../migrations/0002_recurrences.sql"))
        .map_err(|e| format!("Erreur migration v2 : {}", e))
}

fn insert_default_categories(conn: &rusqlite::Connection) -> Result<(), String> {
    let now = chrono::Utc::now().to_rfc3339();
    let all: Vec<(&str, &str, &str, &str, &str)> = DEFAULT_EXPENSE_CATEGORIES
        .iter()
        .map(|c| (c.id, "expense", c.name, c.icon, c.color))
        .chain(
            DEFAULT_INCOME_CATEGORIES
                .iter()
                .map(|c| (c.id, "income", c.name, c.icon, c.color)),
        )
        .collect();

    for (i, (id, kind, name, icon, color)) in all.iter().enumerate() {
        conn.execute(
            "INSERT OR IGNORE INTO categories (id, kind, name, icon, color, sort_order, is_default, is_active, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, 1, 1, ?7, ?7)",
            params![id, kind, name, icon, color, i as i32, &now],
        )
        .map_err(|e| format!("Insertion catégorie {} : {}", id, e))?;
    }
    Ok(())
}
