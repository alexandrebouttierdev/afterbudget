# Système de mise à jour — Plan d'implémentation

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal :** Vérifier à l'ouverture si une version plus récente d'AfterBudget existe (via GitHub Releases), afficher une bannière avec téléchargement de l'installeur adapté à l'OS, et mémoriser la version ignorée.

**Architecture :** Module isolé `src/modules/mise_a_jour/` (versions, client GitHub API, mapping plateforme, téléchargement), intégration via les messages Iced existants (`Task::perform`, pattern identique à `ImportResult`), bannière dans `src/ui/layout.rs`, migration SQL v4 pour la version ignorée, et publication versionnée dans `build.yml`.

**Tech stack :** Rust 1.87, Iced 0.13 (tokio), `reqwest 0.12` (features `json`, `rustls-tls`), `dirs` 5 (déjà présent), SQLite + migrations maison, GitHub Actions.

## Contraintes globales

- La version distante vient du tag `v<version>` de la release `latest` GitHub : `GET https://api.github.com/repos/alexandrebouttierdev/afterbudget/releases/latest` (header `User-Agent` obligatoire).
- Version locale : `env!("CARGO_PKG_VERSION")`.
- Échecs réseau silencieux → jamais bloquant ; erreur de téléchargement → notification `erreur`.
- Comparaison semver maison : `Version { majeur, mineur, correctif }`, pas de dépendance.
- Extensions par OS : Windows `.exe`, macOS `.dmg`, Linux `.deb` (Debian/Ubuntu) / `.rpm` (Fedora), sinon fallback « Voir la page ».
- Noms de variables, messages et commentaires en français (convention du codebase).

## Structure des fichiers

| Fichier | Responsabilité |
|---|---|
| `src/modules/mise_a_jour/mod.rs` (nouveau) | Re-exports du module |
| `src/modules/mise_a_jour/versions.rs` (nouveau) | `Version` : parse, comparaison, affichage |
| `src/modules/mise_a_jour/plateforme.rs` (nouveau) | Extension d'installeur selon l'OS |
| `src/modules/mise_a_jour/client.rs` (nouveau) | Requête GitHub API + parsing JSON |
| `src/modules/mise_a_jour/telechargement.rs` (nouveau) | Téléchargement + ouverture du fichier |
| `src/ui/composants/banniere_maj.rs` (nouveau) | Bannière de mise à jour |
| `migrations/0004_ignored_update_version.sql` (nouveau) | Colonne version ignorée |
| `Cargo.toml`, `src/main.rs`, `src/app/message.rs`, `src/app/state.rs`, `src/app/update.rs`, `src/ui/layout.rs`, `src/core/db/migrations.rs`, `src/domaine/parametres.rs`, `src/modules/parametres/service.rs`, `src/modules/parametres/repository.rs`, `src/modules/mod.rs`, `.github/workflows/build.yml` | Modifications |

---

### Task 1 : Parsing et comparaison de versions (`versions.rs`)

**Fichiers :** Créer `src/modules/mise_a_jour/versions.rs`, `src/modules/mise_a_jour/mod.rs` ; modifier `src/modules/mod.rs`.

**Interfaces :**
- Produces : `pub struct Version { pub majeur: u32, pub mineur: u32, pub correctif: u32 }` avec `Version::parse(&str) -> Option<Version>` (accepte `"v0.2.3"` et `"0.2.3"`), `Version::from_cargo() -> Version`, `impl Ord` (comparaison champ par champ), `impl Display` → `"0.2.3"`.

- [ ] **Step 1 : Test d'échec** — écrire les tests (parse valide/invalide, ordre `0.10.0 > 0.9.9`, `from_cargo` retourne la version Cargo, Display).
- [ ] **Step 2 : Exécuter** — `cargo test -p afterbudget mise_a_jour::versions` → échec (module inexistant).
- [ ] **Step 3 : Implémenter** — struct + parse (split sur `.`, retirer préfixe `v`, 3 composants numériques) + `Ord` dérivé via `#[derive(PartialEq, Eq, PartialOrd, Ord)]` + `from_cargo` via `env!("CARGO_PKG_VERSION")` + Display.
- [ ] **Step 4 : Déclarer les modules** — `pub mod mise_a_jour;` dans `src/modules/mod.rs` ; `pub mod versions; pub mod plateforme; pub mod client; pub mod telechargement;` dans `src/modules/mise_a_jour/mod.rs`.
- [ ] **Step 5 : Vérifier** — tests verts. Commit.

### Task 2 : Mapping OS → extension d'installeur (`plateforme.rs`)

