# DESIGN_SYSTEM — AfterBudget

Référence unique des décisions visuelles. Toute valeur qui apparaît dans plus d'un endroit
est définie ici et **uniquement** dans `src/ui/theme/`.

---

## 1. Direction : « Encre & Cuivre »

AfterBudget est un **registre de comptes personnel**, pas un tableau de bord d'entreprise.
La métaphore retenue est celle du **carnet de comptes relié** : un papier chaud, une encre
profonde, une seule signature métallique cuivrée.

Trois décisions fondatrices en découlent.

**1. Les neutres sont chauds.** Le thème clair n'est pas gris-bleu (#F5F6F8) mais parchemin
(#F4F0E9). Le thème sombre n'est pas ardoise bleutée (#10131A) mais charbon chaud (#14110D).
C'est ce qui distingue immédiatement AfterBudget d'une webapp générique et ce qui porte
l'adjectif « chaleureuse ». Le thème sombre n'est donc pas une inversion : c'est le même
papier vu à la lumière d'une lampe.

**2. Une seule couleur d'accent, et elle n'est pas sémantique.** Le cuivre (`accent`) ne
signifie jamais « bien » ni « mal ». Il signale **l'interactif** : bouton principal, écran
actif, anneau de focus, sélection. Le vert et le rouge sont **réservés** au sens de l'argent
(revenu / dépense). Aucune collision possible entre « ce sur quoi je peux cliquer » et
« comment va mon budget ».

**3. La hiérarchie vient de la surface, pas de la bordure.** Le design précédent donnait à
tous les blocs la même bordure 1 px. Ici, l'importance d'un bloc se lit à son **niveau
d'élévation** (`fond` → `surface` → `surface_haute`) et à son **format**, pas à un cadre.
Une seule chose par écran a le droit d'être hors-échelle : la donnée principale.

### Ce que le produit n'est pas

Pas de dégradés, pas de verre dépoli, pas d'ombres portées longues, pas de coins très
arrondis, pas d'illustrations. Le raffinement vient de la justesse des neutres, de la
densité maîtrisée et de la rareté de l'accent.

### Références étudiées

YNAB (primauté absolue de la question « combien me reste-t-il ? », vocabulaire direct),
Copilot Money (un seul accent, listes denses, typographie sobre), Monarch Money (hiérarchie
tableau de bord, chiffre-héros unique), Actual Budget (densité desktop, tableau plutôt que
cartes), Wallet by BudgetBakers (catégories iconographiées), MoneyWiz (registre à colonnes
alignées). Aucun écran n'a été repris : l'identité chaude papier/cuivre et le vocabulaire
« Ce qui reste à la fin du mois » sont propres à AfterBudget.

---

## 2. Couleurs — rôles

Défini dans `src/ui/theme/palette.rs`. **Aucune couleur littérale ailleurs dans le code**,
à la seule exception des couleurs de catégorie choisies par l'utilisateur (stockées en base,
converties par `couleur_categorie`).

