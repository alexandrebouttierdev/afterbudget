# AFTERBUDGET — Spécifications fonctionnelles et techniques

> **Document :** `SPEC.md`  
> **Version :** 1.0  
> **Statut :** Brouillon initial exploitable  
> **Plateforme cible :** Application desktop  
> **Technologies principales :** Rust, Iced, SQLite  
> **Référence fonctionnelle :** https://afterbudget.vercel.app/

---

## 1. Présentation du projet

### 1.1 Nom

**AfterBudget**

### 1.2 Description

AfterBudget est une application desktop de gestion de budget personnel.

Elle permet à l’utilisateur de connaître à tout moment son **solde prévisionnel de fin de mois**, à partir :

- de son solde bancaire actuel ;
- des revenus qui doivent encore être reçus ;
- des dépenses qui doivent encore être payées ;
- de son découvert bancaire autorisé.

L’application doit répondre simplement à la question :

> **Combien me restera-t-il réellement à la fin du mois ?**

### 1.3 Proposition de valeur

AfterBudget ne cherche pas à remplacer une application bancaire ou un logiciel de comptabilité complet.

Son objectif est de fournir une vision immédiate et compréhensible de la situation financière du mois en cours :

- ce qui est déjà payé ou reçu ;
- ce qui reste à payer ;
- ce qui reste à recevoir ;
- le solde estimé en fin de mois ;
- la marge restante avant le dépassement du découvert autorisé.

### 1.4 Principes du produit

L’application doit rester :

- simple ;
- rapide ;
- locale ;
- sans compte utilisateur ;
- sans serveur distant ;
- sans publicité ;
- sans système bancaire connecté ;
- respectueuse de la vie privée.

---

## 2. Objectifs

### 2.1 Objectifs principaux

- Afficher le solde prévisionnel de fin de mois.
- Permettre l’ajout rapide d’un revenu ou d’une dépense.
- Distinguer les transactions déjà réalisées des transactions encore attendues.
- Afficher la marge disponible avant le découvert autorisé.
- Alerter l’utilisateur lorsque le solde prévu devient dangereux.
- Présenter des statistiques simples et compréhensibles.
- Conserver toutes les données localement dans une base SQLite.

### 2.2 Objectifs secondaires

- Permettre une utilisation complète au clavier.
- Fonctionner sans connexion Internet.
- Démarrer rapidement.
- Conserver une interface claire, même avec un grand nombre de transactions.
- Préparer l’architecture pour de futures fonctions comme les transactions récurrentes, les sauvegardes et l’export.

### 2.3 Indicateurs de réussite

Le produit est considéré comme fonctionnel lorsque l’utilisateur peut :

1. configurer son solde actuel et son découvert autorisé ;
2. ajouter ses revenus et dépenses du mois ;
3. marquer chaque opération comme payée, reçue ou en attente ;
4. consulter immédiatement son solde prévu de fin de mois ;
5. connaître sa marge avant le dépassement du découvert ;
6. retrouver ses données après la fermeture et le redémarrage de l’application.

---

## 3. Périmètre de la première version

### 3.1 Inclus dans le MVP

- Premier démarrage et configuration initiale.
- Gestion du solde bancaire actuel.
- Gestion du découvert autorisé.
- Création, modification et suppression des transactions.
- Types de transaction : revenu et dépense.
- Statuts : réalisé et en attente.
- Classement des transactions par catégorie.
- Sélection d’une date.
- Tableau de bord du mois.
- Calcul du solde prévisionnel.
- Indicateur de risque financier.
- Liste et filtrage des transactions.
- Statistiques mensuelles simples.
- Paramètres de l’application.
- Stockage local SQLite.
- Export manuel d’une copie complète de la base SQLite.
- Import et restauration d’une base SQLite précédemment exportée.
- Fonctionnement hors ligne.
- Thème clair et sombre.
- Compatibilité Windows, macOS et Linux.
- Interface moderne conçue comme une véritable application desktop.

### 3.2 Hors périmètre du MVP

- Création de compte.
- Synchronisation cloud.
- Synchronisation entre plusieurs appareils.
- Connexion automatique aux comptes bancaires.
- Import bancaire automatique.
- Paiements depuis l’application.
- Budgets partagés.
- Gestion de plusieurs utilisateurs.
- Intelligence artificielle.
- Publicité.
- Télémétrie ou suivi publicitaire.
- Application mobile.
- Conversion automatique de devises.
- Gestion comptable professionnelle.

### 3.3 Évolutions possibles

- Transactions récurrentes.
- Duplication rapide d’une transaction.
- Import et export CSV.
- Export PDF.
- Sauvegarde chiffrée de la base.
- Gestion de plusieurs comptes bancaires.
- Budgets mensuels par catégorie.
- Objectifs d’épargne.
- Notifications locales.
- Calendrier des échéances.
- Comparaison entre plusieurs mois.
- Portabilité vers une version mobile.

---

## 4. Utilisateurs cibles

### 4.1 Utilisateur principal

Particulier souhaitant suivre simplement son budget mensuel sans connecter son compte bancaire à un service externe.

### 4.2 Besoins principaux

L’utilisateur souhaite :

- savoir s’il pourra payer toutes ses dépenses du mois ;
- éviter un dépassement de découvert ;
- identifier les dépenses encore à payer ;
- identifier les revenus encore à recevoir ;
- comprendre rapidement où part son argent ;
- conserver ses données uniquement sur son ordinateur.

### 4.3 Niveau technique attendu

Aucune compétence technique ou comptable ne doit être nécessaire.

---

## 5. Définitions métier

### 5.1 Solde actuel

Montant actuellement disponible sur le compte bancaire de l’utilisateur.

Le solde actuel est renseigné manuellement. Il doit correspondre au solde réel affiché par la banque au moment de la mise à jour.

### 5.2 Revenu

Entrée d’argent prévue ou déjà reçue.

Exemples :

- salaire ;
- allocation ;
- remboursement ;
- prime ;
- vente ;
- virement reçu.

### 5.3 Dépense

Sortie d’argent prévue ou déjà payée.

Exemples :

- loyer ;
- courses ;
- énergie ;
- abonnement ;
- assurance ;
- transport.

### 5.4 Transaction en attente

Transaction qui n’est pas encore reflétée dans le solde bancaire actuel.

- Pour un revenu : argent non encore reçu.
- Pour une dépense : argent non encore débité ou payé.

### 5.5 Transaction réalisée

Transaction déjà reflétée dans le solde bancaire actuel.

- Pour un revenu : revenu reçu.
- Pour une dépense : dépense payée ou débitée.

### 5.6 Découvert autorisé

Montant maximal que la banque autorise sous zéro.

Le découvert est enregistré comme une valeur positive.

Exemple :

- découvert autorisé saisi : `500 €` ;
- limite bancaire réelle : `-500 €`.

### 5.7 Mois actif

Mois actuellement sélectionné dans l’application.

Par défaut, l’application ouvre le mois civil correspondant à la date du système.

---

## 6. Règles de calcul

### 6.1 Revenus restant à recevoir

```text
revenus_en_attente =
    somme des revenus du mois actif dont le statut est EN_ATTENTE
```

### 6.2 Dépenses restant à payer

```text
depenses_en_attente =
    somme des dépenses du mois actif dont le statut est EN_ATTENTE
```

### 6.3 Solde prévisionnel de fin de mois

```text
solde_previsionnel =
    solde_actuel
    + revenus_en_attente
    - depenses_en_attente
```

### 6.4 Marge avant dépassement du découvert

```text
marge_avant_depassement =
    solde_previsionnel
    + decouvert_autorise
```

Exemple :

```text
Solde prévisionnel : -478 €
Découvert autorisé : 500 €
Marge restante : 22 €
```

### 6.5 Statut financier

#### Statut « En forme »

```text
solde_previsionnel >= 0
```

Message proposé :

