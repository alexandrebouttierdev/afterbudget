use rusqlite::OpenFlags;
use std::path::Path;
use tracing;

pub struct DatabasePool {
    pub conn: rusqlite::Connection,
    pub path: std::path::PathBuf,
}

impl DatabasePool {
    pub fn open(path: &Path) -> Result<Self, String> {
        let conn = rusqlite::Connection::open_with_flags(
            path,
            OpenFlags::SQLITE_OPEN_READ_WRITE
                | OpenFlags::SQLITE_OPEN_CREATE
                | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )
        .map_err(|e| format!("Impossible d'ouvrir la base : {}", e))?;

        conn.execute_batch("PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL;")
            .map_err(|e| format!("Erreur configuration SQLite : {}", e))?;

        tracing::info!("Base de données ouverte : {}", path.display());

        Ok(Self {
            conn,
            path: path.to_path_buf(),
        })
    }

    /// Ouvre une base temporaire en mémoire pour les tests
    pub fn open_in_memory() -> Result<Self, String> {
        let conn = rusqlite::Connection::open_in_memory()
            .map_err(|e| format!("Erreur base mémoire : {}", e))?;

        conn.execute_batch("PRAGMA foreign_keys = ON;")
            .map_err(|e| format!("Erreur pragma : {}", e))?;

        Ok(Self {
            conn,
            path: std::path::PathBuf::from(":memory:"),
        })
    }

    /// Sauvegarde cohérente via l'API backup SQLite
    pub fn backup_to(&self, dest_path: &Path) -> Result<(), String> {
        let mut dest = rusqlite::Connection::open(dest_path)
            .map_err(|e| format!("Création sauvegarde : {}", e))?;

        let backup = rusqlite::backup::Backup::new(&self.conn, &mut dest)
            .map_err(|e| format!("Initialisation backup : {}", e))?;

        backup
            .run_to_completion(5, std::time::Duration::from_millis(250), None)
            .map_err(|e| format!("Échec backup : {}", e))?;

        Ok(())
    }

    /// Vérifie qu'un fichier est une base SQLite valide
    pub fn validate_sqlite_file(path: &Path) -> Result<(), String> {
        if !path.exists() {
            return Err("Fichier introuvable.".into());
        }
        let header = std::fs::read(path).map_err(|e| format!("Lecture impossible : {}", e))?;
        if header.len() < 16 || &header[0..16] != b"SQLite format 3\0" {
            return Err("Le fichier n'est pas une base SQLite.".into());
        }
        Ok(())
    }
}