| Rôle | Clair | Sombre | Usage |
|---|---|---|---|
| `fond` | `#F4F0E9` | `#14110D` | Fond de fenêtre, zone de contenu |
| `fond_rail` | `#EDE7DC` | `#100D0A` | Barre de navigation latérale |
| `surface` | `#FFFCF6` | `#1E1A15` | Cartes, panneaux, lignes de liste |
| `surface_haute` | `#FFFFFF` | `#272219` | Modales, menus, survol de ligne |
| `surface_basse` | `#EAE4D8` | `#100D0A` | Champs, encoches, pistes de jauge |
| `bordure` | `#E0D8C8` | `#332C22` | Séparateurs, contours au repos |
| `bordure_forte` | `#C9BDA8` | `#4A4032` | Contour au survol, contour de modale |
| `texte_fort` | `#1C1813` | `#F6F1E7` | Titres, montants, valeurs |
| `texte` | `#4A4238` | `#D5CCBD` | Corps, libellés |
| `texte_doux` | `#7A7063` | `#9C9284` | Métadonnées, aides, unités |
| `accent` | `#9C5A2E` | `#E09A5F` | Interactif : principal, actif, focus |
| `accent_contraste` | `#FFFCF6` | `#17130E` | Texte posé sur `accent` |
| `accent_surface` | `#F7E9DC` | `#382718` | Fond d'état actif / sélection |
| `revenu` | `#1F6B4A` | `#6FC49A` | Sens : argent qui entre |
| `depense` | `#A32B33` | `#F08A88` | Sens : argent qui sort |
| `succes` | `#1F6B4A` | `#6FC49A` | État : confirmé, sain |
| `attention` | `#8A6208` | `#E0B45C` | État : à surveiller |
| `danger` | `#A32B33` | `#F08A88` | État : risque, destructif |
| `succes_surface` | `#DCEDE3` | `#1B3327` | Fond de badge / bandeau succès |
| `attention_surface` | `#FAEFD3` | `#372B14` | Fond de badge / bandeau attention |
| `danger_surface` | `#FADEDE` | `#3A1E1E` | Fond de badge / bandeau danger |
| `voile` | `rgba(28,24,19,0.45)` | `rgba(0,0,0,0.6)` | Backdrop de modale |

### Teintes de catégorie

Les catégories livrées par défaut portent des couleurs web très saturées (`#3B82F6`,
`#22C55E`…). Posées telles quelles sur un fond parchemin ou sur un charbon chaud, elles
crèvent l'écran et détruisent l'identité du produit.

`couleur_categorie` les **harmonise** : la teinte est conservée — c'est elle qui identifie
la catégorie — mais la saturation est plafonnée à 0,46 et la clarté ramenée à 0,40 en thème
clair, 0,66 en thème sombre. Deux catégories de teintes distinctes restent distinctes
(vérifié par test), et la même catégorie s'éclaircit automatiquement en thème sombre.

### Contrastes vérifiés (WCAG AA)

Mesurés sur `surface` du thème correspondant, testés par `palette.rs` :

| Paire | Clair | Sombre |
|---|---|---|
| `texte_fort` | 17,3:1 | 15,4:1 |
| `texte` | 9,6:1 | 10,9:1 |
| `texte_doux` | 4,7:1 | 5,6:1 |
| `accent` | 5,2:1 | 7,4:1 |
| `revenu` | 6,2:1 | 8,3:1 |
| `depense` | 6,9:1 | 7,2:1 |
| `attention` | 5,3:1 | 9,0:1 |
| `accent_contraste` sur `accent` | 5,4:1 | 7,9:1 |

Tous ≥ 4,5:1. Un test unitaire (`contrastes_minimaux_respectes`) échoue si une future
retouche descend sous le seuil.

---

## 3. Typographie

Iced/cosmic-text n'a pas de fonte embarquée : la fonte système est utilisée. La graisse
n'est donc **jamais** le seul porteur de hiérarchie — taille, couleur et espace la portent.

`src/ui/theme/typographie.rs` définit des **rôles**, pas des tailles :

| Rôle | Taille | Graisse | Couleur | Usage |
|---|---|---|---|---|
| `montant_heros` | 46 | Light | selon état | Le solde prévisionnel. Un seul par écran. |
| `titre_ecran` | 24 | Semibold | `texte_fort` | En-tête d'écran |
| `montant_fort` | 22 | Medium | selon sens | Montant d'indicateur secondaire |
| `titre_section` | 16 | Semibold | `texte_fort` | Titre de bloc |
| `corps_fort` | 14 | Medium | `texte_fort` | Libellé de transaction, valeur |
| `corps` | 14 | Normal | `texte` | Texte courant |
| `libelle` | 13 | Medium | `texte` | Étiquette de champ |
| `legende` | 12 | Normal | `texte_doux` | Métadonnée, aide, sous-titre |
| `micro` | 11 | Medium | `texte_doux` | En-tête de colonne, badge (majuscules, interlettrage) |