> Tu devrais terminer le mois avec un solde positif.

#### Statut « Attention »

```text
solde_previsionnel < 0
ET
solde_previsionnel >= -decouvert_autorise
```

Message proposé :

> Tu devrais utiliser une partie de ton découvert. Il te reste X € de marge.

#### Statut « Danger »

```text
solde_previsionnel < -decouvert_autorise
```

Message proposé :

> Tu risques de dépasser ton découvert autorisé de X €.

### 6.6 Dépassement prévu

```text
depassement_prevu =
    valeur_absolue(min(marge_avant_depassement, 0))
```

### 6.7 Règles importantes

- Seules les transactions du mois actif participent aux calculs du mois.
- Les transactions réalisées ne doivent pas être ajoutées ou soustraites une seconde fois, car elles sont supposées être déjà reflétées dans le solde actuel.
- Modifier le statut d’une transaction doit recalculer immédiatement tous les indicateurs.
- Modifier le solde actuel doit recalculer immédiatement tous les indicateurs.
- Modifier le découvert autorisé doit recalculer immédiatement le statut financier.
- Les montants monétaires ne doivent jamais être stockés avec un type flottant.

### 6.8 Stockage des montants

Tous les montants sont enregistrés en centimes avec un entier signé 64 bits.

Exemples :

```text
10,00 €  -> 1000
-478,00 € -> -47800
```

Dans la base de données, le montant d’une transaction reste positif. Son effet dépend de son type :

- revenu : addition ;
- dépense : soustraction.

---

## 7. Parcours utilisateur principal

### 7.1 Premier lancement

1. L’utilisateur ouvre AfterBudget.
2. L’application détecte qu’aucune configuration n’existe.
3. Un écran de bienvenue explique brièvement le fonctionnement.
4. L’utilisateur saisit :
   - son solde bancaire actuel ;
   - son découvert autorisé ;
   - sa devise ;
   - éventuellement son nom d’affichage.
5. L’utilisateur valide.
6. L’application crée les paramètres initiaux et les catégories par défaut.
7. L’utilisateur arrive sur le tableau de bord.

### 7.2 Ajout d’une dépense

1. L’utilisateur clique sur `Ajouter une dépense`.
2. Il saisit un libellé.
3. Il saisit un montant.
4. Il choisit une date.
5. Il choisit une catégorie.
6. Il indique si la dépense est déjà payée.
7. Il valide.
8. La transaction est enregistrée.
9. Le tableau de bord est recalculé immédiatement.

### 7.3 Ajout d’un revenu

1. L’utilisateur clique sur `Ajouter un revenu`.
2. Il saisit un libellé.
3. Il saisit un montant.
4. Il choisit une date.
5. Il choisit une catégorie.
6. Il indique si le revenu est déjà reçu.
7. Il valide.
8. La transaction est enregistrée.
9. Le tableau de bord est recalculé immédiatement.

### 7.4 Mise à jour d’une transaction

1. L’utilisateur ouvre une transaction.
2. Il peut modifier ses informations.
3. Il peut la marquer comme réalisée ou en attente.
4. Il enregistre les modifications.
5. Tous les calculs sont mis à jour.

### 7.5 Changement de mois

1. L’utilisateur sélectionne le mois précédent ou suivant.
2. L’application charge les transactions du mois choisi.
3. Les statistiques et indicateurs sont recalculés.
4. Le mois courant reste identifiable visuellement.

---

## 8. Navigation

### 8.1 Navigation principale

L’application comporte les sections suivantes :

- **Accueil**
- **Transactions**
- **Statistiques**
- **Paramètres**

### 8.2 Comportement attendu

- La navigation doit rester visible sur les écrans principaux.
- L’écran actif doit être clairement identifié.
- Les actions d’ajout doivent être accessibles depuis l’accueil et la liste des transactions.
- Le retour arrière doit conserver les filtres et le mois sélectionné durant la session.
- Les raccourcis clavier principaux doivent être documentés dans l’application.

### 8.3 Proposition de raccourcis

| Action | Raccourci proposé |
|---|---|
| Ajouter une dépense | `Ctrl/Cmd + D` |
| Ajouter un revenu | `Ctrl/Cmd + R` |
| Rechercher | `Ctrl/Cmd + F` |
| Enregistrer | `Ctrl/Cmd + S` |
| Fermer une fenêtre modale | `Échap` |
| Mois précédent | `Ctrl/Cmd + ←` |
| Mois suivant | `Ctrl/Cmd + →` |

---

## 9. Écrans

## 9.1 Écran de bienvenue

### Objectif

Expliquer le fonctionnement de l’application et collecter les informations minimales nécessaires.

### Contenu

- Logo et nom AfterBudget.
- Description courte.
- Champ `Solde bancaire actuel`.
- Champ `Découvert autorisé`.
- Sélecteur de devise.
- Bouton `Commencer`.
- Mention indiquant que les données restent sur l’ordinateur.

### Validations

- Le solde actuel peut être positif, nul ou négatif.
- Le découvert autorisé doit être supérieur ou égal à zéro.
- La devise est obligatoire.
- Une erreur compréhensible doit être affichée sous le champ concerné.

---

## 9.2 Écran d’accueil

### Objectif

Afficher immédiatement la situation financière du mois actif.

### Contenu principal

#### En-tête

- Mois sélectionné.
- Boutons mois précédent et mois suivant.
- Bouton retour au mois courant.
- Actions d’ajout rapide.

#### Carte « Fin de mois prévu »

- Solde prévisionnel.
- Formule résumée :

```text
Solde actuel + revenus à recevoir - dépenses à payer
```

- Statut financier.
- Message d’alerte éventuel.

#### Carte « Rentre encore »

- Total des revenus en attente.
- Nombre de revenus concernés.
- Accès à la liste filtrée.

#### Carte « Reste à payer »

- Total des dépenses en attente.
- Nombre de dépenses concernées.
- Accès à la liste filtrée.

#### Résumé mensuel

- Total des revenus du mois.
- Total des dépenses du mois.
- Total déjà reçu.
- Total déjà payé.
- Solde net théorique du mois.

#### Transactions récentes

- Dernières transactions du mois.
- Type.
- Libellé.
- Catégorie.
- Date.
- Montant.
- Statut.
- Bouton pour afficher toutes les transactions.

### États particuliers

#### Aucune transaction

Afficher :

> Aucune transaction pour ce mois.

Actions :

- `Ajouter un revenu`
- `Ajouter une dépense`

#### Données incomplètes

Si le solde actuel n’a jamais été renseigné ou paraît ancien, afficher un rappel non bloquant :

> Vérifie que ton solde actuel correspond bien à celui de ta banque.

---

## 9.3 Écran des transactions

### Objectif

Consulter, rechercher, filtrer et gérer les revenus et dépenses.

### Affichage

Chaque transaction affiche :

- libellé ;
- montant ;
- type ;
- catégorie ;
- date ;
- statut ;
- note éventuelle ;
- actions de modification et suppression.

### Tri par défaut

1. date décroissante ;
2. date de création décroissante en cas d’égalité.

### Filtres

- mois ;
- type ;
- statut ;
- catégorie ;
- recherche textuelle ;
- plage de montant facultative.

### Valeurs de filtre

#### Type

- Tous
- Revenus
- Dépenses

#### Statut

- Tous
- Réalisés
- En attente

### Actions

- Ajouter une transaction.
- Modifier une transaction.
- Supprimer une transaction.
- Marquer comme réalisée.
- Marquer comme en attente.
- Réinitialiser les filtres.

### Suppression

La suppression doit demander confirmation :

> Supprimer définitivement cette transaction ?

La confirmation doit afficher au minimum le libellé et le montant.

---

## 9.4 Fenêtre d’ajout ou de modification d’une transaction

### Champs

