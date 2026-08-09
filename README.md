# AfterBudget

**Application desktop de gestion de budget personnel.**

AfterBudget permet de connaître à tout moment votre **solde prévisionnel de fin de mois** :
`solde actuel + revenus en attente − dépenses en attente`, avec un suivi du découvert autorisé.

## Fonctionnalités principales

- Tableau de bord avec solde prévisionnel et statut financier (En forme / Attention / Danger)
- Création, modification et suppression de revenus et dépenses
- Transactions réalisées ou en attente, classées par catégorie
- Liste filtrable (type, statut, catégorie, mois, recherche)
- Statistiques mensuelles par catégorie
- Paramètres : solde, découvert, devise, thème clair/sombre
- Stockage **100 % local** : aucune donnée transmise sur Internet, aucun compte
- Export / import de la base SQLite
- Navigation complète au clavier

## Technologies

| Brique | Choix |
|---|---|
| Langage | **Rust** (édition 2021) |
| Interface | **Iced 0.13** (native) |
| Base de données | **SQLite** via `rusqlite` (bundled) |
| Autres | `serde`, `chrono`, `uuid`, `tracing`, `validator 0.19+`, `thiserror`, `dirs`, `rfd` |

## Prérequis

- Rust stable 1.87+ (le champ `rust-version` du manifeste est tenu à jour)
- Dépendances système Iced (Linux) : `pkg-config libx11-dev libxkbcommon-dev libfontconfig-dev`

## Compilation et lancement

```bash
cargo build --release
cargo run
```

## Tests

```bash
cargo test --all-targets --all-features
```

## Dépendances et sécurité

- `cargo audit` est vert (`.cargo/audit.toml`).
- Exception documentée : `lru 0.12.5` (RUSTSEC-2026-0002) est épinglée par
  `iced_glyphon 0.6.0` (Iced 0.13, dernière version) ; la fonctionnalité
  concernée (`IterMut`) n'est pas utilisée par AfterBudget.

## Emplacement de la base SQLite

| OS | Emplacement |
|---|---|
| Linux | `~/.local/share/afterbudget/` |
| macOS | `~/Library/Application Support/afterbudget/` |
| Windows | `%APPDATA%/afterbudget/` |

## Démonstration et captures d'écran

Une base SQLite de démonstration, remplie de **fausses données réalistes**
(carnet fictif utilisé de janvier à août 2026, compte en fin de mois dans le
découvert autorisé), est générée par un script : aucune donnée réelle n'est
jamais lue ni modifiée.

```bash
# Recrée la base de démonstration (fichiers dans demo/, ignorés par git)
scripts/seed_demo_db.sh

# Variante « premier lancement » (écran de bienvenue)
scripts/seed_demo_db.sh --onboarding

# Lance l'application avec la base de démonstration
scripts/demo.sh
scripts/demo.sh --onboarding
```

Le générateur est `src/bin/seed_demo.rs` : il réutilise les migrations et le
schéma réels de l'application (`cargo run --bin seed_demo [chemin] [--onboarding]`).
L'application pointe vers la base de démonstration via `XDG_DATA_HOME`
(position par défaut : `demo/afterbudget/afterbudget.sqlite`).

Pour les captures du site (`docs/assets/screenshots/`) : fenêtre par défaut
1240 × 780, thème clair, navigation au clavier (`Ctrl 1`…`4`, `Ctrl N`), puis
enregistrer les PNG sous les noms attendus (voir `docs/assets/screenshots/README.md`).

## Documentation

La documentation détaillée est rassemblée dans [`docs/`](docs/) :

| Document | Contenu |
|---|---|
| [SPEC_AFTERBUDGET.md](docs/SPEC_AFTERBUDGET.md) | Spécifications fonctionnelles et techniques (calculs, écrans, exigences FR/NFR) |
| [ARCHITECTURE.md](docs/ARCHITECTURE.md) | Architecture modulaire, couches, circulation d'une action, règles de dépendance |
| [DATABASE.md](docs/DATABASE.md) | Schéma SQLite : tables, index, migrations, catégories initiales |
| [DESIGN_SYSTEM.md](docs/DESIGN_SYSTEM.md) | Design system « Encre & Cuivre » : couleurs, typographie, composants, accessibilité |
| [DECISIONS.md](docs/DECISIONS.md) | Décisions techniques et leurs justifications |

