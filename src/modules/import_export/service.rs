use std::path::Path;

use crate::core::db::pool::DatabasePool;

pub fn exporter(pool: &DatabasePool, dest: &Path) -> Result<(), String> {
    pool.backup_to(dest)
}

pub fn valider_import(path: &Path) -> Result<(), String> {
    DatabasePool::validate_sqlite_file(path)?;

    let temp = DatabasePool::open(path)?;

    let tables: Vec<String> = {
        let mut stmt = temp
            .conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table'")
            .map_err(|_| "Base de données illisible.".to_string())?;

        let rows = stmt
            .query_map([], |row| row.get(0))
            .map_err(|_| "Base de données illisible.".to_string())?;

        rows.filter_map(|r| r.ok()).collect()
    };

    let required_tables = [
        "transactions",
        "categories",
        "app_settings",
        "schema_migrations",
    ];
    for table in &required_tables {
        if !tables.iter().any(|t| t == table) {
            return Err(format!(
                "Le fichier n'est pas une base AfterBudget valide (table « {} » manquante).",
                table
            ));
        }
    }

    let version: i64 = temp
        .conn
        .query_row(
            "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    if version > 1 {
        return Err(format!(
            "La base importée utilise une version de schéma ({}) plus récente que celle supportée (1).",
            version
        ));
    }

    Ok(())
}

pub fn importer(pool: &mut DatabasePool, source: &Path) -> Result<(), String> {
    valider_import(source)?;

    let backup_name = format!(
        "afterbudget-pre-import-{}.sqlite",
        chrono::Utc::now().format("%Y%m%d-%H%M%S")
    );
    let backup_path = pool
        .path
        .parent()
        .unwrap_or(Path::new("."))
        .join(&backup_name);
    pool.backup_to(&backup_path)?;

    std::fs::copy(source, &pool.path)
        .map_err(|e| format!("Erreur lors du remplacement de la base : {}", e))?;

    *pool = DatabasePool::open(&pool.path)?;

    Ok(())
}

pub fn nom_fichier_sauvegarde() -> String {
    let now = chrono::Utc::now();
    format!("afterbudget-backup-{}.sqlite", now.format("%Y-%m-%d"))
}