**Chiffres.** Les montants d'une **colonne alignée** — liste de transactions, récapitulatif,
légendes de statistiques — utilisent `Font::MONOSPACE` : c'est ce qui garantit que les
virgules et les unités se superposent d'une ligne à l'autre. Les montants **isolés** —
solde héros, indicateurs, chiffres clés — utilisent la fonte proportionnelle : en chasse
fixe ils prendraient un air de ticket de caisse.

**Format.** Toujours `Money::format_fr()` : espace insécable `U+00A0` pour les milliers,
virgule décimale, espace insécable avant `€`. Jamais de formatage local dans une vue.

> L'espace **fine** insécable `U+202F`, typographiquement préférable, a été abandonnée :
> les fontes système proportionnelles ne la fournissent pas toutes et le montant héros
> affichait un rectangle de substitution. `Money::from_input` accepte néanmoins les deux,
> afin qu'un montant affiché reste ressaisissable tel quel.

---

## 4. Espacements, rayons, ombres

`src/ui/theme/espacements.rs`. Échelle à pas 4, plus deux demi-pas pour les composants denses.

| Jeton | px | Usage |
|---|---|---|
| `Esp::XXS` | 2 | Écart intra-ligne |
| `Esp::XS` | 4 | Libellé ↔ valeur |
| `Esp::SM` | 8 | Éléments d'un même groupe |
| `Esp::MD` | 12 | Padding de ligne dense, écart de champs |
| `Esp::LG` | 16 | Padding de carte, écart de blocs |
| `Esp::XL` | 24 | Padding d'écran, écart de sections |
| `Esp::XXL` | 32 | Respiration de modale |
| `Esp::XXXL` | 48 | État vide, onboarding |

| Rayon | px | Usage |
|---|---|---|
| `Rayon::XS` | 4 | Pastille, jauge |
| `Rayon::SM` | 6 | Badge, bouton compact |
| `Rayon::MD` | 8 | Bouton, champ |
| `Rayon::LG` | 12 | Carte, panneau |
| `Rayon::XL` | 16 | Carte héros, modale |
| `Rayon::PLEIN` | 999 | Pastille ronde, indicateur |

Ombres : trois niveaux uniquement (`Ombre::carte`, `Ombre::flottant`, `Ombre::modale`),
teintées à partir de la palette pour rester crédibles en thème sombre.

**Densités de ligne** : `Densite::Compacte` (32 px), `Densite::Normale` (44 px),
`Densite::Confortable` (56 px). La liste de transactions est en `Normale`.

---

## 5. Mise en page desktop

```
┌────────┬─────────────────────────────────────────────────┐
│  ▣     │  en-tête d'écran  (titre · sélecteur · actions)  │  ← fixe
│ marque ├─────────────────────────────────────────────────┤
│ ┌────┐ │                                                 │
│ │ ▣  │ │  contenu défilant                               │  ← scrollable
│ │Acc.│ │                                                 │
│ └────┘ │                                                 │
│  ▢     │                                                 │
│ Trans. │                                                 │
│ ─────  │                                                 │
│  ☾     │                                                 │
│ Sombre │                                                 │
└────────┴─────────────────────────────────────────────────┘
   92 px                                   [ toast ] ← bas-droite
```

- Fenêtre par défaut **1240 × 780**, minimum **860 × 620**.
- Le rail de navigation est **persistant** et occupe toute la hauteur.
- L'en-tête d'écran est **fixe** ; seul le contenu défile. Le titre reste donc toujours visible.
- La zone de contenu a une **largeur maximale de 1320 px** centrée : sur un écran très large,
  le contenu ne s'étire pas indéfiniment, il gagne des marges.
- Les toasts sont ancrés **en bas à droite**, jamais par-dessus la navigation.

### Points de rupture

