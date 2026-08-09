//! Interrogation des releases GitHub d'AfterBudget.
//!
//! La release « latest » du dépôt public porte le tag `v<version>` et les
//! assets d'installation. Seul l'identifiant GitHub est requis : pas de token
//! pour un dépôt public (la limite anonyme de 60 requêtes/heure suffit pour
//! une vérification à chaque ouverture).

use serde::Deserialize;

use super::versions::Version;

/// Dernière release publiée, avec ses assets de téléchargement.
#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseInfo {
    pub version: Version,
    /// Page web de la release, ouverte en dernier recours quand aucun asset
    /// ne correspond au système.
    pub url_page: String,
    pub assets: Vec<AssetInfo>,
}

/// Un asset téléchargeable d'une release.
#[derive(Debug, Clone, PartialEq)]
pub struct AssetInfo {
    pub nom: String,
    pub url: String,
}

/// En-têtes minimaux exigés par l'API GitHub : sans `User-Agent` explicite,
/// la requête est refusée (403).
const NOM_APPLICATION: &str = concat!("afterbudget/", env!("CARGO_PKG_VERSION"));

/// Réponse de l'API, réduite aux champs utiles.
#[derive(Debug, Deserialize)]
struct ReleaseApi {
    tag_name: String,
    html_url: String,
    assets: Vec<AssetApi>,
}

#[derive(Debug, Deserialize)]
struct AssetApi {
    name: String,
    browser_download_url: String,
}

/// Dernière release disponible, si le dépôt en a publié une.
///
/// Les échecs réseau ou une réponse incompréhensible remontent une erreur
/// `String` : l'appelant choisit d'être silencieux (démarrage hors ligne).
pub async fn derniere_release() -> Result<Option<ReleaseInfo>, String> {
    let url = "https://api.github.com/repos/alexandrebouttierdev/afterbudget/releases/latest";
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .user_agent(NOM_APPLICATION)
        .build()
        .map_err(|e| format!("Client HTTP impossible : {e}"))?;

    let reponse = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("Requête de mise à jour impossible : {e}"))?;

    if reponse.status() == reqwest::StatusCode::NOT_FOUND {
        // Aucune release publiée : pas de mise à jour à proposer.
        return Ok(None);
    }
    let statut = reponse.status();
    if !statut.is_success() {
        return Err(format!("Le serveur a répondu {statut}."));
    }

    let texte = reponse
        .text()
        .await
        .map_err(|e| format!("Réponse de mise à jour illisible : {e}"))?;
    Ok(analyser_reponse(&texte))
}

/// Extrait la version et les assets d'une réponse GitHub, hors réseau.
///
/// Fonction pure pour permettre le test hors ligne ; `None` si la réponse ne
/// correspond pas au format attendu (y compris une version invalide).
pub fn analyser_reponse(json: &str) -> Option<ReleaseInfo> {
    let release: ReleaseApi = serde_json::from_str(json).ok()?;
    let version = Version::parse(&release.tag_name)?;
    Some(ReleaseInfo {
        version,
        url_page: release.html_url,
        assets: release
            .assets
            .into_iter()
            .map(|asset| AssetInfo {
                nom: asset.name,
                url: asset.browser_download_url,
            })
            .collect(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const REPONSE_EXEMPLE: &str = r#"{
        "tag_name": "v0.2.3",
        "html_url": "https://github.com/alexandrebouttierdev/afterbudget/releases/tag/v0.2.3",
        "assets": [
            {
                "name": "afterbudget-ubuntu-0.2.3.deb",
                "browser_download_url": "https://github.com/.../afterbudget-ubuntu-0.2.3.deb"
            },
            {
                "name": "afterbudget-windows-0.2.3.exe",
                "browser_download_url": "https://github.com/.../afterbudget-windows-0.2.3.exe"
            }
        ]
    }"#;

    #[test]
    fn une_reponse_complete_est_decodee() {
        let info = analyser_reponse(REPONSE_EXEMPLE).expect("réponse décodée");
        assert_eq!(info.version, Version::parse("v0.2.3").unwrap());
        assert_eq!(
            info.url_page,
            "https://github.com/alexandrebouttierdev/afterbudget/releases/tag/v0.2.3"
        );
        assert_eq!(info.assets.len(), 2);
        assert_eq!(info.assets[0].nom, "afterbudget-ubuntu-0.2.3.deb");
        assert_eq!(
            info.assets[0].url,
            "https://github.com/.../afterbudget-ubuntu-0.2.3.deb"
        );
    }

    #[test]
    fn un_json_incomplet_ou_illisible_retourne_rien() {
        assert_eq!(analyser_reponse("pas du json"), None);
        assert_eq!(analyser_reponse("{}"), None);
        assert_eq!(
            analyser_reponse(r#"{"tag_name": "zzz", "html_url": "x", "assets": []}"#),
            None,
            "un tag non semver ne doit pas produire de mise à jour"
        );
    }
}