| Champ | Type | Obligatoire | Règles |
|---|---|---:|---|
| Type | Revenu ou dépense | Oui | Non modifiable après validation si cette contrainte est retenue |
| Libellé | Texte | Oui | 1 à 120 caractères |
| Montant | Monétaire | Oui | Strictement supérieur à zéro |
| Date | Date | Oui | Date valide |
| Catégorie | Sélection | Oui | Catégorie active correspondant au type |
| Statut | Réalisé ou en attente | Oui | En attente par défaut |
| Note | Texte multiligne | Non | 1 000 caractères maximum |

### Boutons

- `Annuler`
- `Enregistrer`

En mode modification :

- `Supprimer`
- `Enregistrer les modifications`

### Ergonomie

- Le champ montant doit accepter la virgule et le point comme séparateur décimal.
- Le montant doit être formaté dans la devise active.
- La validation avec le clavier doit être possible.
- La touche `Échap` ferme la fenêtre après confirmation si des modifications non enregistrées existent.

---

## 9.5 Écran des statistiques

### Objectif

Aider l’utilisateur à comprendre les revenus et dépenses du mois.

### Indicateurs

- Total des revenus.
- Total des dépenses.
- Différence revenus moins dépenses.
- Total des dépenses réalisées.
- Total des dépenses en attente.
- Total des revenus reçus.
- Total des revenus en attente.
- Nombre total de transactions.

### Graphiques proposés

- Répartition des dépenses par catégorie.
- Répartition des revenus par catégorie.
- Comparaison revenus et dépenses.
- Évolution cumulée des transactions au cours du mois.

### Contraintes

- Les graphiques doivent rester lisibles sans dépendre uniquement des couleurs.
- Une légende textuelle doit être disponible.
- En l’absence de données, afficher un état vide plutôt qu’un graphique vide.
- Les valeurs des graphiques doivent correspondre exactement aux transactions affichées pour la période.

### MVP minimal

Pour limiter la complexité de la première version, les statistiques peuvent commencer par :

- des cartes récapitulatives ;
- une liste des dépenses par catégorie avec montant et pourcentage ;
- une barre de progression par catégorie.

Les graphiques avancés peuvent être ajoutés dans une version ultérieure.

---

## 9.6 Écran des paramètres

### Sections

#### Budget

- Solde bancaire actuel.
- Date de dernière mise à jour du solde.
- Découvert autorisé.
- Devise.
- Premier jour de la semaine.
- Format de date.

#### Apparence

- Thème système.
- Thème clair.
- Thème sombre.
- Taille d’interface, si retenue.

#### Catégories

- Afficher les catégories.
- Ajouter une catégorie.
- Renommer une catégorie.
- Désactiver une catégorie.
- Réordonner les catégories.
- Choisir une icône facultative.

#### Données

- Afficher l’emplacement de la base locale.
- Exporter une copie complète de la base SQLite.
- Importer et restaurer une base SQLite précédemment exportée.
- Afficher la date de la dernière exportation réussie.
- Réinitialiser toutes les données.

#### À propos

- Version de l’application.
- Description de la confidentialité.
- Licence.
- Liens vers les conditions d’utilisation et la politique de confidentialité.

### Réinitialisation des données

La réinitialisation doit nécessiter une confirmation forte.

Exemple :

> Cette action supprimera définitivement tous les revenus, dépenses, catégories personnalisées et paramètres.

Une seconde action explicite est recommandée, par exemple :

```text
SUPPRIMER
```

---

## 10. Catégories

### 10.1 Catégories par défaut

AfterBudget doit créer les catégories suivantes lors du premier lancement.

Les identifiants sont stables et ne doivent pas être modifiés après la création de la base. Le nom affiché pourra être traduit dans de futures versions, mais l’identifiant interne restera identique.

| Identifiant | Nom | Icône | Couleur |
|---|---|---|---|
| `logement` | Logement | `Home` | `#3B82F6` |
| `alimentation` | Alimentation | `ShoppingCart` | `#22C55E` |
| `transport` | Transport | `Car` | `#EAB308` |
| `sante` | Santé | `Heart` | `#EF4444` |
| `loisirs` | Loisirs | `Gamepad2` | `#A855F7` |
| `vetements` | Vêtements | `Shirt` | `#EC4899` |
| `abonnements` | Abonnements | `RefreshCw` | `#6366F1` |
| `restaurants` | Restaurants | `Utensils` | `#F97316` |
| `voyages` | Voyages | `Plane` | `#06B6D4` |
| `education` | Éducation | `GraduationCap` | `#14B8A6` |
| `epargne` | Épargne | `PiggyBank` | `#10B981` |
| `energie` | Énergie | `Zap` | `#F59E0B` |
| `internet` | Internet | `Wifi` | `#0EA5E9` |
| `telephone` | Téléphone | `Smartphone` | `#8B5CF6` |
| `sport` | Sport | `Dumbbell` | `#84CC16` |
| `musique` | Musique | `Music` | `#F43F5E` |
| `livres` | Livres | `BookOpen` | `#78716C` |
| `enfants` | Enfants | `Baby` | `#D946EF` |
| `animaux` | Animaux | `PawPrint` | `#CA8A04` |
| `bricolage` | Bricolage | `Wrench` | `#6B7280` |
| `cadeaux` | Cadeaux | `Gift` | `#F472B6` |
| `bien-etre` | Bien-être | `Sparkles` | `#C084FC` |
| `credit` | Crédit | `CreditCard` | `#FB923C` |
| `tabac` | Tabac | `Cigarette` | `#A8A29E` |
| `assurance` | Assurance | `Shield` | `#38BDF8` |
| `autre` | Autre | `MoreHorizontal` | `#9CA3AF` |

### 10.2 Utilisation des icônes

Les noms d’icônes ci-dessus proviennent de la référence fonctionnelle existante.

Dans l’application Rust/Iced :

- les composants React ou Lucide React ne doivent pas être utilisés directement ;
- chaque nom d’icône doit être associé à une ressource compatible avec Iced ;
- les icônes peuvent être fournies sous forme de SVG embarqués, de police d’icônes ou d’un autre format vectoriel compatible ;
- le rendu doit rester cohérent sur Windows, macOS et Linux ;
- une icône manquante doit utiliser `MoreHorizontal` comme valeur de repli ;
- l’interface ne doit pas dépendre uniquement de l’icône : le nom de la catégorie doit rester visible ou accessible.

### 10.3 Utilisation des couleurs

- Les couleurs sont enregistrées au format hexadécimal `#RRGGBB`.
- La couleur sert aux badges, graphiques, légendes et indicateurs visuels de catégorie.
- Le texte affiché sur une couleur doit respecter un contraste suffisant.
- La couleur ne doit jamais être le seul moyen d’identifier une catégorie.
- Le thème sombre peut adapter légèrement le rendu visuel sans modifier la couleur enregistrée dans la base.
- Les couleurs exportées dans la base SQLite doivent être conservées lors d’un import.

### 10.4 Catégories de revenus

Les catégories ci-dessus sont principalement destinées aux dépenses.

Pour les revenus, AfterBudget doit proposer les catégories initiales suivantes :

- Salaire
- Allocation
- Prime
- Remboursement
- Vente
- Pension
- Revenu indépendant
- Intérêts
- Autre

Les catégories de revenus doivent également posséder :

- un identifiant stable ;
- un nom ;
- une icône ;
- une couleur ;
- un ordre d’affichage.

La palette et les icônes exactes des catégories de revenus pourront être définies durant la phase de design, tout en respectant le design system du projet.

### 10.5 Règles de gestion

