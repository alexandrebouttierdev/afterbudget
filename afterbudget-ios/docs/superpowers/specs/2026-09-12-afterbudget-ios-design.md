# AfterBudget iOS — Spec de design

**Date :** 2026-09-12  
**Statut :** validé en conversation, en attente de review du fichier  
**Emplacement :** `Documents/Mes projets/AfterBudget/afterbudget-ios/`

## 1. Objectif

Recréer **AfterBudget** en application **iOS native** (Swift / SwiftUI), **100 % hors ligne**, avec **parité fonctionnelle** avec la version desktop Rust (hors module réseau de mise à jour).

L’app doit permettre de connaître à tout moment le **solde prévisionnel de fin de mois** :

`solde actuel + revenus en attente − dépenses en attente`

avec suivi du découvert autorisé et statut financier (En forme / Attention / Danger).

## 2. Décisions validées

| Sujet | Décision |
|---|---|
| Plateforme | iOS **17+**, **iPhone seul** |
| UI | SwiftUI, NavigationStack, TabView |
| Architecture | **MVVM par features** + services + repositories (approche A) |
| Stockage | **SQLite** via **GRDB** |
| Réseau / API | **Aucun** — offline strict |
| Parité desktop | **Complète** dès la v1 (sauf check update réseau) |
| Code | Identifiants / code en **anglais** |
| Docs & commentaires | **Français** |
| Design | **Nouveau**, éditorial / sobre / premium — **pas** Encre & Cuivre, **pas** esthétique « IA générique » |
| Tests | Un fichier de tests par fichier de logique ; tests unitaires par feature |

## 3. Architecture

### 3.1 Structure des dossiers

```
afterbudget-ios/
  AfterBudget/
    App/                      # @main, AppRoot, composition racine
    Core/
      Database/               # GRDB, migrations, DatabaseClient
      DesignSystem/           # tokens + composants UI partagés
      Domain/                 # Money, identifiants, erreurs typées
      Utilities/              # formatters, helpers purs
    Features/
      Onboarding/
      Budget/                 # dashboard / solde prévisionnel
      Transactions/
      Categories/
      Statistics/
      Recurrences/
      Settings/
      ImportExport/
    Resources/                # assets, Localizable.xcstrings (FR)
  AfterBudgetTests/           # miroir 1:1 des fichiers testés
  docs/
    ARCHITECTURE.md
    SPEC.md
    DATABASE.md
    DESIGN_SYSTEM.md
    DECISIONS.md
    superpowers/specs/        # ce document
```

### 3.2 Organisation d’une feature

Chaque feature expose typiquement :

- `Models/` — modèles domaine / lecture
- `DTOs/` — create / update / filter
- `Repositories/` — protocoles + implémentation GRDB
- `Services/` — règles métier
- `ViewModels/` — état UI (`@Observable`), intents
- `Views/` — écrans + composants locaux à la feature

### 3.3 Règles de dépendance

- `Feature` → `Core` uniquement
- **Interdit :** `Feature` → `Feature`
- Partage cross-feature via `Core` (protocoles, modèles communs) ou composition dans `App/`
- Flux : `View` → `ViewModel` → `Service` → `Repository` → SQLite
- **Aucune** requête SQL dans les ViewModels ou les Views

### 3.4 Observation

ViewModels en `@Observable` (Observation framework, iOS 17). Injection des dépendances par initializers (protocoles), composée depuis `App/`.

## 4. Features & écrans

### 4.1 Périmètre fonctionnel

1. **Onboarding** — solde initial, découvert, devise ; fullScreenCover au premier lancement
2. **Budget (Accueil)** — solde actuel, en-attente, solde prévisionnel, statut
3. **Transactions** — CRUD, réalisée / en attente, filtres (type, statut, catégorie, mois, recherche)
4. **Categories** — seed initial + pastilles / usage
5. **Statistics** — agrégats mensuels par catégorie
6. **Recurrences** — règles générant des transactions
7. **Settings** — solde, découvert, devise, thème clair/sombre, réinitialisation
8. **Import / Export** — fichier SQLite ; viser la **compatibilité de schéma** avec le desktop autant que possible

### 4.2 Hors périmètre v1

- Compte utilisateur, sync cloud, API
- Module de mise à jour / téléchargement (desktop)
- iPad / Mac Catalyst (cible iPhone seule)
- Widgets / App Clips / Watch

### 4.3 Navigation iPhone

`TabView` :

| Onglet | Feature |
|---|---|
| Accueil | Budget |
| Transactions | Transactions |
| Stats | Statistics |
| Réglages | Settings |

