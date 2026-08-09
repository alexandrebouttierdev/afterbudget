//! Vérification de mises à jour disponibles.
//!
//! Interroge les releases GitHub du projet, compare la version distante à la
//! version locale, et télécharge l'installeur adapté au système.

pub mod client;
pub mod plateforme;
pub mod telechargement;
pub mod versions;