## Modules

L'architecture est **modulaire**. Liste exacte des modules et de leurs fichiers réels
(sous `src/modules/`) :

- `commun.rs` — règles de validation transversales (partagées)
- `transactions/` — `mod.rs`, `commandes.rs`, `mappers.rs`, `dtos/` (`creer_transaction.rs`, `modifier_transaction.rs`, `filtrer_transactions.rs`), `validateurs.rs`, `service.rs`, `repository.rs`, `composants/` (`mod.rs`, `filtres.rs`, `ligne_transaction.rs`), `views/` (`mod.rs`, `index.rs`, `formulaire.rs`, `suppression.rs`), `tests/` (`mod.rs`, `commun.rs`)
- `budget/` — `mod.rs`, `commandes.rs`, `mappers.rs`, `service.rs`, `composants/` (`mod.rs`, `bloc_solde.rs`, `indicateur.rs`), `views/` (`mod.rs`, `index.rs`), `tests/` (`mod.rs`, `commun.rs`)
- `categories/` — `mod.rs`, `commandes.rs`, `service.rs`, `repository.rs`, `composants/` (`mod.rs`, `pastille.rs`), `tests/` (`mod.rs`, `commun.rs`)
- `parametres/` — `mod.rs`, `commandes.rs`, `mappers.rs`, `dtos/` (`modifier_parametres.rs`), `validateurs.rs`, `service.rs`, `repository.rs`, `composants/` (`mod.rs`, `section.rs`), `views/` (`mod.rs`, `index.rs`, `reinitialisation.rs`), `tests/` (`mod.rs`, `commun.rs`)
- `statistiques/` — `mod.rs`, `commandes.rs`, `mappers.rs`, `service.rs`, `views/` (`mod.rs`, `index.rs`)
- `import_export/` — `mod.rs`, `commandes.rs`, `service.rs`, `views.rs`, `tests/` (`mod.rs`)
- `onboarding/` — `mod.rs`, `commandes.rs`, `mappers.rs`, `dtos/` (`terminer_onboarding.rs`), `views/` (`mod.rs`, `index.rs`)
- `recurrences/` — `mod.rs`, `commandes.rs`, `dtos/` (`creer_recurrence.rs`), `validateurs.rs`, `service.rs`, `repository.rs`, `tests.rs`

La logique métier est séparée du SQL et des widgets.

Voir [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) pour le détail.

## Raccourcis clavier

| Action | Raccourci |
|---|---|
| Nouvelle dépense | `Ctrl` `N` |
| Nouveau revenu | `Ctrl` `Maj` `N` |
| Aller à la recherche | `Ctrl` `F` |
| Enregistrer le formulaire ouvert | `Ctrl` `S` |
| Fermer la modale / le formulaire / la notification | `Échap` |
| Mois précédent / suivant | `Ctrl` `←` / `Ctrl` `→` |
| Accueil / Transactions / Statistiques / Paramètres | `Ctrl` `1`…`4` |

## Compatibilité

| Plateforme | Statut |
|---|---|
| Linux | Compilé, testé et linté en CI (fmt, clippy, tests, doc-tests, audit) |
| macOS | Compilé en CI (job de compilation seule) ; les tests ne s'exécutent que sur Linux |
| Windows | Compilé en CI (job de compilation seule) ; les tests ne s'exécutent que sur Linux |

## Confidentialité

Aucun revenu, dépense ou solde n'est transmis sur Internet. Les données restent sur votre
ordinateur, dans une base SQLite locale.

Les fichiers locaux sont protégés : répertoire de données en `0700` et base
SQLite en `0600` sur Linux/macOS (pas de mode POSIX sous Windows). La base
n'est pas chiffrée : tout utilisateur local ayant accès au système peut la
lire s'il a les droits sur le répertoire.