- Une catégorie appartient à un type : revenu ou dépense.
- Une catégorie par défaut ne doit pas être supprimée définitivement.
- Une catégorie par défaut ou personnalisée peut être désactivée.
- Une catégorie déjà utilisée ne doit pas être supprimée brutalement.
- Une transaction historique conserve sa catégorie.
- Une catégorie désactivée n’est plus proposée pour les nouvelles transactions.
- La catégorie `Autre` ne peut pas être désactivée ni supprimée.
- L’identifiant d’une catégorie par défaut ne peut pas être modifié.
- Le nom, l’icône et la couleur d’une catégorie personnalisée peuvent être modifiés.
- Les catégories sont triées selon leur champ `sort_order`.
- L’import d’une base doit conserver les identifiants, icônes, couleurs et ordres d’affichage.
## 11. Exigences fonctionnelles

## 11.1 Configuration

### FR-001 — Premier lancement

L’application doit afficher la configuration initiale lorsqu’aucun paramètre n’est enregistré.

### FR-002 — Solde actuel

L’utilisateur doit pouvoir enregistrer et modifier son solde bancaire actuel.

### FR-003 — Découvert autorisé

L’utilisateur doit pouvoir enregistrer un découvert autorisé supérieur ou égal à zéro.

### FR-004 — Devise

L’utilisateur doit pouvoir choisir une devise. L’euro est la devise par défaut.

### FR-005 — Conservation des paramètres

Les paramètres doivent être restaurés après le redémarrage de l’application.

---

## 11.2 Transactions

### FR-010 — Ajouter une transaction

L’utilisateur doit pouvoir créer un revenu ou une dépense.

### FR-011 — Modifier une transaction

L’utilisateur doit pouvoir modifier toutes les propriétés d’une transaction.

### FR-012 — Supprimer une transaction

L’utilisateur doit pouvoir supprimer une transaction après confirmation.

### FR-013 — Statut

Une transaction doit pouvoir être marquée comme réalisée ou en attente.

### FR-014 — Catégorie

Chaque transaction doit être associée à une catégorie.

### FR-015 — Date

Chaque transaction doit posséder une date valide.

### FR-016 — Montant

Le montant d’une transaction doit être strictement supérieur à zéro.

### FR-017 — Recherche

L’utilisateur doit pouvoir rechercher une transaction par son libellé ou sa note.

### FR-018 — Filtres

L’utilisateur doit pouvoir filtrer les transactions par type, statut, catégorie et mois.

### FR-019 — Mise à jour immédiate

Toute modification d’une transaction doit mettre à jour les calculs sans redémarrage ni rechargement manuel.

---

## 11.3 Tableau de bord

### FR-020 — Mois courant

L’application doit afficher le mois courant au démarrage.

### FR-021 — Navigation mensuelle

L’utilisateur doit pouvoir naviguer entre les mois.

### FR-022 — Solde prévisionnel

L’application doit calculer le solde prévisionnel du mois actif.

### FR-023 — Revenus en attente

L’application doit afficher le total des revenus non encore reçus.

### FR-024 — Dépenses en attente

L’application doit afficher le total des dépenses non encore payées.

### FR-025 — Marge de découvert

L’application doit calculer la marge restante avant le dépassement du découvert autorisé.

### FR-026 — Statut financier

L’application doit afficher un statut `En forme`, `Attention` ou `Danger`.

### FR-027 — Explication du calcul

L’application doit permettre à l’utilisateur de comprendre les montants composant le solde prévisionnel.

---

## 11.4 Statistiques

### FR-030 — Résumé mensuel

L’application doit afficher les totaux de revenus et de dépenses du mois actif.

### FR-031 — Répartition par catégorie

L’application doit afficher la répartition des dépenses par catégorie.

### FR-032 — États vides

L’écran doit afficher un message explicite lorsqu’aucune donnée statistique n’est disponible.

### FR-033 — Cohérence

Les statistiques doivent utiliser les mêmes données et filtres que la liste des transactions.

---

## 11.5 Données locales

### FR-040 — Base locale

Toutes les données doivent être stockées dans une base SQLite locale.

### FR-041 — Absence de compte

L’application ne doit pas nécessiter de compte utilisateur.

### FR-042 — Absence de réseau

Les fonctions du MVP doivent rester utilisables hors ligne.

### FR-043 — Migrations

L’application doit appliquer les migrations de base de données nécessaires au démarrage.

### FR-044 — Intégrité

Une erreur d’enregistrement ne doit pas entraîner la perte silencieuse de données.

### FR-045 — Export SQLite

L’utilisateur doit pouvoir exporter une copie complète de sa base de données SQLite vers l’emplacement de son choix.

L’export doit inclure :

- les transactions ;
- les catégories ;
- les paramètres ;
- les informations nécessaires à la restauration ;
- la version du schéma de données.

### FR-046 — Import SQLite

L’utilisateur doit pouvoir importer une base de données SQLite précédemment exportée par AfterBudget.

Avant l’import, l’application doit :

- vérifier que le fichier est une base SQLite valide ;
- vérifier que le schéma appartient bien à AfterBudget ;
- vérifier que la version est compatible ou migrable ;
- créer automatiquement une sauvegarde de la base actuellement utilisée ;
- demander une confirmation explicite avant le remplacement.

### FR-047 — Restauration atomique

L’import doit être atomique.

En cas d’échec :

- la base actuellement utilisée doit rester intacte ;
- la sauvegarde automatique doit rester disponible ;
- aucun fichier incomplet ne doit remplacer la base courante ;
- un message d’erreur compréhensible doit être affiché.

### FR-048 — Transfert multiplateforme

Une base exportée depuis Windows, macOS ou Linux doit pouvoir être importée sur les autres systèmes pris en charge.

Les dates, montants, catégories, transactions et paramètres doivent rester identiques après le transfert.


---

## 12. Exigences non fonctionnelles

### NFR-001 — Performances

- Le lancement de l’application doit être rapide.
- L’affichage du tableau de bord doit être fluide.
- Les recalculs simples doivent sembler instantanés.
- Une interaction locale courante doit viser un temps de réponse inférieur à 100 ms, hors accès disque exceptionnel.

### NFR-002 — Fiabilité

- Les écritures importantes doivent utiliser des transactions SQLite.
- Les erreurs doivent être journalisées localement.
- L’application ne doit pas fermer brutalement à cause d’une donnée invalide.
- Une migration échouée doit empêcher une utilisation susceptible de corrompre les données.

### NFR-003 — Confidentialité

- Aucun revenu, dépense ou solde ne doit être transmis sur Internet.
- Aucun compte n’est requis.
- Aucun tracker publicitaire n’est intégré.
- Aucune télémétrie n’est activée par défaut.
- Les journaux ne doivent pas contenir les montants ou notes des transactions, sauf mode de diagnostic explicitement activé.

### NFR-004 — Accessibilité

- Navigation complète au clavier.
- Focus visible.
- Contrastes suffisants.
- Libellés explicites.
- Les statuts ne doivent pas être identifiés uniquement par une couleur.
- Les montants négatifs doivent comporter un signe explicite.

### NFR-005 — Compatibilité multiplateforme

L’application doit être compatible avec :

- Windows 10 et versions ultérieures ;
- macOS sur processeurs Intel et Apple Silicon, selon les cibles de compilation retenues ;
- Linux desktop sur les principales distributions modernes.

L’interface, les raccourcis, les chemins de fichiers, les boîtes de dialogue natives et le packaging doivent être testés séparément sur chaque système.

Les sauvegardes SQLite doivent être portables entre les trois systèmes.

### NFR-006 — Fenêtre et comportement desktop

- Taille minimale recommandée : `900 × 600`.
- L’application doit être conçue comme un logiciel desktop, pas comme une webapp.
- La navigation doit exploiter l’espace disponible sur les écrans d’ordinateur.
- Les interactions doivent proposer les comportements attendus sur desktop : survol, clavier, focus, fenêtres modales et boîtes de dialogue natives.
- L’interface doit rester utilisable sur une petite fenêtre.
- Les cartes doivent pouvoir se réorganiser lorsque la largeur diminue.
- La taille et la position de la fenêtre doivent être restaurées au redémarrage.
- Les barres de défilement, champs, menus et sélecteurs doivent conserver une apparence cohérente avec le design de l’application.

