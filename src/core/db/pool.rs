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

        crate::core::config::restreindre_permissions_fichier(path)?;

        tracing::debug!("Base de données ouverte.");

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

    /// Vérifie qu'un fichier est une base SQLite valide : seule l'en-tête de
    /// 16 octets est lue, jamais le fichier entier (AB-003, AB-011).
    pub fn validate_sqlite_file(path: &Path) -> Result<(), String> {
        use std::io::Read;

        if !path.exists() {
            return Err("Fichier introuvable.".into());
        }
        let mut fichier =
            std::fs::File::open(path).map_err(|e| format!("Lecture impossible : {}", e))?;
        let mut en_tete = [0u8; 16];
        let lus = fichier
            .read(&mut en_tete)
            .map_err(|e| format!("Lecture impossible : {}", e))?;
        if lus < 16 || &en_tete != b"SQLite format 3\0" {
            return Err("Le fichier n'est pas une base SQLite.".into());
        }
        Ok(())
    }

    /// Ouvre une base en lecture seule : aucune écriture, aucun WAL/SHM créé.
    /// Sert à valider un fichier importé sans le toucher (AB-003).
    pub fn open_read_only(path: &Path) -> Result<Self, String> {
        let conn = rusqlite::Connection::open_with_flags(
            path,
            OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )
        .map_err(|e| format!("Ouverture en lecture seule impossible : {}", e))?;
        Ok(Self {
            conn,
            path: path.to_path_buf(),
        })
    }

    /// Exécute `PRAGMA integrity_check` sur un fichier, en lecture seule
    /// (AB-003).
    pub fn verifier_integrite(path: &Path) -> Result<(), String> {
        let pool = Self::open_read_only(path)?;
        let resultat: String = pool
            .conn
            .query_row("PRAGMA integrity_check", [], |ligne| ligne.get(0))
            .map_err(|e| format!("Vérification d'intégrité impossible : {}", e))?;
        if resultat != "ok" {
            return Err(format!("Base corrompue : {}", resultat));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// La base créée ne doit pas être lisible par les autres utilisateurs
    /// locaux (AB-010).
    #[test]
    #[cfg(unix)]
    fn la_base_est_privee() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        let chemin = dir.path().join("base.sqlite");
        let pool = DatabasePool::open(&chemin).unwrap();
        drop(pool);
        let mode = std::fs::metadata(&chemin).unwrap().permissions().mode();
        assert_eq!(mode & 0o077, 0, "mode observé : {mode:o}");
    }
}
