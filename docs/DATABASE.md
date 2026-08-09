# DATABASE.md — Schéma SQLite

## Tables

### transactions
| Colonne | Type | Description |
|---|---|---|
| id | TEXT PK | UUID |
| kind | TEXT NOT NULL | income / expense |
| label | TEXT NOT NULL | Libellé |
| amount_cents | INTEGER NOT NULL > 0 | Montant en centimes |
| transaction_date | TEXT NOT NULL | YYYY-MM-DD |
| status | TEXT NOT NULL | pending / completed |
| category_id | TEXT NOT NULL FK | Référence catégorie |
| recurring_rule_id | TEXT NULL FK | Règle productrice |
| note | TEXT NULL | Note |
| created_at | TEXT NOT NULL | ISO UTC |
| updated_at | TEXT NOT NULL | ISO UTC |

### recurring_rules
| Colonne | Type | Description |
|---|---|---|
| id | TEXT PK | UUID |
| kind | TEXT NOT NULL | income / expense |
| label | TEXT NOT NULL | Libellé |
| amount_cents | INTEGER NOT NULL > 0 | Montant en centimes |
| category_id | TEXT NOT NULL FK | Référence catégorie |
| day_of_month | INTEGER NOT NULL 1..31 | Jour de matérialisation |
| start_year | INTEGER NOT NULL | Année de début |
| start_month | INTEGER NOT NULL 1..12 | Mois de début |
| end_year | INTEGER NULL | Année de fin |
| end_month | INTEGER NULL 1..12 | Mois de fin |
| note | TEXT NULL | Note |
| is_active | INTEGER NOT NULL | 0/1 |
| created_at | TEXT NOT NULL | ISO UTC |
| updated_at | TEXT NOT NULL | ISO UTC |

### categories
| Colonne | Type | Description |
|---|---|---|
| id | TEXT PK | Identifiant stable |
| kind | TEXT NOT NULL | income / expense |
| name | TEXT NOT NULL | Nom affiché |
| icon | TEXT NOT NULL | Icône |
| color | TEXT NOT NULL | #RRGGBB |
| sort_order | INTEGER NOT NULL | Ordre |
| is_default | INTEGER NOT NULL | 0/1 |
| is_active | INTEGER NOT NULL | 0/1 |

### app_settings
Une seule ligne (id=1) : solde, découvert, devise, thème, onboarding.

| Colonne | Type | Description |
|---|---|---|
| id | INTEGER PK | Toujours 1 |
| current_balance_cents | INTEGER NOT NULL | Solde courant (centimes) |
| overdraft_limit_cents | INTEGER NOT NULL >= 0 | Découvert autorisé (centimes) |
| currency_code | TEXT NOT NULL | Devise (EUR) |
| locale | TEXT NOT NULL | fr-FR |
| theme | TEXT NOT NULL | system / light / dark |
| balance_updated_at | TEXT NOT NULL | Dernière mise à jour du solde |
| onboarding_completed | INTEGER NOT NULL | 0/1 |
| last_export_date | TEXT NULL | Dernier export réussi |
| created_at | TEXT NOT NULL | ISO UTC |
| updated_at | TEXT NOT NULL | ISO UTC |

### schema_migrations
| Colonne | Type |
|---|---|
| version | INTEGER PK |
| name | TEXT |
| applied_at | TEXT |

## Index
- idx_transactions_date
- idx_transactions_kind_status
- idx_transactions_category
- idx_transactions_recurring
- idx_categories_kind
- idx_recurring_active

## Migrations
- `migrations/0001_initial.sql` — tables initiales, catégories par défaut (v1)
- `migrations/0002_recurrences.sql` — table `recurring_rules` (v2)
- `migrations/0003_last_export_date.sql` — colonne `last_export_date` (v3)
- Appliquées automatiquement au démarrage via `core/db/migrations.rs`
- Chaque migration est appliquée dans une transaction unique avec son
  enregistrement de version ; les ALTER sont rejoués uniquement si la colonne
  manque (idempotence).
- `INSERT OR IGNORE` pour l'idempotence des catégories
- `PRAGMA foreign_keys = ON` + `PRAGMA journal_mode = WAL`

## Catégories initiales
26 dépenses + 9 revenus insérés par la migration v1.