### NFR-007 — Localisation

La première version est en français.

L’architecture doit éviter les textes codés en dur afin de permettre une future traduction.

### NFR-008 — Précision financière

- Aucun calcul monétaire ne doit utiliser `f32` ou `f64`.
- Tous les montants doivent être stockés en centimes.
- L’affichage doit respecter la devise et la locale sélectionnées.

---

## 13. Architecture technique

### 13.1 Stack

- **Langage :** Rust
- **Interface desktop :** Iced
- **Base de données :** SQLite
- **Accès aux données recommandé :** `rusqlite` ou couche équivalente
- **Sérialisation :** `serde`
- **Dates :** `chrono` ou crate équivalente
- **Identifiants :** UUID ou entier SQLite
- **Journalisation :** `tracing`
- **Gestion des erreurs :** erreurs typées avec contexte

Les bibliothèques exactes doivent être validées au démarrage de l’implémentation.

### 13.2 Organisation proposée

```text
src/
├── main.rs
├── app/
│   ├── mod.rs
│   ├── message.rs
│   ├── state.rs
│   ├── update.rs
│   └── view.rs
├── domain/
│   ├── mod.rs
│   ├── transaction.rs
│   ├── category.rs
│   ├── budget.rs
│   └── money.rs
├── database/
│   ├── mod.rs
│   ├── connection.rs
│   ├── migrations.rs
│   └── repositories/
│       ├── transaction_repository.rs
│       ├── category_repository.rs
│       └── settings_repository.rs
├── services/
│   ├── mod.rs
│   ├── budget_calculator.rs
│   └── statistics_service.rs
├── screens/
│   ├── onboarding.rs
│   ├── dashboard.rs
│   ├── transactions.rs
│   ├── statistics.rs
│   └── settings.rs
├── components/
│   ├── amount.rs
│   ├── status_badge.rs
│   ├── transaction_row.rs
│   ├── modal.rs
│   └── empty_state.rs
├── localization/
│   └── fr.rs
└── utils/
    ├── formatting.rs
    └── validation.rs
```

### 13.3 Séparation des responsabilités

#### Interface

Responsable de :

- l’affichage ;
- la collecte des actions ;
- l’état visuel temporaire ;
- la navigation ;
- les messages d’erreur destinés à l’utilisateur.

#### Domaine

Responsable de :

- la représentation des transactions ;
- la représentation monétaire ;
- les règles de calcul ;
- les validations métier ;
- les statuts financiers.

#### Services

Responsables de :

- calculer les indicateurs ;
- agréger les statistiques ;
- préparer les données destinées à l’interface.

#### Base de données

Responsable de :

- créer et migrer le schéma ;
- lire et enregistrer les données ;
- garantir l’intégrité des écritures ;
- convertir les lignes SQL vers les modèles du domaine.

### 13.4 Gestion des opérations SQLite

Les opérations SQLite ne doivent pas bloquer durablement le thread d’interface.

Approche recommandée :

- connexion gérée par une couche dédiée ;
- opérations exécutées via des commandes ou tâches Iced ;
- retour du résultat sous forme de message applicatif ;
- état de chargement visible pour les opérations longues ;
- transactions SQL pour les écritures multiples.

---

## 14. Modèle de données

## 14.1 Table `transactions`

| Colonne | Type SQLite | Contraintes | Description |
|---|---|---|---|
| `id` | TEXT | PRIMARY KEY | UUID |
| `kind` | TEXT | NOT NULL | `income` ou `expense` |
| `label` | TEXT | NOT NULL | Libellé |
| `amount_cents` | INTEGER | NOT NULL, CHECK > 0 | Montant positif en centimes |
| `transaction_date` | TEXT | NOT NULL | Date ISO `YYYY-MM-DD` |
| `status` | TEXT | NOT NULL | `pending` ou `completed` |
| `category_id` | TEXT | NOT NULL | Référence vers catégorie |
| `note` | TEXT | NULL | Note facultative |
| `created_at` | TEXT | NOT NULL | Date et heure ISO UTC |
| `updated_at` | TEXT | NOT NULL | Date et heure ISO UTC |

### Index proposés

```sql
CREATE INDEX idx_transactions_date
ON transactions(transaction_date);

CREATE INDEX idx_transactions_kind_status
ON transactions(kind, status);

CREATE INDEX idx_transactions_category
ON transactions(category_id);
```

---

## 14.2 Table `categories`

| Colonne | Type SQLite | Contraintes | Description |
|---|---|---|---|
| `id` | TEXT | PRIMARY KEY | Identifiant stable, par exemple `logement` |
| `kind` | TEXT | NOT NULL | `income` ou `expense` |
| `name` | TEXT | NOT NULL | Nom affiché |
| `icon` | TEXT | NOT NULL | Identifiant d’icône, par exemple `Home` |
| `color` | TEXT | NOT NULL | Couleur hexadécimale `#RRGGBB` |
| `sort_order` | INTEGER | NOT NULL | Ordre d’affichage |
| `is_default` | INTEGER | NOT NULL | Booléen SQLite |
| `is_active` | INTEGER | NOT NULL | Booléen SQLite |
| `created_at` | TEXT | NOT NULL | Date et heure ISO UTC |
| `updated_at` | TEXT | NOT NULL | Date et heure ISO UTC |

### Contraintes proposées

```sql
CHECK(length(color) = 7 AND substr(color, 1, 1) = '#');
UNIQUE(kind, name);
```

Les catégories par défaut utilisent des identifiants textuels lisibles et stables. Les catégories personnalisées peuvent utiliser un UUID afin d’éviter les collisions.

---

### Données initiales des catégories de dépenses

La migration initiale doit insérer les 26 catégories de dépenses dans l’ordre défini dans la section 10.1.

Exemple de représentation métier :

```rust
pub const DEFAULT_EXPENSE_CATEGORIES: &[DefaultCategory] = &[
    DefaultCategory::new("logement", "Logement", "Home", "#3B82F6"),
    DefaultCategory::new("alimentation", "Alimentation", "ShoppingCart", "#22C55E"),
    DefaultCategory::new("transport", "Transport", "Car", "#EAB308"),
    DefaultCategory::new("sante", "Santé", "Heart", "#EF4444"),
    DefaultCategory::new("loisirs", "Loisirs", "Gamepad2", "#A855F7"),
    DefaultCategory::new("vetements", "Vêtements", "Shirt", "#EC4899"),
    DefaultCategory::new("abonnements", "Abonnements", "RefreshCw", "#6366F1"),
    DefaultCategory::new("restaurants", "Restaurants", "Utensils", "#F97316"),
    DefaultCategory::new("voyages", "Voyages", "Plane", "#06B6D4"),
    DefaultCategory::new("education", "Éducation", "GraduationCap", "#14B8A6"),
    DefaultCategory::new("epargne", "Épargne", "PiggyBank", "#10B981"),
    DefaultCategory::new("energie", "Énergie", "Zap", "#F59E0B"),
    DefaultCategory::new("internet", "Internet", "Wifi", "#0EA5E9"),
    DefaultCategory::new("telephone", "Téléphone", "Smartphone", "#8B5CF6"),
    DefaultCategory::new("sport", "Sport", "Dumbbell", "#84CC16"),
    DefaultCategory::new("musique", "Musique", "Music", "#F43F5E"),
    DefaultCategory::new("livres", "Livres", "BookOpen", "#78716C"),
    DefaultCategory::new("enfants", "Enfants", "Baby", "#D946EF"),
    DefaultCategory::new("animaux", "Animaux", "PawPrint", "#CA8A04"),
    DefaultCategory::new("bricolage", "Bricolage", "Wrench", "#6B7280"),
    DefaultCategory::new("cadeaux", "Cadeaux", "Gift", "#F472B6"),
    DefaultCategory::new("bien-etre", "Bien-être", "Sparkles", "#C084FC"),
    DefaultCategory::new("credit", "Crédit", "CreditCard", "#FB923C"),
    DefaultCategory::new("tabac", "Tabac", "Cigarette", "#A8A29E"),
    DefaultCategory::new("assurance", "Assurance", "Shield", "#38BDF8"),
    DefaultCategory::new("autre", "Autre", "MoreHorizontal", "#9CA3AF"),
];
```

