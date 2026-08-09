use std::path::{Path, PathBuf};

use crate::core::db::migrations;
use crate::core::db::pool::DatabasePool;

pub fn exporter(pool: &DatabasePool, dest: &Path) -> Result<(), String> {
    pool.backup_to(dest)
}

pub fn valider_import(path: &Path) -> Result<(), String> {
    DatabasePool::validate_sqlite_file(path)?;
    DatabasePool::verifier_integrite(path)?;

    let temp = DatabasePool::open_read_only(path)?;

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

    let version: i64 = temp
        .conn
        .query_row(
            "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let tables_requises: &[&str] = if version >= 2 {
        &[
            "transactions",
            "categories",
            "app_settings",
            "schema_migrations",
            "recurring_rules",
        ]
    } else {
        &["transactions", "categories", "app_settings", "schema_migrations"]
    };
    for table in tables_requises {
        if !tables.iter().any(|t| t == table) {
            return Err(format!(
                "Le fichier n'est pas une base AfterBudget valide (table « {} » manquante).",
                table
            ));
        }
    }

    if version < 1 {
        return Err("La base importée ne contient aucune migration enregistrée.".into());
    }
    if version > migrations::VERSION_COURANTE {
        return Err(format!(
            "La base importée utilise une version de schéma ({}) plus récente que celle supportée ({}).",
            version,
            migrations::VERSION_COURANTE
        ));
    }

    Ok(())
}

/// Importe un fichier validé dans la base courante, **hors du thread UI**
/// (AB-003, AB-011).
///
/// La séquence est : validation en lecture seule → backup → copie vers un
/// fichier temporaire du même répertoire → fsync → rename atomique → migration
/// du fichier importé. Tout échec après le backup restaure le backup ; si la
/// restauration échoue elle-même, le chemin de la sauvegarde est indiqué.
pub fn importer_en_arriere_plan(chemin_actuel: PathBuf, source: PathBuf) -> Result<(), String> {
    valider_import(&source)?;

    let backup_path = chemin_actuel
        .parent()
        .ok_or("Chemin de base invalide.")?
        .join(format!(
            "afterbudget-pre-import-{}.sqlite",
            chrono::Utc::now().format("%Y%m%d-%H%M%S")
        ));

    let pool = DatabasePool::open(&chemin_actuel)?;
    pool.backup_to(&backup_path)?;
    drop(pool);

    if let Err(e) = remplacer_fichier(&chemin_actuel, &source) {
        let restauration = restaurer_fichier(&chemin_actuel, &backup_path);
        return Err(match restauration {
            Ok(()) => format!("{e} ; tes données ont été restaurées."),
            Err(erreur) => format!(
                "{e} ; la restauration a échoué ({erreur}). Sauvegarde disponible : {}",
                backup_path.display()
            ),
        });
    }

    // Les anciens exports (v1/v2) doivent être migrés avant réouverture.
    let migre = DatabasePool::open(&chemin_actuel).and_then(|p| {
        migrations::run_migrations(&p.conn).map_err(|e| format!("Migration de l'import impossible : {e}"))
    });
    if let Err(e) = migre {
        let restauration = restaurer_fichier(&chemin_actuel, &backup_path);
        return Err(match restauration {
            Ok(()) => format!("{e} ; tes données ont été restaurées."),
            Err(erreur) => format!(
                "{e} ; la restauration a échoué ({erreur}). Sauvegarde disponible : {}",
                backup_path.display()
            ),
        });
    }

    Ok(())
}

/// Remplace `destination` par `source` de façon atomique : copie vers un
/// fichier temporaire du même répertoire, synchronisation, puis rename.
fn remplacer_fichier(destination: &Path, source: &Path) -> Result<(), String> {
    let repertoire = destination.parent().ok_or("Chemin de base invalide.")?;
    let temporaire = repertoire.join(format!(
        ".afterbudget-import-{}.sqlite",
        uuid::Uuid::new_v4()
    ));

    std::fs::copy(source, &temporaire)
        .map_err(|e| format!("Copie du fichier impossible : {e}"))?;

    let fichier = std::fs::File::open(&temporaire)
        .map_err(|e| format!("Ouverture du fichier temporaire : {e}"))?;
    fichier.sync_all().map_err(|e| format!("Synchronisation impossible : {e}"))?;
    drop(fichier);

    std::fs::rename(&temporaire, destination)
        .map_err(|e| format!("Remplacement du fichier impossible : {e}"))?;
    Ok(())
}

fn restaurer_fichier(destination: &Path, backup: &Path) -> Result<(), String> {
    remplacer_fichier(destination, backup)
}

pub fn nom_fichier_sauvegarde() -> String {
    let now = chrono::Utc::now();
    format!("afterbudget-backup-{}.sqlite", now.format("%Y-%m-%d"))
}
