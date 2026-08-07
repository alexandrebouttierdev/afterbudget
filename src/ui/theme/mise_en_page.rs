//! Règles d'adaptation à la taille de fenêtre.
//!
//! Il ne s'agit pas d'un passage en interface mobile : rien ne devient une
//! colonne unique pleine largeur, aucune cible n'est agrandie. On décide
//! seulement du repli du rail, du nombre de colonnes et des colonnes de tableau
//! qu'on peut se permettre d'afficher.

use super::espacements::Esp;

/// Largeur de fenêtre minimale imposée au système de fenêtrage.
pub const LARGEUR_MINIMALE: f32 = 860.0;
/// Hauteur de fenêtre minimale imposée au système de fenêtrage.
pub const HAUTEUR_MINIMALE: f32 = 620.0;
/// Hauteur supposée tant que la fenêtre n'a pas signalé sa taille.
const HAUTEUR_PAR_DEFAUT: f32 = 780.0;
/// Largeur au-delà de laquelle le contenu cesse de s'étirer et gagne des marges.
pub const LARGEUR_CONTENU_MAX: f32 = 1320.0;

const RAIL_ETENDU: f32 = 1040.0;
const DEUX_COLONNES: f32 = 1180.0;
const FILTRES_SUR_UNE_LIGNE: f32 = 980.0;

/// Décisions de mise en page dérivées de la largeur de fenêtre.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MiseEnPage {
    pub largeur_fenetre: f32,
    pub hauteur_fenetre: f32,
}

impl MiseEnPage {
    pub fn depuis_largeur(largeur_fenetre: f32) -> Self {
        Self::depuis_taille(largeur_fenetre, HAUTEUR_PAR_DEFAUT)
    }

    pub fn depuis_taille(largeur_fenetre: f32, hauteur_fenetre: f32) -> Self {
        Self {
            largeur_fenetre: largeur_fenetre.max(LARGEUR_MINIMALE),
            hauteur_fenetre: hauteur_fenetre.max(HAUTEUR_MINIMALE),
        }
    }

    /// Rail replié sur ses icônes.
    pub fn rail_compact(&self) -> bool {
        self.largeur_fenetre < RAIL_ETENDU
    }

    pub fn largeur_rail(&self) -> f32 {
        if self.rail_compact() {
            60.0
        } else {
            92.0
        }
    }

    /// Tableau de bord en deux colonnes (bloc principal + colonne latérale).
    pub fn tableau_de_bord_deux_colonnes(&self) -> bool {
        self.largeur_fenetre >= DEUX_COLONNES
    }

    /// Colonne « statut » séparée dans la liste des transactions. En dessous, le
    /// statut revient dans la colonne du libellé plutôt que de disparaître.
    pub fn colonne_statut_visible(&self) -> bool {
        self.largeur_fenetre >= DEUX_COLONNES
    }

    /// Barre de filtres tenant sur une seule ligne.
    pub fn filtres_sur_une_ligne(&self) -> bool {
        self.largeur_fenetre >= FILTRES_SUR_UNE_LIGNE
    }

    /// Répartition par catégorie affichée à côté de l'anneau plutôt que dessous.
    pub fn statistiques_deux_colonnes(&self) -> bool {
        self.largeur_fenetre >= FILTRES_SUR_UNE_LIGNE
    }

    /// Largeur réellement offerte au contenu, rail déduit et plafond appliqué.
    pub fn largeur_contenu(&self) -> f32 {
        (self.largeur_fenetre - self.largeur_rail()).min(LARGEUR_CONTENU_MAX)
    }

    /// Largeur d'une modale : bornée par la fenêtre, jamais débordante.
    pub fn largeur_modale(&self, souhaitee: f32) -> f32 {
        souhaitee.min(self.largeur_fenetre - 2.0 * 48.0).max(320.0)
    }

