# Spec — Remédiation complète de l'audit de production (AB-001 → AB-014)

**Date :** 2026-08-09
**Source :** `AUDIT_APP_REPORT.md` (constats AB-001 à AB-014)
**Périmètre décidé avec l'utilisateur :** tous les constats, P0 + P1 + P2.

## Objectif

Rendre AfterBudget distribuable : corriger les 14 constats de l'audit (persistance,
import, reset, arithmétique monétaire, Unicode, migrations, dépendances, mappers,
invariants métier, permissions, performances, architecture, démarrage, docs/CI)
sans régression des 224 tests existants, en ajoutant les tests de non-régression
attendus par l'audit.

## Contraintes globales

- **Invariant monétaire (exigence utilisateur) :** un solde de compte peut toujours
  être négatif (onboarding et paramètres) ; le montant d'une transaction ou d'une
  récurrence est **strictement positif**, le sens étant porté par le type.
- Montants stockés en **centimes `i64`** exclusivement, **jamais `f64`** sur un
  chemin de montant.
- Schéma : le binaire applique les migrations v1 → v2 → v3 ; les anciennes bases
  v1/v2 doivent être migrées sans intervention.
- Rust edition 2021 ; dépendances : Iced 0.13, rusqlite 0.31 (bundled/backup) ;
  `validator` passe en 0.19+ (idna 1.x) ; exception documentée pour `lru`
  (RUSTSEC-2026-0002, piné par iced_glyphon 0.6.0, aucun correctif upstream).
- Langue du code : français (messages, noms de tests, commentaires existants).
- TDD : test rouge → implémentation minimale → test vert, commit par constat.
- Portes finales : `cargo fmt --all -- --check`, `cargo clippy --locked
  --all-targets --all-features -- -D warnings`, `cargo test --locked
  --all-targets --all-features`, `cargo test --doc --locked --all-features`,
  `cargo build --locked --release --all-features`, `cargo audit --file Cargo.lock`.

## Architecture

Chaque constat est traité dans son module, en suivant l'architecture existante
(Vue → Message → DTO → validateur → commande → service → repository). Les
corrections s'appuient sur les commandes/services/repositories déjà en place ;
`src/app/update.rs` cesse d'accéder directement aux repositories et à la copie
de fichier d'import. Le domaine (`Money`, `Transaction`, `RecurringRule`) reste
sans dépendance Iced/SQLite.

Nouveaux fichiers attendus :
- `migrations/0003_last_export_date.sql` (colonne `last_export_date` sur `app_settings`)
- `.cargo/audit.toml` (exception lru justifiée)
- `.github/workflows/ci.yml` (portes Linux + compilation macOS/Windows)
- `docs/superpowers/specs/2026-08-09-remediation-audit-production.md` (ce document)

Fichiers modifiés principaux : `src/app/update.rs`, `src/core/config.rs`,
`src/core/db/pool.rs`, `src/core/db/migrations.rs`, `src/core/db/modeles.rs`,
`src/domaine/argent.rs`, `src/domaine/budget.rs`, `src/domaine/transaction.rs`,
`src/modules/{transactions,categories,recurrences,parametres,import_export}/{repository,service,validateurs,commandes}.rs`,
`src/modules/statistiques/service.rs`, `src/main.rs`, `Cargo.toml`,
`README.md`, `docs/ARCHITECTURE.md`, `docs/DATABASE.md`.

---

## AB-001 — Succès affiché malgré une erreur de persistance (ÉLEVÉ, bloquant)

### Comportement exigé

Une notification de succès n'apparaît que si l'écriture a été confirmée **et** que
le rechargement de l'état a réussi. Toute erreur SQLite (connexion non inscriptible,
ligne absente) produit une notification d'erreur et laisse l'état inchangé.

### Changements