| Largeur de fenêtre | Comportement |
|---|---|
| `< 1040 px` | Rail **compact** : tuile carrée de 52 px sans libellé, 60 px de large, intitulé en infobulle. |
| `≥ 1040 px` | Rail **étendu** : tuile carrée de 76 px, icône au-dessus du libellé, 92 px de large. |
| `< 1180 px` | Tableau de bord : les indicateurs passent sous le bloc héros. Colonne « Statut » masquée dans la liste (le badge revient dans la colonne libellé). |
| `≥ 1180 px` | Tableau de bord en deux colonnes (héros 2/3, colonne latérale 1/3). |
| `< 980 px` | Barre de filtres sur deux lignes. Statistiques en une colonne. |

Ce n'est **pas** une adaptation mobile : rien ne passe jamais en pile pleine largeur pour
téléphone, aucune cible tactile surdimensionnée, la souris et le clavier restent le mode
d'entrée supposé.

---

## 6. Composants

`src/ui/composants/`. Un fichier = un composant. Chaque composant reçoit la `Palette` et
les seules données dont il a besoin — **jamais `&AppState`**, sauf les vues d'écran.

### Primitives

| Composant | API | Variantes |
|---|---|---|
| `icone` | `icone(Icone, taille, couleur)` | 62 icônes SVG au trait, dessinées pour le produit, teintables. `Icone::depuis_nom("ShoppingCart")` résout les noms stockés en base. |
| `bouton` | `bouton(contenu, message, palette).variante(V).taille(T)` | `Principal`, `Secondaire`, `Discret`, `Destructif`, `Fantome` × `Normale`, `Compacte`, `Icone` |
| `champ` | `champ_texte`, `champ_montant`, `champ_date`, `champ_recherche`, `selecteur` | Chacun : libellé + contrôle + erreur + aide |
| `carte` | `carte(contenu, palette)` | `Plate`, `Elevee`, `Accent`, `Contour` |
| `badge` | `badge(texte, Ton, palette)` | `Neutre`, `Succes`, `Attention`, `Danger`, `Accent` — **toujours pastille + texte**, jamais couleur seule. Le ton `Neutre` utilise `texte` et non `texte_doux` : ce dernier ne passe pas 4,5:1 sur `surface_basse` en thème clair. |
| `jauge` | `jauge(fraction, couleur, palette)` | Piste + remplissage, minimum visible 2 px |
| `anneau` | `anneau(segments, palette)` | Donut `canvas` pour la répartition |
| `separateur` | `separateur(palette)` | Filet 1 px |
| `infobulle` | `infobulle(contenu, texte, palette)` | Position adaptative |
| `etat` | `etat_vide`, `etat_chargement`, `etat_erreur` | Icône + titre + explication + action |
| `modale` | `modale(titre, corps, actions, largeur, palette)` | Voile + carte, `Échap` ferme, clic sur le voile ferme |
| `notification` | `notification(texte, Ton, palette)` | Toast bas-droite, auto-fermeture 5 s |

### Composants de produit

| Composant | Rôle |
|---|---|
| `navigation` | **Rail vertical à tuiles carrées** : chaque entrée est un carré de 76 px portant une pastille d'icône de 32 px au-dessus de son libellé. C'est le motif des barres d'outils de logiciels de création (Affinity, DaVinci) et du *navigation rail* de Material 3 — délibérément pas la liste de liens d'une barre latérale web. |
| `en_tete_ecran` | Titre + sous-titre + zone d'actions, fixe |
| `selecteur_mois` | `‹ Août 2026 ›` + « Aujourd'hui » quand on n'y est pas |
| `bloc_solde` | Le chiffre-héros : montant, état, jauge de découvert, formule |
| `indicateur` | Carte d'indicateur secondaire : libellé, montant, comparaison, filet coloré |
| `pastille_categorie` | Icône de catégorie dans un disque teinté + nom |
| `ligne_transaction` | Ligne de tableau à colonnes alignées, survol, actions au survol |
| `entete_colonnes` | En-tête de tableau (`micro`, majuscules) |
| `section_parametres` | Titre + description + contrôles alignés à droite |

### Le rail en détail