Formulaires en sheet ou push. Onboarding en `fullScreenCover`. Récurrences et import/export accessibles depuis Réglages (et/ou actions contextuelles Transactions si pertinent).

## 5. Données

### 5.1 Moteur

- SQLite + **GRDB**
- Migrations versionnées (alignées sur l’esprit desktop : settings, categories, transactions, recurrences, métadonnées)
- Montants en **centimes** (`Int64`) — pas de `Double` monétaire
- Base stockée dans le sandbox app (Application Support)
- Tests : base **in-memory** ou fichier temporaire

### 5.2 DTOs

Chaque écriture / filtre passe par un DTO dédié (ex. `CreateTransactionDTO`, `UpdateTransactionDTO`, `FilterTransactionsDTO`, `UpdateSettingsDTO`, `CreateRecurrenceDTO`, `CompleteOnboardingDTO`) avec validation côté service.

### 5.3 Calcul budget (règle métier centrale)

```
projectedEndOfMonthBalance = currentBalance + pendingIncome - pendingExpenses
```

Statut :

- **En forme** — solde prévisionnel confortablement au-dessus du seuil lié au découvert
- **Attention** — zone intermédiaire
- **Danger** — solde prévisionnel sous le seuil / découvert

Les seuils exacts seront calqués sur `docs/SPEC_AFTERBUDGET.md` du desktop lors de l’implémentation (une seule source de vérité documentée dans `docs/SPEC.md` iOS).

## 6. Design system

### 6.1 Intention

Look **finance personnelle éditoriale** : sobre, précis, premium craft — **pas** template IA (pas de gradients héro, glassmorphism, accent teal/violet startup, grilles de cards identiques).

### 6.2 Principes

- Palette **restreinte** : charbon / papier cassé / **un accent cuivré-corail mat** / gris neutres
- SF Pro + chiffres tabulaires pour les montants
- SF Symbols uniquement, poids régulier, pas d’icônes multicolores
- Listes natives SwiftUI ; séparateurs fins ; respiration asymétrique assumée
- Accueil = solde dominant ; Transactions = scan rapide ; Stats = lecture claire
- Dynamic Type, VoiceOver, Reduce Motion, contrastes WCAG
- Thème clair / sombre (override Settings) avec les **mêmes** ratios de contraste
- Tokens centralisés dans `Core/DesignSystem` — **aucun** hex hardcodé dans les features

### 6.3 Composants Core (indicatif)

`ABAmountText`, `ABStatusPill`, `ABCategoryChip`, `ABEmptyState`, `ABMonthHeader`, `ABSectionLabel` — noms définitifs dans `DESIGN_SYSTEM.md`.

## 7. Tests & qualité

### 7.1 Convention

- **1 fichier de logique → 1 fichier de tests** (même nom, suffixe `Tests`)
- Cibles : Services, Repositories, Validators, Mappers, calculs Budget, ViewModels
- Views : pas de snapshot obligatoire en v1
- Chaque feature listée en §4.1 a des tests unitaires sur sa logique

### 7.2 Erreurs

- `AppError` (ou équivalent) typé : validation, persistence, import/export
- Messages utilisateur en **français**
- Pas de `try!` / `force unwrap` en chemins production

### 7.3 Lisibilité

Fichiers courts, responsabilités uniques, protocols mockables, Swift concurrency (`async`/`await`) pour I/O DB hors main thread quand pertinent.

## 8. Documentation livrable

Dans `afterbudget-ios/docs/` (FR) :

| Fichier | Rôle |
|---|---|
| `ARCHITECTURE.md` | Couches, dépendances, conventions |
| `SPEC.md` | Exigences fonctionnelles / calculs / écrans |
| `DATABASE.md` | Schéma, migrations, index |
| `DESIGN_SYSTEM.md` | Tokens, composants, accessibilité |
| `DECISIONS.md` | ADR courts (GRDB, MVVM features, design, etc.) |

## 9. Critères de succès v1

- Parité fonctionnelle desktop (hors update réseau)
- Fonctionne sans réseau
- Architecture features respectée + tests miroir
- Design éditorial sobre, HIG Apple
- Docs FR à jour ; code EN commenté en FR
- Import/export SQLite utilisable ; schéma documenté pour compatibilité desktop

## 10. Prochaine étape

Après **review et validation de ce fichier** par l’utilisateur → skill **writing-plans** → plan d’implémentation détaillé → scaffolding Xcode + implémentation (cloud agent / pool Mac si builds iOS).
