//! Anneau de répartition.
//!
//! Répond à une question précise : « où part mon argent ce mois-ci ? ». Il
//! n'est affiché que lorsqu'il y a des données, et il est **toujours** doublé
//! d'une légende textuelle donnant le nom, le montant et la part de chaque
//! segment — l'information ne dépend jamais de la couleur.

use iced::mouse;
use iced::widget::canvas::{self, Path, Stroke};
use iced::widget::Canvas;
use iced::{Color, Element, Length, Radians, Rectangle, Renderer, Theme};

use crate::ui::theme::palette::Palette;

/// Part maximale de segments détaillés avant regroupement.
const SEGMENTS_MAX: usize = 6;

/// Un segment prêt à dessiner.
#[derive(Debug, Clone, PartialEq)]
pub struct Segment {
    pub intitule: String,
    pub montant_centimes: i64,
    pub part: f32,
    pub couleur: Color,
}

/// Prépare les segments : tri décroissant, regroupement de la traîne sous
/// « Autres », et recalcul des parts sur le total réel.
///
/// Fonction pure, testée : c'est une règle de présentation, pas du dessin.
pub fn preparer_segments(
    entrees: Vec<(String, i64, Color)>,
    couleur_autres: Color,
) -> Vec<Segment> {
    let mut entrees: Vec<(String, i64, Color)> =
        entrees.into_iter().filter(|(_, m, _)| *m > 0).collect();
    if entrees.is_empty() {
        return Vec::new();
    }

    entrees.sort_by_key(|(_, montant, _)| std::cmp::Reverse(*montant));

    let total: i64 = entrees.iter().map(|(_, m, _)| *m).sum();
    if total <= 0 {
        return Vec::new();
    }

    let mut segments: Vec<Segment> = Vec::new();
    for (intitule, montant, couleur) in entrees.iter().take(SEGMENTS_MAX) {
        segments.push(Segment {
            intitule: intitule.clone(),
            montant_centimes: *montant,
            part: *montant as f32 / total as f32,
            couleur: *couleur,
        });
    }

    if entrees.len() > SEGMENTS_MAX {
        let reste: i64 = entrees.iter().skip(SEGMENTS_MAX).map(|(_, m, _)| *m).sum();
        if reste > 0 {
            segments.push(Segment {
                intitule: format!("{} autres", entrees.len() - SEGMENTS_MAX),
                montant_centimes: reste,
                part: reste as f32 / total as f32,
                couleur: couleur_autres,
            });
        }
    }

    segments
}

/// Dessin de l'anneau.
struct Dessin {
    segments: Vec<Segment>,
    piste: Color,
}

impl<Message> canvas::Program<Message> for Dessin {
    type State = ();

    fn draw(
        &self,
        _etat: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bornes: Rectangle,
        _souris: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = canvas::Frame::new(renderer, bornes.size());
        let centre = frame.center();
        let cote = bornes.width.min(bornes.height);
        if cote <= 0.0 {
            return vec![frame.into_geometry()];
        }

        let epaisseur = (cote * 0.17).max(8.0);
        let rayon = (cote - epaisseur) / 2.0 - 2.0;
        if rayon <= 0.0 {
            return vec![frame.into_geometry()];
        }

        // Piste de fond : l'anneau reste lisible même avec un seul segment.
        frame.stroke(
            &Path::circle(centre, rayon),
            Stroke::default()
                .with_color(self.piste)
                .with_width(epaisseur),
        );

        // Un écart angulaire sépare les segments, ce qui les distingue sans
        // dépendre uniquement du contraste entre deux teintes voisines.
        let ecart = if self.segments.len() > 1 { 0.030 } else { 0.0 };
        let mut angle = -std::f32::consts::FRAC_PI_2;

        for segment in &self.segments {
            let balayage = segment.part * std::f32::consts::TAU;
            let utile = (balayage - ecart).max(0.02);

            let mut arc = canvas::path::Builder::new();
            arc.arc(canvas::path::Arc {
                center: centre,
                radius: rayon,
                start_angle: Radians(angle + ecart / 2.0),
                end_angle: Radians(angle + ecart / 2.0 + utile),
            });
            frame.stroke(
                &arc.build(),
                Stroke::default()
                    .with_color(segment.couleur)
                    .with_width(epaisseur)
                    .with_line_cap(canvas::LineCap::Butt),
            );

            angle += balayage;
        }

        vec![frame.into_geometry()]
    }
}