Les insertions initiales doivent être idempotentes afin de ne jamais créer de doublons.

---

## 14.3 Table `app_settings`

Une seule ligne de paramètres peut être utilisée dans le MVP.

| Colonne | Type SQLite | Contraintes | Description |
|---|---|---|---|
| `id` | INTEGER | PRIMARY KEY | Toujours `1` |
| `current_balance_cents` | INTEGER | NOT NULL | Solde bancaire actuel |
| `overdraft_limit_cents` | INTEGER | NOT NULL, CHECK >= 0 | Découvert positif |
| `currency_code` | TEXT | NOT NULL | `EUR` par défaut |
| `locale` | TEXT | NOT NULL | `fr-FR` par défaut |
| `theme` | TEXT | NOT NULL | `system`, `light`, `dark` |
| `balance_updated_at` | TEXT | NOT NULL | Dernière mise à jour du solde |
| `onboarding_completed` | INTEGER | NOT NULL | Booléen SQLite |
| `created_at` | TEXT | NOT NULL | Date et heure ISO UTC |
| `updated_at` | TEXT | NOT NULL | Date et heure ISO UTC |

---

## 14.4 Table `schema_migrations`

| Colonne | Type SQLite | Contraintes | Description |
|---|---|---|---|
| `version` | INTEGER | PRIMARY KEY | Numéro de migration |
| `name` | TEXT | NOT NULL | Nom |
| `applied_at` | TEXT | NOT NULL | Date d’application |

---

## 15. Modèles métier Rust proposés

```rust
pub struct Money {
    cents: i64,
}

pub enum TransactionKind {
    Income,
    Expense,
}

pub enum TransactionStatus {
    Pending,
    Completed,
}

pub struct Category {
    pub id: CategoryId,
    pub kind: TransactionKind,
    pub name: String,
    pub icon: String,
    pub color: String,
    pub sort_order: i32,
    pub is_default: bool,
    pub is_active: bool,
}

pub struct Transaction {
    pub id: TransactionId,
    pub kind: TransactionKind,
    pub label: String,
    pub amount: Money,
    pub date: Date,
    pub status: TransactionStatus,
    pub category_id: CategoryId,
    pub note: Option<String>,
}

pub struct BudgetSummary {
    pub current_balance: Money,
    pub pending_income: Money,
    pub pending_expenses: Money,
    pub projected_balance: Money,
    pub overdraft_limit: Money,
    pub remaining_overdraft_margin: Money,
    pub status: FinancialStatus,
}

pub enum FinancialStatus {
    Healthy,
    Warning,
    Danger,
}
```

Le code final doit utiliser les types de date et d’identifiant choisis par le projet.

---

## 16. Validation des données

### 16.1 Libellé

- Obligatoire.
- Espaces en début et fin supprimés.
- Longueur maximale : 120 caractères.
- Une chaîne composée uniquement d’espaces est refusée.

### 16.2 Montant

- Obligatoire.
- Strictement supérieur à zéro.
- Deux décimales maximum pour l’euro.
- Conversion immédiate en centimes.
- Les valeurs trop grandes pour un entier 64 bits sont refusées.

### 16.3 Date

- Obligatoire.
- Format interne ISO.
- L’interface utilise le format local.
- Une date future est autorisée pour une transaction en attente.
- Une date passée est autorisée.

### 16.4 Note

- Facultative.
- Longueur maximale : 1 000 caractères.
- Les espaces inutiles sont supprimés.

### 16.5 Découvert

- Supérieur ou égal à zéro.
- Enregistré comme montant positif.

### 16.6 Catégorie

- L’identifiant est obligatoire et unique.
- L’identifiant d’une catégorie par défaut est immuable.
- Le nom est obligatoire.
- L’icône est obligatoire.
- La couleur est obligatoire.
- La couleur doit respecter le format `#RRGGBB`.
- L’ordre d’affichage doit être un entier supérieur ou égal à zéro.
- Une catégorie de dépense ne peut pas être assignée à un revenu, et inversement.

---

## 17. Gestion des erreurs

### 17.1 Erreurs utilisateur

Exemples :

- montant invalide ;
- libellé vide ;
- date invalide ;
- catégorie manquante.

Ces erreurs doivent être affichées près du champ concerné.

### 17.2 Erreurs de stockage

En cas d’échec SQLite :

- ne pas annoncer que l’enregistrement a réussi ;
- conserver les données saisies dans le formulaire ;
- afficher un message compréhensible ;
- proposer de réessayer ;
- enregistrer les détails techniques dans les journaux locaux.

Message proposé :

> Impossible d’enregistrer la transaction. Tes modifications sont conservées dans le formulaire. Réessaie dans quelques instants.

### 17.3 Base inaccessible ou corrompue

L’application doit :

1. détecter l’erreur ;
2. éviter toute écriture supplémentaire risquée ;
3. afficher l’emplacement de la base ;
4. proposer une action de récupération ou de sauvegarde lorsque cette fonction existe ;
5. ne jamais recréer silencieusement une base vide à la place de la base existante.

### 17.4 Migration échouée

L’application doit afficher une erreur bloquante avec :

- la version de migration concernée ;
- l’emplacement de la base ;
- une option pour copier les détails techniques ;
- une option pour fermer l’application.

---

## 18. États d’interface

Chaque écran chargé depuis SQLite doit prévoir les états suivants :

- initial ;
- chargement ;
- contenu disponible ;
- contenu vide ;
- erreur ;
- action en cours ;
- action réussie.

### 18.1 Action en cours

Pendant une écriture :

- désactiver le bouton de validation ;
- empêcher les doubles clics ;
- afficher un indicateur discret ;
- réactiver le formulaire en cas d’échec.

### 18.2 Notification de succès

Après une action réussie, afficher un message temporaire :

- `Dépense ajoutée`
- `Revenu ajouté`
- `Transaction modifiée`
- `Transaction supprimée`
- `Solde mis à jour`

---

## 19. Journalisation

### 19.1 Informations autorisées

- démarrage et arrêt de l’application ;
- version de l’application ;
- durée des migrations ;
- type général d’erreur ;
- nom de l’opération échouée ;
- identifiant technique d’une transaction, si nécessaire.

### 19.2 Informations interdites par défaut

- montant du solde ;
- montant d’une transaction ;
- libellé ;
- note ;
- nom personnalisé de catégorie ;
- contenu complet de la base.

---

## 20. Design et expérience desktop

### 20.1 Direction visuelle

AfterBudget doit proposer une interface très soignée, moderne et rassurante, inspirée des applications financières et bancaires actuelles.

Le design doit transmettre :

- la maîtrise du budget ;
- la fiabilité ;
- la clarté ;
- la confidentialité ;
- la simplicité d’utilisation.

L’application doit posséder sa propre identité graphique. Elle ne doit pas donner l’impression d’être un site web responsive affiché dans une fenêtre.

### 20.2 Style attendu

- Interface moderne et épurée.
- Hiérarchie visuelle forte.
- Cartes financières lisibles.
- Typographie nette.
- Espacements réguliers.
- Bordures discrètes.
- Ombres légères uniquement lorsqu’elles améliorent la hiérarchie.
- Icônes cohérentes.
- Animations courtes et fonctionnelles.
- Aucun effet décoratif ralentissant l’utilisation.

### 20.3 Expérience réellement desktop

L’application doit être pensée pour une utilisation sur ordinateur avec une souris et un clavier.