| Élément | Valeur |
|---|---|
| Largeur du rail | 92 px étendu · 60 px compact |
| Tuile | 76 × 76 px étendu · 52 × 52 px compact, rayon `MD` |
| Pastille d'icône | 32 × 32 px, rayon `MD`, contour 1 px |
| Icône | 18 px, centrée dans la pastille |
| Écart pastille ↔ libellé | 6 px |
| Libellé | rôle `micro` (11 px), sans retour à la ligne |
| Bord droit du rail | filet 1 px `bordure` |

**État actif** : pastille en `accent` plein, icône en `accent_contraste`, libellé en `accent`.
La masse colorée se limite ainsi à 32 × 32 px — le rail reste calme même avec une couleur
saturée. **État inactif** : pastille en `surface` avec contour `bordure`.

> `surface_basse` a été essayée pour la pastille inactive puis écartée : sur `fond_rail`
> elle est indiscernable en thème clair et **strictement identique** en thème sombre, ce
> qui faisait disparaître la forme carrée dès qu'une entrée n'était pas active.

**Survol** : seule la tuile réagit, par une surface légère. Iced ne propageant pas l'état
de survol d'un conteneur à ses enfants, la pastille ne peut pas répondre au survol.

### Règles

- Un composant ne lit jamais la base, n'appelle jamais un repository, ne décide jamais
  d'une règle métier. Il reçoit des valeurs déjà formatées ou des types du domaine.
- Les variantes sont des `enum` explicites, jamais des `bool` empilés.
- Un écran ne redéfinit jamais un bouton, une carte, un champ ou un badge.
- Toute fonction de vue dépassant ~60 lignes est découpée en sections nommées.

---

## 7. États interactifs

Iced expose `Status` sur `button`, `text_input`, `pick_list`, `scrollable`, `svg`.
**Toutes** les fermetures de style de l'application exploitent ce paramètre.

| État | Traitement |
|---|---|
| Repos | Surface et bordure de base |
| **Survol** | Surface montée d'un niveau (`surface` → `surface_haute`), bordure → `bordure_forte`. Sur bouton plein : accent assombri de 8 % (clair) / éclairci de 8 % (sombre). |
| **Focus** | Anneau `accent` de 2 px. Visible dans les deux thèmes, jamais supprimé. |
| **Pression** | Surface descendue d'un niveau, sans déplacement de contenu. |
| **Désactivé** | Opacité de couleur ramenée vers `texte_doux`, fond `surface_basse`, pas de bordure d'accent. `on_press` absent (Iced grise automatiquement). |
| **Sélectionné / actif** | Fond `accent_surface`, texte `accent`, **plus** un repère de forme (filet vertical dans le rail, pastille dans un groupe de filtres) — jamais la couleur seule. |

---

## 8. Icônes

62 icônes SVG 24×24, trait 1,75 px, bouts arrondis, `currentColor`, embarquées via
`include_bytes!` dans `assets/icones/`. Aucun emoji nulle part.

Navigation : `accueil`, `transactions`, `statistiques`, `parametres`.
Actions : `plus`, `moins`, `crayon`, `corbeille`, `croix`, `coche`, `recherche`, `filtre`,
`import`, `export`, `chevron-gauche`, `chevron-droit`, `chevron-bas`, `calendrier`.
Sens : `fleche-entrante`, `fleche-sortante`, `tendance-haut`, `tendance-bas`.
États : `info`, `alerte`, `danger`, `succes`, `sablier`, `soleil`, `lune`, `cadenas`.
Catégories : `maison`, `panier`, `voiture`, `coeur`, `manette`, `vetement`, `cycle`,
`couverts`, `avion`, `diplome`, `tirelire`, `eclair`, `onde`, `mobile`, `halteres`,
`musique`, `livre`, `enfant`, `patte`, `cle`, `cadeau`, `etincelles`, `carte-bancaire`,
`cigarette`, `bouclier`, `portefeuille`, `pieces`, `etoile`, `retour`, `etiquette`,
`mallette`, plus `points` en repli universel.

