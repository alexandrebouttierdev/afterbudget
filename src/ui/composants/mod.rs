//! Composants d'interface partagés.
//!
//! Ces composants ne lisent jamais la base, n'appellent jamais un repository et
//! ne décident d'aucune règle métier : ils reçoivent une palette et les seules
//! valeurs dont ils ont besoin.

pub mod anneau;
pub mod badge;
pub mod bouton;
pub mod calendrier;
pub mod carte;
pub mod champ;
pub mod en_tete_ecran;
pub mod entete_colonnes;
pub mod etat;
pub mod icone;
pub mod infobulle;
pub mod jauge;
pub mod modale;
pub mod navigation;
pub mod notification;
pub mod selecteur_mois;
pub mod separateur;
