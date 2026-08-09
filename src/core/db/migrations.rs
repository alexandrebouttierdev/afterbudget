use crate::domaine::categorie::{DEFAULT_EXPENSE_CATEGORIES, DEFAULT_INCOME_CATEGORIES};
use rusqlite::params;

/// Dernière version de schéma produite par cette application. Le validateur
/// d'import s'en sert pour accepter ou refuser un fichier (AB-002).
pub const VERSION_COURANTE: i64 = 3;

type MigrationFn = Box<dyn Fn(&rusqlite::Transaction) -> Result<(), String>>;

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
    let migrations: Vec<(i64, &str, MigrationFn)> = vec![
        (1, "v1", Box::new(migration_v1)),
        (2, "v2", Box::new(migration_v2)),
        (3, "v3", Box::new(migration_v3)),
    ];

    for (v, nom, m) in migrations {
        if v > current {
            appliquer_migration(conn, v, nom, &m)?;
        }
    }
    Ok(())
}

/// Applique une migration dans une transaction unique : le SQL et
/// l'enregistrement de version sont engagés ensemble, ou annulés ensemble
/// (AB-007).
fn appliquer_migration(
    conn: &rusqlite::Connection,
    v: i64,
    nom: &str,
    m: &MigrationFn,
) -> Result<(), String> {
    // Aucun appelant n'ouvre de transaction autour de run_migrations : une
    // transaction imbriquée est donc impossible, `unchecked` est sûr.
    let tx = conn
        .unchecked_transaction()
        .map_err(|e| format!("Démarrage de la transaction v{v} : {e}"))?;
    m(&tx).map_err(|e| format!("Erreur migration {nom} (v{v}) : {e}"))?;

    let now = chrono::Utc::now().to_rfc3339();
    tx.execute(
        "INSERT INTO schema_migrations VALUES (?1, ?2, ?3)",
        params![v, nom, now],
    )
    .map_err(|e| format!("Erreur enregistrement migration v{v} : {e}"))?;

    tx.commit()
        .map_err(|e| format!("Erreur commit migration v{v} : {e}"))?;
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

fn migration_v1(tx: &rusqlite::Transaction) -> Result<(), String> {
    tx.execute_batch(include_str!("../../../migrations/0001_initial.sql"))
        .map_err(|e| format!("Erreur migration v1 : {}", e))?;
    insert_default_categories(tx)
}

fn migration_v2(tx: &rusqlite::Transaction) -> Result<(), String> {
    // Garde d'idempotence : une base interrompue par l'ancien bug AB-007 a
    // déjà la colonne sans la version enregistrée. Ne jamais rejouer l'ALTER.
    if !colonne_existe(tx, "transactions", "recurring_rule_id")? {
        tx.execute_batch(
            "ALTER TABLE transactions ADD COLUMN recurring_rule_id TEXT REFERENCES recurring_rules(id);",
        )
        .map_err(|e| format!("Erreur migration v2 (colonne) : {}", e))?;
    }
    tx.execute_batch(include_str!("../../../migrations/0002_recurrences.sql"))
        .map_err(|e| format!("Erreur migration v2 : {}", e))?;
    Ok(())
}

fn migration_v3(tx: &rusqlite::Transaction) -> Result<(), String> {
    if !colonne_existe(tx, "app_settings", "last_export_date")? {
        tx.execute_batch(include_str!(
            "../../../migrations/0003_last_export_date.sql"
        ))
        .map_err(|e| format!("Erreur migration v3 : {}", e))?;
    }
    Ok(())
}

/// Vrai si une colonne existe déjà dans une table : sert à rendre les ALTER
/// idempotents (AB-007).
fn colonne_existe(tx: &rusqlite::Transaction, table: &str, colonne: &str) -> Result<bool, String> {
    // `table` est toujours un nom de table interne (transactions,
    // app_settings) : jamais une entrée utilisateur, l'interpolation est sans
    // risque d'injection SQL.
    let mut stmt = tx
        .prepare(&format!("PRAGMA table_info({table})"))
        .map_err(|e| format!("Lecture du schéma de {table} : {e}"))?;
    let lignes = stmt
        .query_map([], |ligne| ligne.get::<_, String>(1))
        .map_err(|e| format!("Lecture du schéma de {table} : {e}"))?;
    for nom in lignes {
        let nom = nom.map_err(|e| format!("Lecture du schéma de {table} : {e}"))?;
        if nom == colonne {
            return Ok(true);
        }
    }
    Ok(false)
}

fn insert_default_categories(tx: &rusqlite::Transaction) -> Result<(), String> {
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
        tx.execute(
            "INSERT OR IGNORE INTO categories (id, kind, name, icon, color, sort_order, is_default, is_active, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, 1, 1, ?7, ?7)",
            params![id, kind, name, icon, color, i as i32, &now],
        )
        .map_err(|e| format!("Insertion catégorie {} : {}", id, e))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn connexion() -> rusqlite::Connection {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS schema_migrations (
                version INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                applied_at TEXT NOT NULL
            );",
        )
        .unwrap();
        conn
    }

    /// Une migration dont le SQL échoue doit laisser la base intacte : ni
    /// table partielle, ni version enregistrée (AB-007).
    #[test]
    fn une_migration_qui_echoue_est_annulee() {
        let conn = connexion();
        let echouante: MigrationFn = Box::new(|tx| {
            tx.execute_batch("CREATE TABLE partielle_test (x INTEGER);")
                .map_err(|e| e.to_string())?;
            Err("échec injecté".into())
        });
        assert!(appliquer_migration(&conn, 1, "v1", &echouante).is_err());

        let tables: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='partielle_test'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(tables, 0, "la table partielle doit être annulée");

        let version: i64 = conn
            .query_row(
                "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(version, 0, "aucune version ne doit être enregistrée");
    }

    /// Une migration réussie enregistre SQL et version ensemble.
    #[test]
    fn une_migration_reussie_enregistre_sql_et_version() {
        let conn = connexion();
        let reussie: MigrationFn = Box::new(|tx| {
            tx.execute_batch("CREATE TABLE complete_test (x INTEGER);")
                .map_err(|e| e.to_string())?;
            Ok(())
        });
        appliquer_migration(&conn, 1, "v1", &reussie).unwrap();

        let tables: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='complete_test'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(tables, 1);
        let version: i64 = conn
            .query_row("SELECT MAX(version) FROM schema_migrations", [], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(version, 1);
    }
}
