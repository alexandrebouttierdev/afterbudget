use std::path::{Path, PathBuf};

pub const APP_NAME: &str = "afterbudget";

/// Restreint l'accès à un répertoire de données : 0700 sur Unix, sans effet
/// sur Windows (pas de mode POSIX) (AB-010).
pub fn restreindre_permissions(chemin: &Path) -> Result<(), String> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(chemin, std::fs::Permissions::from_mode(0o700))
            .map_err(|e| format!("Restriction des permissions impossible : {e}"))?;
    }
    #[cfg(not(unix))]
    let _ = chemin;
    Ok(())
}

/// Restreint l'accès à un fichier de données : 0600 sur Unix (AB-010).
pub fn restreindre_permissions_fichier(chemin: &Path) -> Result<(), String> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(chemin, std::fs::Permissions::from_mode(0o600))
            .map_err(|e| format!("Restriction des permissions impossible : {e}"))?;
    }
    #[cfg(not(unix))]
    let _ = chemin;
    Ok(())
}

pub fn app_data_dir() -> Result<PathBuf, String> {
    let dir = dirs::data_dir().ok_or("Répertoire de données introuvable.")?;
    let app_dir = dir.join(APP_NAME);
    std::fs::create_dir_all(&app_dir)
        .map_err(|e| format!("Création du répertoire impossible : {e}"))?;
    restreindre_permissions(&app_dir)?;
    Ok(app_dir)
}

pub fn database_path() -> Result<PathBuf, String> {
    Ok(app_data_dir()?.join("afterbudget.sqlite"))
}

pub fn backup_dir() -> Result<PathBuf, String> {
    Ok(app_data_dir()?.join("backups"))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Le répertoire de données et les fichiers qu'il contient ne doivent pas
    /// être lisibles par les autres utilisateurs locaux (AB-010).
    #[test]
    #[cfg(unix)]
    fn le_repertoire_est_prive() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        restreindre_permissions(dir.path()).unwrap();
        let mode = std::fs::metadata(dir.path()).unwrap().permissions().mode();
        assert_eq!(mode & 0o077, 0, "mode observé : {mode:o}");
    }
}
