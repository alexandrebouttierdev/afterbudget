# ARCHITECTURE.md — Architecture modulaire d'AfterBudget

## Structure des modules

Chaque module métier contient tout ce qui le concerne :

```
modules/
├── commun.rs              — règles de validation transversales
├── budget/
│   ├── mod.rs
│   ├── commandes.rs       — orchestration des cas d'usage
│   ├── mappers.rs         — mapping entre couches
│   ├── service.rs         — logique métier (pas de SQL)
│   ├── views/
│   │   └── index.rs       — écran tableau de bord
│   └── tests/
│       └── mod.rs         — tests d'intégration
├── categories/
│   ├── mod.rs
│   ├── commandes.rs
│   ├── mappers.rs
│   ├── dtos/
│   │   ├── mod.rs
│   │   ├── creer_categorie.rs
│   │   └── modifier_categorie.rs
│   ├── validateurs.rs     — validation des DTOs
│   ├── service.rs
│   ├── repository.rs      — requêtes SQL
│   ├── composants/
│   └── tests/
├── import_export/
│   ├── mod.rs
│   ├── commandes.rs
│   ├── mappers.rs
│   ├── dtos/
│   │   ├── mod.rs
│   │   └── importer_sauvegarde.rs
│   ├── service.rs
│   └── tests/
├── onboarding/
│   ├── mod.rs
│   ├── commandes.rs
│   ├── mappers.rs
│   ├── dtos/
│   │   ├── mod.rs
│   │   └── terminer_onboarding.rs
│   └── views/
│       └── index.rs
├── parametres/
│   ├── mod.rs
│   ├── commandes.rs
│   ├── mappers.rs
│   ├── dtos/
│   │   ├── mod.rs
│   │   └── modifier_parametres.rs
│   ├── validateurs.rs
│   ├── service.rs
│   ├── repository.rs
│   └── views/
│       └── index.rs
├── statistiques/
│   ├── mod.rs
│   ├── commandes.rs
│   ├── mappers.rs
│   ├── service.rs
│   └── views/
│       └── index.rs
└── transactions/
    ├── mod.rs
    ├── commandes.rs
    ├── mappers.rs
    ├── dtos/
    │   ├── mod.rs
    │   ├── creer_transaction.rs
    │   ├── modifier_transaction.rs
    │   └── filtrer_transactions.rs
    ├── validateurs.rs
    ├── service.rs
    ├── repository.rs
    ├── composants/
    │   ├── mod.rs
    │   ├── filtre.rs
    │   └── ligne_transaction.rs
    ├── views/
    │   ├── mod.rs
    │   ├── index.rs           — liste des transactions
    │   └── formulaire.rs      — formulaire ajout/modification
    └── tests/
        └── mod.rs
```

## Couches transversales

```
core/               — noyau technique
  config.rs         — chemins multiplateformes
  erreurs.rs        — hiérarchie thiserror
  evenements.rs     — événements clavier
  utils.rs          — formatage, helpers
  db/
    modeles.rs      — structs représentant les lignes SQLite
    pool.rs         — connexion, backup, validation
    migrations.rs   — migrations + catégories initiales

domaine/            — modèles métier purs (sans Iced, sans SQL)
  argent.rs         — Money (centimes i64)
  budget.rs         — BudgetSummary, FinancialStatus, MonthlyStatistics
  categorie.rs      — Category, DefaultCategory
  parametres.rs     — AppSettings
  transaction.rs    — Transaction, TransactionKind, TransactionStatus

ui/                 — interface Iced transversale
  theme/            — couleurs, espacements, typographie, styles
  composants/       — bouton, carte, badge, modal, notification, navigation, etc.
  layout.rs         — layout principal (sidebar + contenu)
  gestionnaire_ecrans.rs — routage des écrans

app/                — état global Iced
  message.rs        — enum Message, Screen
  state.rs          — AppState
  update.rs         — traitement des messages
  view.rs           — vue principale, dispatch vers les écrans

migrations/         — fichiers SQL
  0001_initial.sql
```

## Circulation d'une action

```
Utilisateur → Iced → Message → app/update.rs
  → construit un DTO (modules/*/dtos/)
  → valide via validateurs.rs du module
  → appelle commandes.rs du module
  → appelle service.rs du module
  → appelle repository.rs du module
  → SQLite
  → résultat → Message → app/state.rs mis à jour → Iced redessine
```

## Règles de dépendance

```
ui/composants/  →  (pas de dépendance SQL)
modules/*/views/  →  modules/*/composants/  →  (pas de SQL)
modules/*/commandes  →  modules/*/service  →  modules/*/repository  →  SQLite
```

- Les vues ne contiennent aucun SQL
- Les services ne contiennent aucun SQL brut
- Les DTOs sont séparés des modèles SQLite
- Les validateurs sont dans chaque module
- Le domaine est pur, sans dépendance Iced ni SQLite