    /// Hauteur maximale d'une modale : la fenêtre moins ses marges.
    ///
    /// Sans cela, une modale au contenu long — le formulaire calendrier
    /// déplié — dépasse le bas de la fenêtre au lieu de faire défiler son
    /// corps.
    ///
    /// La marge est exactement celle du conteneur qui centre la carte. Une
    /// valeur plus généreuse ici ferait calculer la modale sur une hauteur
    /// qu'elle n'obtient pas, et le surplus serait pris sur son pied.
    pub fn hauteur_modale(&self) -> f32 {
        (self.hauteur_fenetre - 2.0 * f32::from(Esp::XXXL)).max(320.0)
    }
}

impl Default for MiseEnPage {
    fn default() -> Self {
        Self::depuis_taille(1240.0, HAUTEUR_PAR_DEFAUT)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn le_rail_se_replie_sous_le_point_de_rupture() {
        assert!(MiseEnPage::depuis_largeur(900.0).rail_compact());
        assert!(!MiseEnPage::depuis_largeur(1240.0).rail_compact());
        assert!(
            MiseEnPage::depuis_largeur(900.0).largeur_rail()
                < MiseEnPage::depuis_largeur(1240.0).largeur_rail()
        );
    }

    #[test]
    fn les_deux_colonnes_apparaissent_sur_les_grandes_fenetres() {
        assert!(!MiseEnPage::depuis_largeur(1000.0).tableau_de_bord_deux_colonnes());
        assert!(MiseEnPage::depuis_largeur(1400.0).tableau_de_bord_deux_colonnes());
    }

    #[test]
    fn les_filtres_se_replient_sur_les_petites_fenetres() {
        assert!(!MiseEnPage::depuis_largeur(900.0).filtres_sur_une_ligne());
        assert!(MiseEnPage::depuis_largeur(1240.0).filtres_sur_une_ligne());
    }

    /// Le contenu ne doit jamais s'étirer indéfiniment sur un écran très large.
    #[test]
    fn la_largeur_de_contenu_est_plafonnee() {
        let tres_large = MiseEnPage::depuis_largeur(3840.0);
        assert_eq!(tres_large.largeur_contenu(), LARGEUR_CONTENU_MAX);
    }

    /// Une modale reste toujours contenue dans la fenêtre, y compris au
    /// minimum autorisé.
    #[test]
    fn une_modale_ne_deborde_jamais() {
        for largeur in [LARGEUR_MINIMALE, 1000.0, 1240.0, 2560.0] {
            let mise = MiseEnPage::depuis_largeur(largeur);
            assert!(mise.largeur_modale(720.0) <= largeur);
            assert!(mise.largeur_modale(720.0) >= 320.0);
        }
    }

    /// Une modale ne dépasse jamais la hauteur de la fenêtre, y compris au
    /// minimum autorisé.
    #[test]
    fn une_modale_tient_toujours_en_hauteur() {
        for hauteur in [HAUTEUR_MINIMALE, 780.0, 1440.0] {
            let mise = MiseEnPage::depuis_taille(1240.0, hauteur);
            assert!(mise.hauteur_modale() <= hauteur);
            assert!(mise.hauteur_modale() >= 320.0);
        }
    }

    /// Une hauteur aberrante est ramenée au minimum, comme la largeur.
    #[test]
    fn la_hauteur_est_bornee_par_le_minimum() {
        let minuscule = MiseEnPage::depuis_taille(1240.0, 80.0);
        assert_eq!(minuscule.hauteur_fenetre, HAUTEUR_MINIMALE);
        assert!(minuscule.hauteur_modale() > 0.0);
    }

    /// Une largeur aberrante ne doit pas produire de mise en page dégénérée.
    #[test]
    fn la_largeur_est_bornee_par_le_minimum() {
        let minuscule = MiseEnPage::depuis_largeur(120.0);
        assert_eq!(minuscule.largeur_fenetre, LARGEUR_MINIMALE);
        assert!(minuscule.largeur_contenu() > 0.0);
    }
}