`Icone::depuis_nom` mappe les noms Lucide stockés en base (`Home`, `ShoppingCart`, `Car`,
`Heart`, `Gamepad2`, `Wallet`, `Tag`…) vers ce jeu. Un test vérifie que **chacune** des 35
catégories livrées par défaut obtient une icône dédiée — seule « Autre » a le droit de
tomber sur `points`. Une catégorie créée avec un nom inconnu reste lisible grâce au repli.

---

## 9. Thèmes

Le thème est une **donnée d'application** (`ThemeMode`), pas le thème Iced. `iced::Theme`
reste neutre ; toute la peinture passe par `Palette`. Cela garantit qu'aucun widget ne
retombe silencieusement sur les couleurs par défaut d'Iced.

Le thème sombre n'est pas une inversion :

- les surfaces montent en luminosité avec l'élévation (`fond` 8 % → `surface` 12 % →
  `surface_haute` 15 %), alors qu'en clair elles montent vers le blanc ;
- l'accent cuivre est **éclairci et désaturé** (#E09A5F) pour ne pas vibrer sur fond sombre,
  et le texte posé dessus devient sombre ;
- le vert et le rouge sont **éclaircis** (#6FC49A, #F08A88) : les teintes profondes du
  thème clair seraient illisibles ;
- les fonds sémantiques sont des teintes sombres saturées, pas des pastels assombris ;
- les ombres deviennent des halos noirs très diffus, plus faibles, car l'élévation est
  déjà portée par la luminosité.

Réglage : `Clair` / `Sombre` dans Paramètres, plus une bascule rapide en pied de rail.

---

## 10. Animations et micro-interactions

Iced 0.13 n'a pas de moteur de transition. On assume donc des **changements d'état
instantanés mais systématiques**, plus deux animations réelles pilotées par souscription :

1. **Notification** — apparaît, reste 5 s, se ferme seule (`Message::Tick`, cadence 1 s).
2. **Indicateur de chargement** — quatre points dont l'intensité tourne, cadencé par le
   même `Tick`.

Tout le reste (survol, focus, pression, sélection) est un changement de style immédiat.
Aucune animation décorative, aucun mouvement de mise en page.

---

## 11. Accessibilité

1. **Jamais la couleur seule.** Un revenu porte une icône de flèche entrante, le signe `+`,
   et le mot « Revenu » dans son infobulle. Une dépense porte la flèche sortante et le signe
   `−`. Un statut porte son libellé (« En attente » / « Réalisé ») en plus de sa teinte.
   Chaque catégorie porte son icône en plus de sa couleur. Chaque segment d'anneau est
   répété dans une légende textuelle avec son montant et son pourcentage.
2. **Contraste** ≥ 4,5:1 pour tout texte, ≥ 3:1 pour les contours porteurs de sens ;
   vérifié par test unitaire.
3. **Taille minimale de texte** : 11 px, réservée aux en-têtes de colonne en majuscules.
4. **Zones cliquables** ≥ 32 × 32 px, même pour les boutons-icônes.
5. **Focus** toujours visible, jamais retiré.
6. **Erreurs** affichées sous le champ concerné, avec icône, texte explicite, et le contour
   du champ en `danger`.
7. **Actions destructives** : toujours une confirmation, l'action sûre est le bouton par
   défaut, l'action destructive est visuellement distincte et à droite.

---

## 12. Raccourcis clavier

| Raccourci | Action |
|---|---|
| `Ctrl` `N` | Nouvelle dépense |
| `Ctrl` `Maj` `N` | Nouveau revenu |
| `Ctrl` `F` | Aller à la recherche (bascule sur Transactions) |
| `Ctrl` `S` | Enregistrer le formulaire ouvert |
| `Échap` | Fermer la modale / le formulaire / la notification |
| `Ctrl` `←` / `→` | Mois précédent / suivant |
| `Ctrl` `1…4` | Accueil / Transactions / Statistiques / Paramètres |

Les raccourcis à modificateur ne perturbent pas la saisie de texte. `Échap` est capté
globalement.
