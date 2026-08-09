//! Choix de l'installeur adapté au système.
//!
//! Chaque plateforme cible a son extension d'asset : `.exe` (Windows), `.dmg`
//! (macOS), `.deb` (Debian/Ubuntu) ou `.rpm` (Fedora/RHEL). Sur Linux, la
//! famille est déduite de `/etc/os-release`.

/// Extension d'asset à télécharger pour ce système, sans le point.
///
/// Retourne `None` pour un système non reconnu : l'utilisateur est alors
/// renvoyé vers la page de la release.
pub fn extension_installeur() -> Option<&'static str> {
    if cfg!(target_os = "windows") {
        return extension_pour("windows", &[]);
    }
    if cfg!(target_os = "macos") {
        return extension_pour("macos", &[]);
    }
    if cfg!(target_os = "linux") {
        let ids = lire_os_release_ids();
        let referents: Vec<&str> = ids.iter().map(String::as_str).collect();
        return extension_pour("linux", &referents);
    }
    None
}

/// Extension correspondant à un couple système/famille, sans le point.
///
/// Fonction pure, testable indépendamment de la machine : `os` vaut
/// `"windows"`, `"macos"` ou `"linux"` ; `ids` porte les identifiants de
/// famille Linux (`ID` et `ID_LIKE` de `/etc/os-release`).
pub fn extension_pour(os: &str, ids: &[&str]) -> Option<&'static str> {
    match os {
        "windows" => Some("exe"),
        "macos" => Some("dmg"),
        "linux" => {
            let de = ids.iter().any(|id| *id == "debian" || *id == "ubuntu");
            let rhel = ids.iter().any(|id| *id == "fedora" || *id == "rhel" || *id == "centos");
            if de {
                Some("deb")
            } else if rhel {
                Some("rpm")
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Identifiants `ID` et `ID_LIKE` du système, en minuscules.
///
/// Se lit uniquement sur Linux ; ailleurs, la liste est vide. Une absence de
/// fichier ou une lecture impossible sont traitées comme une famille inconnue.
fn lire_os_release_ids() -> Vec<String> {
    let mut ids = Vec::new();
    if let Ok(contenu) = std::fs::read_to_string("/etc/os-release") {
        for ligne in contenu.lines() {
            if let Some((cle, valeur)) = ligne.split_once('=') {
                if cle == "ID" || cle == "ID_LIKE" {
                    let valeur = valeur.trim_matches('"');
                    for id in valeur.split_whitespace() {
                        ids.push(id.to_lowercase());
                    }
                }
            }
        }
    }
    ids
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn windows_et_macos_ont_une_extension_fixe() {
        assert_eq!(extension_pour("windows", &[]), Some("exe"));
        assert_eq!(extension_pour("macos", &[]), Some("dmg"));
    }

    #[test]
    fn les_familles_debian_et_ubuntu_preferent_deb() {
        assert_eq!(extension_pour("linux", &["debian"]), Some("deb"));
        assert_eq!(extension_pour("linux", &["ubuntu"]), Some("deb"));
        assert_eq!(extension_pour("linux", &["debian", "ubuntu"]), Some("deb"));
    }

    #[test]
    fn les_familles_redhat_preferent_rpm() {
        assert_eq!(extension_pour("linux", &["fedora"]), Some("rpm"));
        assert_eq!(extension_pour("linux", &["rhel"]), Some("rpm"));
        assert_eq!(extension_pour("linux", &["centos"]), Some("rpm"));
    }

    /// Une famille Debian prime sur une famille Red Hat si les deux sont
    /// présentes : l'ordre de `ID_LIKE` est signifiant chez les dérivés.
    #[test]
    fn la_famille_debian_prime() {
        assert_eq!(
            extension_pour("linux", &["debian", "fedora"]),
            Some("deb")
        );
    }

    #[test]
    fn une_famille_inconnue_ou_un_os_inconnu_retourne_rien() {
        assert_eq!(extension_pour("linux", &[]), None);
        assert_eq!(extension_pour("linux", &["arch", "nixos"]), None);
        assert_eq!(extension_pour("plan9", &[]), None);
    }
}