/// Anneau prêt à insérer, de côté fixe.
pub fn anneau<'a, Message: 'a>(
    segments: Vec<Segment>,
    cote: f32,
    palette: Palette,
) -> Element<'a, Message> {
    Canvas::new(Dessin {
        segments,
        piste: palette.surface_basse,
    })
    .width(Length::Fixed(cote))
    .height(Length::Fixed(cote))
    .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entree(nom: &str, montant: i64) -> (String, i64, Color) {
        (nom.to_string(), montant, Color::BLACK)
    }

    #[test]
    fn aucune_donnee_ne_produit_aucun_segment() {
        assert!(preparer_segments(Vec::new(), Color::WHITE).is_empty());
        assert!(preparer_segments(vec![entree("Vide", 0)], Color::WHITE).is_empty());
    }

    /// Les montants négatifs ou nuls ne doivent jamais entrer dans un anneau.
    #[test]
    fn les_montants_non_positifs_sont_ecartes() {
        let segments = preparer_segments(
            vec![entree("A", 100), entree("B", 0), entree("C", -50)],
            Color::WHITE,
        );
        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].intitule, "A");
    }

    #[test]
    fn les_segments_sont_tries_par_montant_decroissant() {
        let segments = preparer_segments(
            vec![
                entree("Petit", 100),
                entree("Grand", 900),
                entree("Moyen", 400),
            ],
            Color::WHITE,
        );
        let intitules: Vec<&str> = segments.iter().map(|s| s.intitule.as_str()).collect();
        assert_eq!(intitules, vec!["Grand", "Moyen", "Petit"]);
    }

    /// Au-delà du seuil, la traîne est regroupée pour que l'anneau reste
    /// lisible plutôt que découpé en une dizaine de filets.
    #[test]
    fn la_traine_est_regroupee() {
        let entrees: Vec<_> = (0..10)
            .map(|i| entree(&format!("Cat {i}"), 1000 - i as i64 * 50))
            .collect();
        let segments = preparer_segments(entrees, Color::WHITE);

        assert_eq!(segments.len(), SEGMENTS_MAX + 1);
        assert_eq!(segments.last().unwrap().intitule, "4 autres");
    }

    /// Sans traîne, aucun segment « Autres » ne doit apparaître.
    #[test]
    fn pas_de_regroupement_inutile() {
        let entrees: Vec<_> = (0..SEGMENTS_MAX)
            .map(|i| entree(&format!("Cat {i}"), 100))
            .collect();
        let segments = preparer_segments(entrees, Color::WHITE);

        assert_eq!(segments.len(), SEGMENTS_MAX);
        assert!(!segments.iter().any(|s| s.intitule.contains("autres")));
    }

    /// La somme des parts doit couvrir exactement le cercle, sinon l'anneau
    /// présente un trou ou se recouvre.
    #[test]
    fn les_parts_couvrent_tout_le_cercle() {
        let entrees: Vec<_> = (0..9)
            .map(|i| entree(&format!("Cat {i}"), 137 + i as i64 * 29))
            .collect();
        let segments = preparer_segments(entrees, Color::WHITE);

        let somme: f32 = segments.iter().map(|s| s.part).sum();
        assert!((somme - 1.0).abs() < 1e-4, "somme des parts = {somme}");
    }

    /// Le total conservé doit rester celui des données d'origine.
    #[test]
    fn le_regroupement_conserve_le_total() {
        let entrees: Vec<_> = (0..12)
            .map(|i| entree(&format!("Cat {i}"), 500 - i as i64 * 20))
            .collect();
        let total: i64 = entrees.iter().map(|(_, m, _)| *m).sum();
        let segments = preparer_segments(entrees, Color::WHITE);

        let conserve: i64 = segments.iter().map(|s| s.montant_centimes).sum();
        assert_eq!(conserve, total);
    }
}