Elle doit notamment proposer :

- une navigation persistante ;
- une densité d’information adaptée aux grands écrans ;
- des raccourcis clavier ;
- des états de survol ;
- un focus clavier visible ;
- des fenêtres modales adaptées au desktop ;
- des boîtes de dialogue natives pour ouvrir ou enregistrer un fichier ;
- des menus contextuels lorsque cela améliore réellement l’utilisation ;
- une gestion correcte du redimensionnement de la fenêtre ;
- des tableaux et listes adaptés à la souris et au clavier.

Les composants ne doivent pas reproduire inutilement les conventions d’une interface mobile.

### 20.4 Palette fonctionnelle

La couleur ne doit jamais être le seul moyen de transmettre une information.

Utilisations recommandées :

- positif : solde sain ou revenu ;
- neutre : information générale ;
- attention : utilisation prévue du découvert ;
- danger : dépassement prévu ;
- accent principal : actions importantes et sélection active.

Chaque état doit également comporter un texte, une icône ou un libellé explicite.

### 20.5 Design system

Le projet doit définir des composants cohérents pour :

- les boutons principaux, secondaires et destructifs ;
- les champs texte et monétaires ;
- les sélecteurs ;
- les cartes de synthèse ;
- les badges de statut ;
- les tableaux et listes ;
- les fenêtres modales ;
- les notifications ;
- les états vides ;
- les indicateurs de chargement ;
- la navigation ;
- les infobulles ;
- les séparateurs ;
- les graphiques et leurs légendes.

### 20.6 Compatibilité visuelle multiplateforme

Le langage visuel général doit rester cohérent sur Windows, macOS et Linux, tout en respectant les principales conventions de chaque système.

Les différences natives acceptables concernent notamment :

- les boîtes de dialogue de fichiers ;
- les raccourcis `Ctrl` ou `Cmd` ;
- l’emplacement de certains menus ;
- les conventions de fermeture et de validation ;
- le rendu des polices disponibles.

---

## 21. Sécurité et confidentialité

### 21.1 Principes

- Stockage local uniquement.
- Aucun envoi automatique.
- Aucun compte.
- Aucun mot de passe applicatif dans le MVP.
- Aucun secret distant.
- Aucun accès bancaire.
- Aucune collecte de données personnelles.

### 21.2 Emplacement de la base

La base doit être placée dans le répertoire de données applicatives recommandé par le système d’exploitation.

Exemples conceptuels :

- Windows : répertoire de données de l’utilisateur ;
- macOS : `Application Support` ;
- Linux : répertoire conforme à XDG.

Le chemin ne doit pas être codé en dur.

### 21.3 Export et restauration SQLite

L’export et l’import de la base SQLite font partie du MVP.

#### Export

L’application doit :

1. terminer ou annuler proprement les écritures en cours ;
2. produire une copie cohérente de la base ;
3. utiliser une boîte de dialogue native pour choisir l’emplacement ;
4. proposer un nom de fichier explicite, par exemple :

```text
afterbudget-backup-2026-08-03.sqlite
```

5. confirmer le succès de l’export.

Une simple copie du fichier pendant une écriture active ne doit pas être utilisée si elle risque de produire une sauvegarde incohérente.

L’utilisation de l’API de sauvegarde SQLite ou d’un mécanisme équivalent est recommandée.

#### Import

L’application doit :

1. demander à l’utilisateur de sélectionner un fichier ;
2. vérifier que le fichier est une base SQLite valide ;
3. vérifier que le schéma correspond à AfterBudget ;
4. vérifier que la version est compatible ou migrable ;
5. créer une sauvegarde automatique de la base courante ;
6. fermer les connexions actives ;
7. restaurer la base de manière atomique ;
8. appliquer les migrations nécessaires ;
9. rouvrir la base ;
10. recharger l’interface et recalculer les indicateurs ;
11. confirmer la réussite de l’opération.

En cas d’échec, l’application doit restaurer automatiquement la base précédente.

### 21.4 Sécurité des fichiers importés

- Ne jamais exécuter de contenu provenant de la base importée.
- Refuser les bases ne contenant pas les tables minimales attendues.
- Refuser les versions de schéma trop récentes et non prises en charge.
- Vérifier les contraintes essentielles avant le remplacement.
- Ne jamais supprimer la base actuelle avant d’avoir validé le fichier importé.
- Conserver la sauvegarde automatique tant que le nouvel import n’a pas été validé.

---

## 22. Critères d’acceptation

## 22.1 Calcul nominal

### Données

```text
Solde actuel : -360 €
Revenus en attente : 1 207 €
Dépenses en attente : 1 325 €
Découvert autorisé : 500 €
```

### Résultat attendu

```text
Solde prévisionnel : -478 €
Marge avant dépassement : 22 €
Statut : Attention
```

---

## 22.2 Solde positif

### Données

```text
Solde actuel : 300 €
Revenus en attente : 1 500 €
Dépenses en attente : 1 200 €
Découvert autorisé : 200 €
```

### Résultat attendu

```text
Solde prévisionnel : 600 €
Statut : En forme
```

---

## 22.3 Dépassement du découvert

### Données

```text
Solde actuel : -400 €
Revenus en attente : 200 €
Dépenses en attente : 500 €
Découvert autorisé : 500 €
```

### Résultat attendu

```text
Solde prévisionnel : -700 €
Marge avant dépassement : -200 €
Dépassement prévu : 200 €
Statut : Danger
```

---

## 22.4 Transaction réalisée

### Étant donné

- une dépense de 100 € marquée comme en attente ;
- un solde bancaire actuel qui ne contient pas encore cette dépense.

### Lorsque

L’utilisateur met à jour son solde bancaire réel après le débit, puis marque la dépense comme réalisée.

### Alors

- la dépense n’est plus incluse dans `reste à payer` ;
- elle reste visible dans les dépenses du mois ;
- les statistiques restent cohérentes ;
- le calcul ne soustrait pas deux fois la dépense.

---

## 22.5 Persistance

### Lorsque

1. l’utilisateur ajoute une transaction ;
2. il ferme l’application ;
3. il relance l’application.

### Alors

La transaction, les paramètres et les calculs associés sont restaurés.

---

## 22.6 Filtrage

### Étant donné

Une liste contenant des revenus et dépenses, réalisés et en attente.

### Lorsque

L’utilisateur filtre sur :

```text
Type = Dépense
Statut = En attente
```

### Alors

Seules les dépenses en attente sont affichées.

---

## 22.7 Suppression annulée

### Lorsque

L’utilisateur lance la suppression puis annule la confirmation.

### Alors

La transaction reste présente et aucune donnée n’est modifiée.

---

## 22.8 Absence de données

### Lorsque

Le mois sélectionné ne contient aucune transaction.

### Alors

- aucun graphique vide ou cassé n’est affiché ;
- un message d’état vide est visible ;
- les boutons d’ajout sont accessibles.

---

## 22.9 Catégories par défaut

### Lors du premier lancement

L’application doit créer exactement les catégories de dépenses définies dans la section 10.1.

### Résultat attendu

- chaque catégorie possède le bon identifiant ;
- le nom correspond à la spécification ;
- l’icône correspond à la spécification ;
- la couleur correspond à la spécification ;
- l’ordre d’affichage est stable ;
- aucune catégorie par défaut n’est créée en double lors des lancements suivants.

---

## 22.10 Conservation des catégories lors d’un transfert

### Lorsque

Une base SQLite est exportée puis importée sur un autre système.

### Alors

- les identifiants sont conservés ;
- les noms sont conservés ;
- les icônes sont conservées ;
- les couleurs sont conservées ;
- l’ordre et l’état actif des catégories sont conservés.

---

## 22.11 Export de la base SQLite

### Lorsque

1. l’utilisateur ouvre les paramètres ;
2. il choisit `Exporter mes données` ;
3. il sélectionne un emplacement valide.

