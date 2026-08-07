# DECISIONS.md — Décisions techniques

## Bibliothèque SQLite : rusqlite

**Choix** : `rusqlite` avec le feature `bundled` (compilation statique de SQLite).

**Raison** : rusqlite est la bibliothèque Rust la plus mature pour SQLite. Le mode `bundled` évite les dépendances système et garantit la portabilité.

**Alternative** : `sqlx` — plus orienté async mais plus lourd et non nécessaire pour une app desktop locale.

## Bibliothèque de boîtes de dialogue : rfd

**Choix** : `rfd` (Rust File Dialog).

**Raison** : rfd fournit des boîtes de dialogue natives multiplateformes sans dépendre de GTK ou autre toolkit lourd. Simple et direct.

## Gestion des icônes

**Choix** : Émojis Unicode comme placeholders pour les icônes de catégorie.

**Raison** : Les icônes React/Lucide ne sont pas disponibles en Rust. Pour le MVP, les émojis offrent un rendu acceptable sur tous les OS. Les noms d'icônes (Home, Car, etc.) sont conservés dans la base pour une future migration vers des icônes vectorielles SVG ou une police d'icônes.

## Organisation de l'état Iced

**Choix** : Architecture Elm-like avec `State` → `Message` → `update` → `view`.

**Raison** : Modèle recommandé par Iced. L'état est centralisé dans `AppState`, les mutations sont déclenchées par des messages. Les opérations SQLite sont synchrones (pas de thread séparé pour le MVP).

## Gestion des opérations asynchrones

**Choix** : Opérations SQLite synchrones dans le thread principal pour le MVP.

**Raison** : Pour une utilisation locale avec un volume de données modeste, les requêtes SQLite sont quasi instantanées. Une version future pourra déporter les opérations lourdes (import de base) vers une tâche de fond.

## Mécanisme de sauvegarde SQLite

**Choix** : API `rusqlite::backup::Backup` pour l'export.

**Raison** : L'API de backup SQLite produit une copie cohérente même en cas d'écritures concurrentes, contrairement à une simple copie de fichier. Pour l'import, on effectue une copie de fichier après fermeture de la connexion.

## Organisation du thème

**Choix** : Constantes dans `theme/colors.rs`, `theme/spacing.rs`, `theme/typography.rs`.

**Raison** : Centraliser les valeurs de design évite la dispersion et facilite les modifications futures. Les thèmes clair et sombre sont gérés via deux ensembles de constantes (AppColors et DarkColors).
