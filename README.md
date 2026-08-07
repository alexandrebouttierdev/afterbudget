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
| Autres | `serde`, `chrono`, `uuid`, `tracing`, `validator`, `thiserror`, `dirs`, `rfd` |

## Prérequis

- Rust stable 1.75+
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

## Emplacement de la base SQLite

| OS | Emplacement |
|---|---|
| Linux | `~/.local/share/afterbudget/` |
| macOS | `~/Library/Application Support/afterbudget/` |
| Windows | `%APPDATA%/afterbudget/` |

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

L'architecture est **modulaire** : chaque module (`transactions/`, `budget/`, `categories/`, `parametres/`,
`statistiques/`, `import_export/`, `onboarding/`) contient ses DTOs, validateurs, composants UI,
écrans, service, repository et tests. La logique métier est séparée du SQL et des widgets.

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
| Linux | Compilé |
| macOS | Non testé |
| Windows | Non testé |

## Confidentialité

Aucun revenu, dépense ou solde n'est transmis sur Internet. Les données restent sur votre
ordinateur, dans une base SQLite locale.