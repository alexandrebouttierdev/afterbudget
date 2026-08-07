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
| note | TEXT NULL | Note |
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
- idx_categories_kind

## Migrations
- Fichier SQL : `migrations/0001_initial.sql`
- Appliquées automatiquement au démarrage via `core/db/migrations.rs`
- `INSERT OR IGNORE` pour l'idempotence des catégories
- `PRAGMA foreign_keys = ON` + `PRAGMA journal_mode = WAL`

## Catégories initiales
26 dépenses + 9 revenus insérés par la migration v1.