**Interfaces :**
- Produces : `pub fn extension_installeur() -> Option<&'static str>` (détection réelle, via `cfg!(target_os)` + `/etc/os-release` sur Linux) et `pub fn extension_pour(os: &str, ids: &[&str]) -> Option<&'static str>` (pure, testable : `"linux"` + `["debian","ubuntu"]` → `"deb"` ; `["fedora","rhel"]` → `"rpm"` ; sinon `None` ; `"windows"` → `"exe"` ; `"macos"` → `"dmg"`).

- [ ] **Steps TDD** : tests d'abord (`extension_pour`), implémentation avec lecture de `/etc/os-release` (`ID=` + `ID_LIKE=`, casse insensible), `extension_installeur` = `extension_pour` avec les valeurs détectées. Commit.

### Task 3 : Client GitHub API (`client.rs`)

**Interfaces :**
- Consumes : `Version::parse` (Task 1).
- Produces : `pub struct ReleaseInfo { pub version: Version, pub url_page: String, pub assets: Vec<AssetInfo> }` ; `pub struct AssetInfo { pub nom: String, pub url: String }` ; `pub async fn derniere_release() -> Result<Option<ReleaseInfo>, String>` ; `pub fn analyser_reponse(json: &str) -> Option<ReleaseInfo>` (pure, testable sans réseau).

- [ ] **Step TDD 1** : test `analyser_reponse` avec un JSON GitHub réaliste en dur (champ `tag_name`, `html_url`, `assets[].name`/`browser_download_url`) → filtre les assets, parse le tag.
- [ ] **Step TDD 2** : implémentation — `serde::Deserialize` sur une struct privée minimale, URL API avec `reqwest::Client` + header `User-Agent: afterbudget/<version>` (requis par GitHub, sinon 403), timeout 10 s, `Ok(None)` si 404 (aucune release).
- [ ] **Step 3 : Ajouter la dépendance** — `Cargo.toml` : `reqwest = { version = "0.12", default-features = false, features = ["json", "rustls-tls"] }`.
- [ ] **Step 4 : Vérifier** — tests verts + `cargo check`. Commit.

### Task 4 : Version ignorée — migration, domaine, repository, service

**Fichiers :** Créer `migrations/0004_ignored_update_version.sql` ; modifier `src/core/db/migrations.rs`, `src/domaine/parametres.rs`, `src/modules/parametres/repository.rs`, `src/modules/parametres/service.rs`.

**Interfaces :** Produces : `AppSettings.ignored_update_version: Option<String>` (défaut `None`) ; `pub fn mettre_a_jour_version_ignoree(pool: &DatabasePool, version: &str) -> Result<(), String>` dans service.rs.