1. `src/app/update.rs` — supprimer tous les `let _ =` et `.ok()` de mutation :
   - `UpdateBalance`, `UpdateOverdraft`, `SetCurrency` : propager le `Result` de
     `parametres_service::…`, puis recharger ; notification d'erreur sinon.
   - `ConfirmDeleteTransaction` : propager `tx_cmd::supprimer` ; succès seulement
     après suppression confirmée et rechargement.
   - `ToggleTransactionStatus` : propager `tx_cmd::changer_statut`.
   - `appliquer_theme` : propager `parametres_service::mettre_a_jour_theme`.
   - `ExportDatabase` : propager l'écriture de `last_export_date`.
   - `SubmitTransactionForm` : vérifier le rechargement avant la notification de
     succès (le `Result` de `state.load_month_data()` n'est plus ignoré).
2. Repositories (`transactions/repository.rs`, `categories/repository.rs`,
   `recurrences/repository.rs`, `parametres/repository.rs`) : `UPDATE`/`DELETE`
   retournent une erreur si `rows_affected == 0` (message français avec l'identifiant).
   `update_settings`/`insert_settings` : vérifient les lignes affectées.

### Tests de non-régression

- Connexion `PRAGMA query_only = ON` → `mettre_a_jour_solde` échoue, notification d'erreur, valeur en base inchangée.
- `supprimer` une transaction inexistante → `Err` (rows_affected = 0).
- `changer_statut` / `update` d'un id inexistant → `Err`.
- `update_settings` sur une base sans ligne id=1 → `Err`.

## AB-002 — L'import refuse la version de schéma livrée (CRITIQUE, bloquant)

### Comportement exigé

Un export produit par le binaire courant (version de schéma v2 après correction,
v3 après ajout) est réimportable. Les bases v1 restent importables (migrées au
chargement).

### Changements

1. `src/modules/import_export/service.rs` — `valider_import` :
   - version maximale acceptée = **version courante** (constante partagée exportée
     par `core/db/migrations.rs`, ex. `pub const VERSION_COURANTE: i64`).
   - tables requises étendues selon la version : `recurring_rules` requise pour
     v ≥ 2 ; `schema_migrations` toujours requise.
2. `migrations.rs` expose `VERSION_COURANTE` utilisée par le validateur.

### Tests

- Fixture SQLite v1 complète (tables v1, `schema_migrations` version 1) → acceptée.
- Fixture v2 (`recurring_rules` + `recurring_rule_id`) → acceptée.
- Base avec version > `VERSION_COURANTE` → refusée avec message explicite.
- Export réel du binaire courant → réimporté (test d'intégration).

## AB-003 — Remplacement d'import non atomique et message mensonger (CRITIQUE, bloquant)

### Comportement exigé

Un import qui échoue ne peut pas laisser la base partielle : remplacement par
copie temporaire + fsync + rename atomique, validation en lecture seule avant
toute mutation, rollback du backup sur échec. Le message « tes données sont
intactes » n'est affiché que si le rollback a abouti.

### Changements

1. `core/db/pool.rs` :
   - `validate_sqlite_file` : ne plus lire tout le fichier en mémoire ; lire
     uniquement les 16 premiers octets (l'en-tête). (contribue à AB-011)
   - nouvelle méthode `open_read_only(path)` : `SQLITE_OPEN_READ_ONLY`, ne crée
     ni WAL ni SHM.
2. `import_export/service.rs` — `importer(pool, source)` :
   - `valider_import` (lecture seule + `PRAGMA integrity_check` + tables + version) ;
   - backup du pool courant via `backup_to` ;
   - copie `source` → fichier temporaire `afterbudget.sqlite.tmp-<uuid>` dans le
     même répertoire que `current_path` ;
   - `File::sync_all` sur le temporaire ;
   - `std::fs::rename(temp, current_path)` (atomique même répertoire) ;
   - réouverture `DatabasePool::open` + `run_migrations` ;
   - **tout échec après le backup** : restaurer le backup par le même mécanisme
     (copie temporaire + rename) ; si la restauration échoue, erreur explicite
     signalant que le backup est disponible à `<chemin>`.
3. `src/app/update.rs` — `perform_import` supprimé ; `ConfirmImport` délègue à
   `io_cmd::importer` (en tâche de fond, voir AB-011). Succès seulement après
   réouverture et migrations. Le texte d'erreur reprend le statut du rollback.

### Tests

- Import d'un fichier non-SQLite → `Err`, base d'origine intacte (hash avant/après).
- `integrity_check` en échec (base corrompue) → `Err`, base intacte.
- Migration en échec après copie → rollback appliqué, base d'origine intacte.
- Import valide → données remplacées, réouverture OK, hash conforme à la source.

## AB-004 — Réinitialisation destructive non transactionnelle (CRITIQUE, bloquant)

### Comportement exigé

`reset_all_data` efface tout **ou rien** : une transaction unique, toutes les
tables (y compris `recurring_rules`), rollback complet sur la moindre erreur.
Après reset réussi : transactions 0, paramètres 0, règles 0, catégories
personnalisées 0, catégories par défaut conservées.

### Changements

1. `src/modules/parametres/repository.rs` — `reset_all_data` :
   - une `pool.conn.transaction()` unique ;
   - ordre FK sûr : `DELETE FROM recurring_rules` → `DELETE FROM transactions` →
     `DELETE FROM categories WHERE is_default = 0` → `DELETE FROM app_settings` ;
   - erreur → rollback (drop implicite) ; succès → commit.
2. `src/app/update.rs` — `ConfirmResetData` : après succès, rechargement vérifié.

### Tests

- Reset sans règle ni catégorie personnalisée → tous les compteurs attendus.
- Fixture avec catégorie personnalisée + règle + transaction (le scénario de
  l'audit, FK reproduite) → reset OK, compteurs corrects, aucune ligne résiduelle.
- Base en `PRAGMA query_only` → `Err`, aucune table modifiée (compteurs avant/après).
- Redémarrage après reset → onboarding à nouveau affiché.

## AB-005 — Arithmétique monétaire flottante et overflow non contrôlé (ÉLEVÉ, bloquant)

### Comportement exigé

Aucun montant ne passe par `f64`. `from_input` parse exactement en centimes
`i64`, refuse les dépassements. Les additions/soustractions/valeurs absolues
sont vérifiées (checked) et propagent une erreur en cas de débordement. Les
opérateurs `+`/`-` de `Money` sont supprimés au profit des méthodes checked.

### Changements

1. `src/domaine/argent.rs` :
   - `from_euros(f64)` **supprimé** ; `to_euros_f64` **supprimé** (tests et
     `montant_editable` passent en centimes / formatage direct).
   - `from_input` : parsing manuel — signe `-` en tête autorisé, espaces
     (ordinaire, `\u{00a0}`, `\u{202f}`) ignorés, un séparateur décimal
     `,` ou `.`, au plus 2 décimales (au-delà refusé, pas arrondi), conversion
     en centimes par `checked_mul`/`checked_add`, erreur « Le montant est trop
     grand. » en cas de débordement. Aucune conversion flottante.
   - `abs()` → `checked_abs(&self) -> Option<Self>` ; `Add`/`Sub`/`AddAssign`/
     `SubAssign` retirés ; `checked_add`, `checked_sub` ajoutés.
   - `format_fr` : inchangé (déjà exact en centimes) ; vérifier que
     `i64::MIN` ne panique pas dans `format_fr` (abs des centimes — utilisation
     de `unsigned_abs`).
2. Appels d'opérateurs remplacés :
   - `src/domaine/budget.rs` — `BudgetSummary::compute` : calculs via
     `checked_add`/`checked_sub`, retourne `Result<Self, String>` ;
   - `src/domaine/transaction.rs` — `apply_to_balance` et `signed_amount` via
     `checked_*` ;
   - `src/modules/statistiques/service.rs` — additions/soustractions vérifiées
     (propagation d'erreur) ;
   - `src/modules/budget/service.rs` et commandes : propagation.
3. `src/app/update.rs` — `montant_editable` : formate les centimes en chaîne
   (partie entière, virgule, deux décimales) sans `f64`.
4. Invariants conservés et verrouillés par tests :
   - solde négatif toujours accepté : `from_input("-478") == -47800` ;
   - transaction strictement positive : `valider_montant` (`commun.rs`) conserve
     `cents <= 0` → erreur ; champ `filtrer_saisie` ignore le signe ;
     `CHECK(amount_cents > 0)` conservé dans les tables `transactions` et
     `recurring_rules` ; `overdraft_limit_cents >= 0` conservé.

### Tests

- Parsing : `"10"` → 1000, `"10,50"` → 1050, `"10.50"` → 1050, `"-478"` → −47800,
  `"1 207,50"` / `"1\u{202f}207,50"` / NBSP → 120750, `"1e5"` → erreur,
  `"12,345"` → erreur (3 décimales), `""`/`"abc"` → erreur.
- Bornes : `i64::MAX`/`i64::MIN` en centimes acceptés au parsing des valeurs
  maxi ; une valeur qui déborde `i64` → erreur.
- `checked_add`/`checked_sub`/`checked_abs` : dépassement → `None`.
- `format_fr` sur `Money::from_cents(i64::MIN)` → ne panique pas.
- `valider_montant("-10")`/`valider_montant("0")` → `Err` ; `valider_solde("-360")`
  → OK. Insertion SQL de `amount_cents <= 0` → contrainte CHECK rejetée.
- Budget/statistiques : pas de régression (valeurs nominales inchangées).

## AB-006 — Panic sur note Unicode longue (ÉLEVÉ, bloquant)

### Comportement exigé

Aucun panic sur une note de plus de 1000 octets, quelle que soit la frontière
UTF-8. La limite est exprimée en caractères (cohérente avec le message
« 1 000 caractères »).

### Changements

1. `src/app/update.rs` (`SubmitTransactionForm`) : tronquage par
   `note.chars().take(1000).collect::<String>()`.
2. `src/modules/transactions/validateurs.rs` : `chars().count() > 1000` au lieu
   de `len() > 1000`.

### Tests

- Note ASCII de 1000 caractères → OK ; 1001 → erreur (validateur).
- Note avec 250 emojis (1000 octets+) → pas de panic, tronquage à 1000
  caractères (scénario exact de l'audit : `"a" + 250 × "😀"`).
- Note exactement à la limite et frontière multioctet.

## AB-007 — Migrations non atomiques et v2 non idempotente (ÉLEVÉ, bloquant)

### Comportement exigé

Chaque migration (SQL + enregistrement de version) est atomique : échec de l'un
ou l'autre → rien n'est appliqué, le prochain démarrage rejoue proprement.
Chaque migration est idempotente (base rejouable sans erreur).

### Changements

1. `src/core/db/migrations.rs` :
   - `MigrationFn` prend `&rusqlite::Transaction` ; le SQL et
     `INSERT schema_migrations` s'exécutent dans la même transaction,
     commit unique ;
   - `migration_v2` : garde avant l'`ALTER TABLE` — vérifier
     `PRAGMA table_info(transactions)` ; si `recurring_rule_id` existe déjà,
     ne pas rejouer l'ALTER (permet de réparer les bases laissées par le bug
     AB-007 : ALTER appliqué mais version non enregistrée) ;
   - `migration_v3` : ajout de la colonne `last_export_date` avec la même garde
     (`PRAGMA table_info(app_settings)`) ;
   - `VERSION_COURANTE` constante publique (utilisée par AB-002).
2. Les fichiers SQL v1/v2/v3 n'embarquent pas leur propre gestion de version.

### Tests

- Base vierge → v3, tables + colonnes attendues.
- Reprise sur base v1 et sur base v2 → v3 sans erreur.
- Base « cassée » (colonnes déjà présentes, version non enregistrée) → `run_migrations` OK.
- Échec injecté (ex. contrainte impossible dans le SQL) → transaction annulée,
  version non enregistrée ; nouvelle exécution → succès (ou erreur propre).
- Idempotence : `run_migrations` deux fois → second appel sans effet.

## AB-014 — Porte RustSec rouge (MOYEN, bloquant en production)

### Changements

1. `Cargo.toml` : `validator = { version = "0.19", features = ["derive"] }` ;
   `cargo update -p validator` (idna 0.5.0 → 1.x, corrige RUSTSEC-2024-0421).
   Vérifier la compilation des dérivés `#[derive(Validate)]` existants.
2. `.cargo/audit.toml` :
   ```toml
   [advisories]
   ignore = ["RUSTSEC-2026-0002"]  # lru 0.12.5 (unsound) via iced_glyphon 0.6.0,
                                   # piné par iced 0.13 (dernière version) ;
                                   # aucun usage d'IterMut dans le graphe AfterBudget.
   ```
3. Documenter l'exception dans `README.md` (section dépendances).

### Tests

- `cargo audit --file Cargo.lock` → code 0.
- `cargo build` → OK avec validator 0.19.

## AB-008 — Mapping silencieux de données invalides (ÉLEVÉ)

### Comportement exigé

Une ligne SQLite invalide (enum, date, timestamp) fait échouer le chargement
avec un message contextualisé (identifiant de ligne), jamais une valeur par
défaut.

### Changements

1. `src/modules/transactions/repository.rs` : `row_to_transaction` →
   `Result<Transaction, String>` ; kind/date/statut/timestamps invalides →
   `Err(format!("Transaction {} : …", id))`. `find_by_month`, `find_recent_by_month`,
   `find_by_id`, `insert_from_rule` propagent.
2. `src/modules/categories/repository.rs` : `row_to_category` → `Result`,
   même traitement.
3. `src/modules/recurrences/repository.rs` : `depuis_ligne` → `Result`
   (genre inconnu, horodatages invalides).

### Tests

- Ligne avec `kind='transfert'` → `Err` ; date `'2026-13-99'` → `Err` ;
  statut inconnu → `Err` ; timestamp invalide → `Err` ; l'erreur contient l'id.

## AB-009 — Invariants de catégorie et lignes affectées non vérifiés (ÉLEVÉ)

### Comportement exigé

Une transaction/récurrence ne peut référencer qu'une catégorie existante,
active, et du même `kind` que son type. Une mise à jour/suppression qui ne
touche aucune ligne est une erreur.

### Changements

1. `src/modules/transactions/service.rs` (`creer_transaction`,
   `modifier_transaction`) : vérifier via `categories::repository::find_by_id` —
   existence, `is_active`, `kind == type` ; erreurs françaises explicites.
2. `src/modules/recurrences/service.rs` (`creer`) : même vérification.
3. Repositories : `update`, `delete_by_id`, `supprimer` (récurrences),
   `changer_statut` vérifient `rows_affected`.

### Tests

- Matrice income/expense × catégorie active/inactive/inconnue/mauvais kind :
  seules les combinaisons valides passent.
- Récurrence sur catégorie inactive → `Err`.

## AB-010 — Permissions locales et exposition de chemin (MOYEN)

### Changements

1. `src/core/config.rs` : après création, chmod du répertoire à `0700`
   (`cfg(unix)`, `std::os::unix::fs::PermissionsExt`). `app_data_dir` retourne
   `Result<PathBuf, String>` (voir AB-013).
2. `src/core/db/pool.rs` : après ouverture, chmod du fichier DB à `0600`
   (`cfg(unix)`) ; le log `tracing::info!` du chemin de la base passe en
   `debug!` (ou suppression), et ne contient plus le chemin en `info`.
3. `README.md` : documenter le modèle de confidentialité (fichiers `0600`,
   répertoire `0700`, pas de chiffrement, Windows sans mode POSIX).

### Tests (`#[cfg(unix)]`)

- Après `open()`, le fichier DB a le mode `0600` ; après `app_data_dir()`,
  le répertoire a le mode `0700`.

## AB-011 — Croissance et blocage probables sur grosses bases (MOYEN) — approche pragmatique

### Changements

1. **Import en tâche de fond** : `Message::ConfirmImport` → `Task::perform`
   (Iced 0.13) d'une fonction asynchrone qui exécute validation + backup +
   copie temp + rename hors du thread UI ; un `Message::ImportResult(Result<…>)`
   applique le résultat (réouverture du pool, rechargement, notifications).
   `state.db` reste utilisable pendant l'import.
2. **Statistiques sans chargement complet** : `src/modules/statistiques/service.rs`
   — remplacer l'appel à `find_by_month` (qui charge toutes les transactions)
   par deux `COUNT(*)` SQL (`count_by_kind` ajouté à `transactions/repository.rs`)
   pour `transaction_count`/`income_count`/`expense_count`.
3. `validate_sqlite_file` ne lit plus le fichier entier (fait en AB-003).
4. **Benchmark de non-régression** : test `#[ignore]` dans
   `src/modules/transactions/tests/mod.rs` (ou `tests/benchmark.rs`) : base
   synthétique 100 000 transactions, mesure du temps de `find_by_month` +
   `calculer_statistiques_mensuelles` ; seuil large (ex. < 2 s) pour rester
   stable en CI ; exécutable via `cargo test -- --ignored`.

### Tests

- Benchmark ignore : `cargo test -- --ignored perf_` → passe sur machine locale.
- Statistiques : résultats identiques avant/après refactor (valeurs nominales).

## AB-012 — Architecture et DTO déclarés mais contournés (MOYEN)

### Changements

1. `src/app/update.rs` :
   - `OpenEditTransaction` / `DeleteTransaction` : `find_by_id` via une commande
     `tx_cmd::trouver(id)` (nouvelle) au lieu de `transaction_repo` direct ;
   - `ExportDatabase` : `last_export_date` via `params_cmd`/`service` (pas de
     `params_repo` direct) ;
   - `ConfirmImport` : délègue à `io_cmd::importer` (AB-003) ;
   - `appliquer_theme` : `parametres_cmd::mettre_a_jour` ou service dédié ;
   - plus aucun accès repository direct dans `update.rs` (hors `state.db`).
2. `src/modules/categories/validateurs.rs` : `valider_modification` construit
   le bon DTO (le `type_categorie` du DTO de modification est conservé au lieu
   d'une chaîne vide).
3. DTO/validateurs/mappers morts — vérifié par recherche d'usage : aucun chemin
   métier ne référence `CreerCategorieDto`, `ModifierCategorieDto`,
   `ImporterSauvegardeDto`, ni `categories::validateurs`. Décision : **supprimer**
   `src/modules/categories/dtos/` (creer_categorie.rs, modifier_categorie.rs),
   `src/modules/categories/validateurs.rs`, `src/modules/categories/mappers.rs`
   (vide), `src/modules/import_export/dtos/`, `src/modules/import_export/mappers.rs`
   (vide) et les `mod` correspondants ; cela supprime le défaut du validateur
   « type vide » par élimination du code mort. Vérifier aussi
   `categories::commandes::par_type` : supprimé si inutilisé.
4. Documents réalignés (voir P2 docs) : le flux décrit correspond au code.

### Tests

- Les tests existants passent (aucun usage rompu) ; `rg` ne montre plus d'accès
  repository/SQL dans `update.rs`.

## AB-013 — Démarrage fragile sur erreurs de chemin ou de migration (MOYEN)

### Changements

1. `src/core/config.rs` : `app_data_dir() -> Result<PathBuf, String>` (l'échec de
   `create_dir_all` est une erreur propagée avec message) ;
   `database_path() -> Result<PathBuf, String>` ; `backup_dir()` pareil.
2. `src/main.rs` : `DatabasePool::open` et `run_migrations` → `tracing::error!`
   + `eprintln!` + `std::process::exit(1)` avec un message contextualisé au lieu
   de `panic!` ; le chargement initial échoué (`state.load_data`) → notification
   d'erreur dans l'UI si possible (message affiché) sans continuer en état partiel.
3. `AppState::new` et autres appels de `database_path` adaptés au `Result`.

### Tests

- Unité : `app_data_dir` retourne une erreur propre quand le répertoire est
  inutilisable (chemin sous un fichier, si testable) ou vérification du message.
- `main` : non testable E2E ; vérifier compilation et `cargo run` sur base saine.

## P2 — `last_export_date` persisté

### Changements

1. `migrations/0003_last_export_date.sql` :
   `ALTER TABLE app_settings ADD COLUMN last_export_date TEXT;`
   (la garde idempotente est dans `migrations.rs`, voir AB-007).
2. `src/core/db/modeles.rs` : `SettingsRow` + colonne `last_export_date`.
3. `src/modules/parametres/repository.rs` : lecture/écriture de la colonne ;
   `row_to_settings` lit la valeur réelle (plus de `None` forcé).
4. `src/app/update.rs` (ExportDatabase) : écrit via le service, erreur propagée (AB-001).

### Tests

- Export → `last_export_date` non nulle en base ; relecture via
  `get_settings` → valeur présente après redémarrage simulé (nouvelle connexion).

## P2 — Cargo.toml, CI, documentation

### Cargo.toml

- `rust-version = "1.85"` (ou version minimale vérifiée) ;
- `[profile.release]` : `lto = "thin"`, `codegen-units = 1`, `strip = "symbols"` ;
- metadata : repository, homepage, keywords.

### `.github/workflows/ci.yml`

- Job Linux (ubuntu-latest) : `cargo fmt --check`, `cargo clippy --locked
  --all-targets --all-features -- -D warnings`, `cargo test --locked
  --all-targets --all-features`, `cargo test --doc --locked --all-features`,
  `cargo build --locked --release --all-features`, `cargo audit` (avec
  `.cargo/audit.toml`).
- Jobs de **compilation** macOS (macos-latest) et Windows (windows-latest) :
  `cargo build --release` (les portes de test restent Linux, documenté).

### Documentation

- `README.md` : section dépendances + exception lru ; modèle de confidentialité ;
  mise à jour des commandes de vérification.
- `docs/ARCHITECTURE.md` : aligner le flux décrit (Vue → Message → DTO →
  validateur → commande → service → repository) avec le code réel ; préciser le
  rôle de `update.rs` (orchestration d'état uniquement) ; retirer la promesse de
  DTO/catégories si les fichiers morts sont supprimés.
- `docs/DATABASE.md` : liste v1, v2, v3 et la colonne `last_export_date`.

---

## Ordre d'implémentation et commits

1. AB-006 (isolé, rapide) — commit `fix: tronquer la note par frontière de caractère (AB-006)`
2. AB-005 (Money) — commit `fix: arithmétique monétaire entière sans f64 (AB-005)`
3. AB-001 + AB-009 rows_affected — commit `fix: propager les erreurs de persistance (AB-001, AB-009)`
4. AB-008 (mappers stricts)
5. AB-009 invariants catégorie
6. AB-007 + migration v3 — commit migrations transactionnelles
7. AB-002 + AB-003 + AB-011 import (import sûr, en tâche de fond)
8. AB-004 reset transactionnel
9. AB-010 + AB-013 (permissions, démarrage)
10. AB-011 statistiques COUNT + benchmark
11. AB-012 architecture + DTO morts
12. P2 : last_export_date, Cargo.toml, audit.toml, CI, docs
13. Portes finales : fmt, clippy, tests, doc-tests, release, audit.

## Critères de réussite

- Les 14 constats traités avec leur test de non-régression associé.
- Portes finales vertes (liste « Contraintes globales »).
- Invariant « solde négatif oui, transaction jamais » vérifié par tests.
- Aucun fichier hors périmètre modifié sans justification ; commits par constat.
