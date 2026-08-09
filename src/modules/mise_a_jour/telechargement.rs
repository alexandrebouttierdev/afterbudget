//! Téléchargement et ouverture de l'installeur.
//!
//! L'asset est téléchargé dans le dossier Téléchargements du système (ou le
//! dossier personnel à défaut), puis ouvert avec le lanceur par défaut.

use std::path::{Path, PathBuf};

/// Dossier où poser l'installeur téléchargé.
fn dossier_telechargements() -> PathBuf {
    dirs::download_dir().unwrap_or_else(|| dirs::home_dir().unwrap_or_else(|| PathBuf::from(".")))
}

/// Nom de fichier sûr, sans les caractères interdits ou douteux.
///
/// L'asset vient de la release du dépôt ; l'assainissement protège quand même
/// d'un nom inattendu dans un chemin du système de fichiers.
pub fn nom_de_fichier_sur(nom: &str) -> String {
    let nettoye: String = nom
        .chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' | '\0' => '_',
            _ => c,
        })
        .collect();
    if nettoye.trim().is_empty() || nettoye == "." || nettoye == ".." {
        "afterbudget-installateur".to_string()
    } else {
        nettoye
    }
}

/// Télécharge l'installeur dans le dossier Téléchargements et retourne son
/// chemin. Le fichier est remplacé s'il existe déjà.
pub async fn telecharger_installeur(url: &str, nom_fichier: &str) -> Result<PathBuf, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(600))
        .user_agent(concat!("afterbudget/", env!("CARGO_PKG_VERSION")))
        .build()
        .map_err(|e| format!("Client HTTP impossible : {e}"))?;

    let reponse = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("Téléchargement impossible : {e}"))?;
    let statut = reponse.status();
    if !statut.is_success() {
        return Err(format!("Le téléchargement a échoué ({statut})."));
    }
    let contenu = reponse
        .bytes()
        .await
        .map_err(|e| format!("Téléchargement interrompu : {e}"))?;

    let chemin = dossier_telechargements().join(nom_de_fichier_sur(nom_fichier));
    std::fs::write(&chemin, &contenu)
        .map_err(|e| format!("Écriture impossible de {} : {e}", chemin.display()))?;
    Ok(chemin)
}

/// Ouvre un fichier avec le lanceur par défaut du système.
///
/// Chaque plateforme a son lanceur ; aucune dépendance supplémentaire n'est
/// nécessaire pour les trois cibles visées.
pub fn ouvrir_fichier(chemin: &Path) -> Result<(), String> {
    let chemin = chemin.to_string_lossy();
    #[cfg(target_os = "linux")]
    let (programme, arguments): (&str, Vec<&str>) = ("xdg-open", vec![&chemin]);
    #[cfg(target_os = "macos")]
    let (programme, arguments): (&str, Vec<&str>) = ("open", vec![&chemin]);
    #[cfg(target_os = "windows")]
    let (programme, arguments): (&str, Vec<&str>) = ("cmd", vec!["/C", "start", "", &chemin]);

    std::process::Command::new(programme)
        .args(arguments)
        .spawn()
        .map(|_| ())
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn un_nom_normal_est_conserve() {
        assert_eq!(
            nom_de_fichier_sur("afterbudget-0.2.3.deb"),
            "afterbudget-0.2.3.deb"
        );
    }

    #[test]
    fn les_caracteres_dangereux_sont_remplaces() {
        assert_eq!(
            nom_de_fichier_sur("a/b\\c:d*e?f\"g<h>i|j"),
            "a_b_c_d_e_f_g_h_i_j"
        );
    }

    /// Un nom vide ou réduit à un point ne doit jamais désigner le dossier
    /// courant ou parent : on retombe sur un nom sûr.
    #[test]
    fn un_nom_dangereux_ou_vide_est_remplace() {
        assert_eq!(nom_de_fichier_sur(""), "afterbudget-installateur");
        assert_eq!(nom_de_fichier_sur("."), "afterbudget-installateur");
        assert_eq!(nom_de_fichier_sur(".."), "afterbudget-installateur");
        assert_eq!(nom_de_fichier_sur("  "), "afterbudget-installateur");
    }
}