- [ ] **Step 1 : SQL** — `migrations/0004_ignored_update_version.sql` : `ALTER TABLE app_settings ADD COLUMN ignored_update_version TEXT;`
- [ ] **Step 2 : migrations.rs** — `VERSION_COURANTE = 4`, ajouter `(4, "v4", Box::new(migration_v4))`, `fn migration_v4` avec la garde `colonne_existe` (pattern v3) + `include_str!("../../../migrations/0004_ignored_update_version.sql")`. Test : `run_migrations` sur base neuve crée la colonne ; re-run idempotent.
- [ ] **Step 3 : Domaine** — champ + `Default`.
- [ ] **Step 4 : repository.rs** — ajouter `ignored_update_version` aux 3 requêtes (`SELECT` index 11 → `row.get(11)?`, `INSERT`, `UPDATE`) + assertions du test existant (ligne ~123).
- [ ] **Step 5 : service.rs** — `mettre_a_jour_version_ignoree` (pattern de `marquer_dernier_export`).
- [ ] **Step 6 : Vérifier** — `cargo test` (les tests d'aller-retour export→import AB-008 valident la compatibilité des anciennes bases). Commit.

### Task 5 : Téléchargement et ouverture (`telechargement.rs`)

**Fichiers :** Créer `src/modules/mise_a_jour/telechargement.rs`.

**Interfaces :** Produces : `pub async fn telecharger_installeur(url: &str, nom_fichier: &str) -> Result<std::path::PathBuf, String>` (télécharge dans `dirs::download_dir()` sinon le dossier personnel, écrit le fichier, retourne le chemin) ; `pub fn ouvrir_fichier(chemin: &std::path::Path) -> Result<(), String>` (lanceur OS : `xdg-open`/`open`/`start` — pattern identique à `ouvrir_lien` de `update.rs:880`).

- [ ] **Steps TDD** : test pur `nom_de_fichier_sur` (sanitize : enlever les caractères dangereux `/\:*?"<>|`), puis implémentation des deux fonctions. Commit.

### Task 6 : Flux applicatif — messages, état, update

**Fichiers :** Modifier `src/app/message.rs`, `src/app/state.rs`, `src/app/update.rs`.

**Interfaces :**
- Consumes : `derniere_release()`, `ReleaseInfo`, `Version`, `extension_installeur()`, `telecharger_installeur()`, `ouvrir_fichier()`, `mettre_a_jour_version_ignoree()`.
- Produces : messages `CheckForUpdates`, `UpdateCheckResult(Result<Option<UpdateInfo>, String>)`, `DownloadUpdate`, `DownloadResult(Result<std::path::PathBuf, String>)`, `IgnoreUpdate` ; dans `state.rs` : `pub struct UpdateInfo { pub version: Version, pub url_page: String, pub asset: Option<AssetInfo> }`, champs `pub update_info: Option<UpdateInfo>`, `pub update_downloading: bool` ; fonction pure testable `pub fn update_pertinente(locale: &Version, distante: &Version, ignoree: Option<&str>) -> bool`.

- [ ] **Step 1 : Messages** — ajouter les 5 variantes dans `message.rs` (section `// ---- Mise à jour ----`).
- [ ] **Step 2 : État** — `UpdateInfo` + champs + init dans `AppState::new()`.
- [ ] **Step 3 : Tests** — `update_pertinente` : distante > locale et non ignorée → true ; distante ≤ locale → false ; distante == ignorée → false ; ignorée autre version → true.
- [ ] **Step 4 : update.rs** — handlers :
  - `CheckForUpdates` : si `update_info` déjà affiché → `Task::none()` ; sinon `Task::perform(derniere_release(), Message::UpdateCheckResult)`.
  - `UpdateCheckResult(Ok(Some(release)))` : si `update_pertinente(Version::from_cargo(), &release.version, version_ignoree_de(state))` → construire `UpdateInfo` en choisissant l'asset par `extension_installeur()`, poser `state.update_info`.
  - `UpdateCheckResult(Ok(None))` / `Err` : silencieux.
  - `DownloadUpdate` : si asset présent → `state.update_downloading = true`, `Task::perform(telecharger_installeur(...), DownloadResult)`.
  - `DownloadResult(Ok(path))` : `ouvrir_fichier(&path)` ; succès → notification `succes("Téléchargé : <nom>. Lancement de l'installation.")`, `update_info = None` ; échec d'ouverture → notification erreur.
  - `DownloadResult(Err(e))` : notification erreur, `update_downloading = false`.
  - `IgnoreUpdate` : `mettre_a_jour_version_ignoree(db, &version)`, recharger `state.settings`, `update_info = None`.
- [ ] **Step 5 : Vérifier** — `cargo test`. Commit.

### Task 7 : Bannière UI

**Fichiers :** Créer `src/ui/composants/banniere_maj.rs` ; modifier `src/ui/composants/mod.rs`, `src/ui/layout.rs`.

- [ ] **Step 1 : Composant** — `pub fn banniere(state: &AppState) -> Option<Element<'static, Message>>` : rendu si `state.update_info` est `Some` — conteneur pleine largeur, fond de la palette, texte « Nouvelle version **X** disponible », boutons `Bouton` (pattern `src/ui/composants/bouton.rs`) : « Télécharger » (`Message::DownloadUpdate`, désactivé si `update_downloading` → « Téléchargement… ») ou « Voir la page » (ouvre `url_page` via `ouvrir_lien`), et « Ignorer » (`Message::IgnoreUpdate`).
- [ ] **Step 2 : Déclarer** dans `src/ui/composants/mod.rs`.
- [ ] **Step 3 : Intégration** — dans `layout.rs`, insérer la bannière dans la colonne `zone` entre `en_tete` et le corps.
- [ ] **Step 4 : Vérifier** — `cargo check` + `cargo clippy`. Commit.

### Task 8 : Déclenchement à l'ouverture (`main.rs`)

- [ ] **Step 1 : Modifier** — remplacer `.run_with(|| (state, iced::Task::none()))` par `.run_with(|| (state, iced::Task::done(Message::CheckForUpdates)))`.
- [ ] **Step 2 : Vérifier** — `cargo check` ; test manuel : app se lance sans erreur, une requête réseau part au démarrage. Commit.

### Task 9 : Publication versionnée dans `build.yml`

**Fichiers :** Modifier `.github/workflows/build.yml` (job `publish`, après « Assemble builds/ »).

- [ ] **Step 1 : Étape** — après l'étape « Assemble builds/ », ajouter l'étape « Publish versioned release » (script : `VERSION` depuis Cargo.toml, `TAG=v<VERSION>`, si le tag n'existe pas via `git ls-remote --tags`, renommer chaque asset `afterbudget-<plateforme>-<VERSION>.<ext>` et `gh release create "$TAG" ... --latest=false`).
- [ ] **Step 2 : Vérifier** — validation YAML. Commit.

---

## Vérification finale

`cargo fmt --all -- --check`, `cargo clippy --locked --all-targets --all-features -- -D warnings`, `cargo test --locked --all-targets --all-features` — tout doit être vert avant de terminer.