### Alors

- un fichier SQLite complet est créé ;
- la base exportée peut être validée ;
- toutes les données courantes sont présentes ;
- la base utilisée par l’application n’est pas modifiée ;
- un message de réussite affiche l’emplacement du fichier.

---

## 22.12 Import d’une base SQLite valide

### Étant donné

Une base exportée par une version compatible d’AfterBudget.

### Lorsque

L’utilisateur choisit de l’importer et confirme le remplacement.

### Alors

- une sauvegarde automatique de la base courante est créée ;
- la base importée est validée ;
- les données importées remplacent les données courantes ;
- l’application recharge le tableau de bord ;
- les calculs correspondent aux données importées.

---

## 22.13 Import d’une base invalide

### Étant donné

Un fichier qui n’est pas une sauvegarde AfterBudget valide.

### Lorsque

L’utilisateur tente de l’importer.

### Alors

- l’import est refusé ;
- un message explique que le fichier est invalide ou incompatible ;
- la base courante reste inchangée ;
- aucune donnée n’est perdue.

---

## 22.14 Transfert multiplateforme

### Étant donné

Une base exportée depuis Windows, macOS ou Linux.

### Lorsque

Elle est importée sur un autre système pris en charge.

### Alors

- l’import réussit ;
- les transactions, catégories et paramètres sont conservés ;
- les dates et montants restent identiques.

---

## 23. Tests

### 23.1 Tests unitaires

À couvrir en priorité :

- addition et soustraction de montants ;
- conversion texte vers centimes ;
- formatage monétaire ;
- calcul du solde prévisionnel ;
- calcul de la marge ;
- choix du statut financier ;
- validation d’une transaction ;
- agrégation par catégorie ;
- filtrage par mois.

### 23.2 Tests d’intégration

- création de la base ;
- application des migrations ;
- création des 26 catégories de dépenses par défaut ;
- absence de doublons après plusieurs démarrages ;
- conservation des identifiants, icônes et couleurs ;
- validation du format hexadécimal des couleurs ;
- ajout puis lecture d’une transaction ;
- modification ;
- suppression ;
- rollback en cas d’échec ;
- persistance des paramètres ;
- filtres SQL ;
- agrégations mensuelles.

### 23.3 Tests d’import et d’export

- export d’une base vide ;
- export d’une base contenant des transactions ;
- import d’une base valide ;
- import d’un fichier non SQLite ;
- import d’une base SQLite avec un schéma inconnu ;
- import d’une ancienne version nécessitant une migration ;
- refus d’une version plus récente non prise en charge ;
- interruption simulée pendant l’import ;
- restauration automatique après un échec ;
- transfert Windows vers Linux ;
- transfert Linux vers macOS ;
- transfert macOS vers Windows.

### 23.4 Tests d’interface

- premier lancement ;
- ajout d’un revenu ;
- ajout d’une dépense ;
- modification du statut ;
- suppression avec confirmation ;
- navigation entre les écrans ;
- navigation clavier ;
- états vides ;
- affichage d’une erreur SQLite simulée.

### 23.5 Tests manuels multiplateformes

- Windows ;
- macOS ;
- Linux ;
- thème clair ;
- thème sombre ;
- différentes tailles de fenêtre ;
- séparateur décimal avec virgule ;
- montants négatifs ;
- grandes listes de transactions.

---

## 24. Données de démonstration

En environnement de développement uniquement :

```text
Solde actuel : -360 €
Découvert autorisé : 500 €

Revenu :
- Salaire
- 1 207 €
- En attente

Dépenses :
- Loyer
- 850 €
- En attente

- Courses
- 120 €
- En attente

- Assurance
- 155 €
- En attente

- Énergie
- 200 €
- En attente
```

Ces données permettent de vérifier rapidement l’affichage du statut `Attention`.

Aucune donnée de démonstration ne doit être créée automatiquement dans une installation de production.

---

## 25. Plan de réalisation proposé

### Phase 1 — Fondation

- Initialiser le projet Rust/Iced.
- Mettre en place la navigation.
- Définir les modèles métier.
- Implémenter le type `Money`.
- Configurer SQLite.
- Créer les migrations.
- Ajouter les catégories par défaut.

### Phase 2 — Transactions

- Créer les repositories.
- Ajouter le formulaire.
- Ajouter la liste.
- Ajouter modification et suppression.
- Ajouter les filtres.
- Ajouter les validations.

### Phase 3 — Calcul du budget

- Implémenter le service de calcul.
- Créer les cartes de synthèse.
- Ajouter les statuts.
- Ajouter les alertes de découvert.
- Ajouter la navigation par mois.

### Phase 4 — Statistiques

- Ajouter les agrégations.
- Ajouter les résumés par catégorie.
- Ajouter les états vides.
- Ajouter les premières visualisations.

### Phase 5 — Paramètres et données

- Solde.
- Découvert.
- Devise.
- Thème.
- Catégories.
- Export de la base SQLite.
- Import et restauration de la base SQLite.
- Sauvegarde automatique avant import.
- Informations de confidentialité.

### Phase 6 — Qualité

- Tests unitaires.
- Tests d’intégration.
- Tests multiplateformes.
- Gestion des erreurs.
- Accessibilité.
- Packaging et publication.

---

## 26. Décisions à confirmer avant implémentation

Les points suivants ne sont pas entièrement définis par la présentation actuelle du produit et doivent être validés :

1. L’application doit-elle gérer un seul compte bancaire ou plusieurs comptes ?
2. Le solde actuel est-il uniquement manuel ?
3. Le MVP doit-il inclure les transactions récurrentes ?
4. Le MVP doit-il proposer un export CSV ?
5. Quelles plateformes sont prioritaires ?
6. Les catégories personnalisées sont-elles incluses dès la première version ?
7. Le thème sombre est-il obligatoire pour le MVP ?
8. Les statistiques doivent-elles inclure de véritables graphiques dès la première version ?
9. Faut-il conserver un historique des modifications du solde actuel ?
10. Le changement du statut d’une transaction doit-il seulement changer son état, ou aussi proposer la mise à jour automatique du solde actuel ?

### Décision recommandée pour le MVP

- Un seul compte bancaire.
- Solde actuel renseigné manuellement.
- Aucune transaction récurrente dans la première version.
- Export et import de la base SQLite inclus dans le MVP.
- Catégories personnalisées incluses.
- Statistiques simples sans bibliothèque graphique complexe.
- Mise à jour du solde indépendante du changement de statut.
- Rappel visible demandant à l’utilisateur de maintenir son solde actuel à jour.

---

## 27. Définition de « terminé »

Une fonctionnalité est terminée lorsque :

- son comportement correspond aux présentes spécifications ;
- ses états de chargement, vide et erreur sont traités ;
- ses validations sont implémentées ;
- les données sont persistées correctement ;
- les calculs concernés sont recalculés ;
- les tests unitaires principaux sont présents ;
- aucun crash connu n’est reproductible ;
- l’utilisation au clavier est possible ;
- les textes français sont relus ;
- la fonctionnalité fonctionne sur les plateformes officiellement prises en charge.

---

## 28. Résumé du MVP

AfterBudget doit permettre à un utilisateur de :

1. saisir son solde bancaire actuel ;
2. renseigner son découvert autorisé ;
3. ajouter ses revenus et dépenses ;
4. indiquer ce qui est déjà reçu ou payé ;
5. voir ce qui reste à recevoir et à payer ;
6. connaître son solde prévisionnel de fin de mois ;
7. savoir s’il restera positif, utilisera son découvert ou dépassera sa limite ;
8. consulter des statistiques mensuelles simples ;
9. exporter sa base SQLite pour sauvegarder ou transférer ses données ;
10. importer une base SQLite précédemment exportée ;
11. utiliser l’application sur Windows, macOS ou Linux ;
12. conserver toutes ses données localement, sans compte et sans connexion Internet.
