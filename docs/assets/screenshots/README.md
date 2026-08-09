# Captures d'écran du site

Ces images sont affichées sur la landing page (GitHub Pages, `docs/`).
Elles doivent provenir **de la véritable application** lancée avec la base de
démonstration — jamais de maquettes.

## Générer la base de démonstration

```bash
scripts/seed_demo_db.sh              # carnet fictif de huit mois, fin de mois à −250,00 €
scripts/demo.sh                      # lance l'application sur cette base
```

La base contient uniquement de fausses données (personnage fictif, mois de
janvier à août 2026, compte dans le découvert autorisé).

## Réglages conseillés

| Élément | Valeur |
|---|---|
| Fenêtre | 1240 × 780 (taille par défaut, non redimensionnée) |
| Thème | Clair (par défaut de la base de démonstration) |
| Mois affiché | Août 2026 (mois courant) |
| État | Tableau de bord « Attention », solde prévisionnel **−250,00 €** |

## Navigation rapide

| Action | Raccourci |
|---|---|
| Accueil / Transactions / Statistiques / Paramètres | `Ctrl 1` … `Ctrl 4` |
| Nouvelle dépense | `Ctrl N` |
| Fermer le formulaire | `Échap` |

## Fichiers attendus

Enregistrer chaque capture en **PNG**, fenêtre 1240 × 780, sous ces noms
exacts (le site y fait référence) :

| Fichier | Écran | Contenu attendu |
|---|---|---|
| `dashboard.png` | Accueil | Solde prévisionnel −250,00 € en rouge, badge « Attention », jauge de découvert, indicateurs, derniers mouvements, récapitulatif |
| `transactions.png` | Transactions | Liste d'août avec colonnes alignées, statuts Réalisé / En attente, barre de filtres, total affiché |
| `statistics.png` | Statistiques | Comparaison avec juillet, avancement du mois, anneaux de répartition |
| `settings.png` | Paramètres | Budget, apparence, opérations récurrentes, données locales |
| `form.png` | Formulaire | Modale « Nouvelle dépense » ouverte (`Ctrl N` depuis l'accueil) |
| `onboarding.png` | Bienvenue | Écran de premier lancement (base `--onboarding`) |

Pour l'écran de bienvenue :

```bash
scripts/demo.sh --onboarding
```

## Recommandations de propreté

- Éviter les notifications (toasts) : ne pas cliquer sur des actions juste avant la capture.
- Ne pas ouvrir le sélecteur de mois ni la recherche.
- Ne pas capturer pendant une animation de chargement.
- Si la souris affiche un survol gênant, la déplacer hors de la fenêtre avant de capturer.
