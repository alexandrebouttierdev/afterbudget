//! Calendrier de sélection de date.
//!
//! Iced n'en fournit pas : celui-ci est dessiné à partir de boutons. Il s'ouvre
//! sous le champ date plutôt qu'en surimpression — dans une modale, une couche
//! flottante supplémentaire serait à la fois inutile et fragile.

use chrono::{Datelike, NaiveDate};
use iced::widget::{button, column, container, row, Space};
use iced::{Alignment, Element, Length};

use crate::core::utils;
use crate::ui::composants::bouton::Bouton;
use crate::ui::composants::icone::Icone;
use crate::ui::theme::espacements::{Esp, Rayon};
use crate::ui::theme::palette::Palette;
use crate::ui::theme::styles::{self, Elevation};
use crate::ui::theme::typographie::{texte_colore, Role};

/// Initiales des jours, du lundi au dimanche.
const JOURS: [&str; 7] = ["L", "M", "M", "J", "V", "S", "D"];

/// Côté d'une case du calendrier.
const CASE: f32 = 30.0;

/// Messages émis par le calendrier.
pub struct Actions<Message> {
    pub mois_precedent: Message,
    pub mois_suivant: Message,
    /// Reçoit la date choisie.
    pub choisir: fn(NaiveDate) -> Message,
}

/// Grille du mois : six semaines de sept cases, `None` hors du mois.
///
/// Fonction pure, testée : c'est la structure du calendrier, pas son décor.
pub fn grille_du_mois(annee: i32, mois: u32) -> Vec<Vec<Option<u32>>> {
    let Some(premier) = NaiveDate::from_ymd_opt(annee, mois, 1) else {
        return Vec::new();
    };

    // La semaine commence le lundi, usage français.
    let decalage = premier.weekday().num_days_from_monday() as usize;
    let dernier = utils::last_day_of_month(annee, mois);

    let mut cases: Vec<Option<u32>> = vec![None; decalage];
    cases.extend((1..=dernier).map(Some));
    while !cases.len().is_multiple_of(7) {
        cases.push(None);
    }

    cases.chunks(7).map(|semaine| semaine.to_vec()).collect()
}

/// Calendrier complet.
pub fn calendrier<'a, Message: Clone + 'a>(
    annee: i32,
    mois: u32,
    selection: Option<NaiveDate>,
    aujourdhui: NaiveDate,
    actions: Actions<Message>,
    palette: Palette,
) -> Element<'a, Message> {
    let entete = row![
        Bouton::icone(Icone::ChevronGauche, palette)
            .sur_clic(actions.mois_precedent)
            .vue(),
        container(texte_colore(
            utils::month_year(annee, mois),
            Role::Libelle,
            palette.texte_fort,
        ))
        .width(Length::Fill)
        .align_x(Alignment::Center),
        Bouton::icone(Icone::ChevronDroit, palette)
            .sur_clic(actions.mois_suivant)
            .vue(),
    ]
    .align_y(Alignment::Center);

    let mut initiales = row![].spacing(Esp::XXS);
    for jour in JOURS {
        initiales = initiales.push(
            container(texte_colore(jour, Role::Micro, palette.texte_doux))
                .width(Length::Fixed(CASE))
                .align_x(Alignment::Center),
        );
    }

    let mut grille = column![].spacing(Esp::XXS);
    for semaine in grille_du_mois(annee, mois) {
        let mut ligne = row![].spacing(Esp::XXS);
        for jour in semaine {
            ligne = ligne.push(case(
                jour,
                annee,
                mois,
                selection,
                aujourdhui,
                actions.choisir,
                palette,
            ));
        }
        grille = grille.push(ligne);
    }

    container(
        column![entete, initiales, grille]
            .spacing(Esp::SM)
            .align_x(Alignment::Center),
    )
    .padding(Esp::MD)
    .style(move |_theme| styles::surface(palette, Elevation::Flottante, Rayon::LG))
    .into()
}

#[allow(clippy::too_many_arguments)]
fn case<'a, Message: Clone + 'a>(
    jour: Option<u32>,
    annee: i32,
    mois: u32,
    selection: Option<NaiveDate>,
    aujourdhui: NaiveDate,
    choisir: fn(NaiveDate) -> Message,
    palette: Palette,
) -> Element<'a, Message> {
    let Some(jour) = jour else {
        return Space::new(Length::Fixed(CASE), Length::Fixed(CASE)).into();
    };

    let date = NaiveDate::from_ymd_opt(annee, mois, jour).unwrap_or(aujourdhui);
    let choisi = selection == Some(date);
    let est_aujourdhui = date == aujourdhui;

    let teinte = if choisi {
        palette.accent_contraste
    } else if est_aujourdhui {
        palette.accent
    } else {
        palette.texte
    };

    button(
        container(texte_colore(jour.to_string(), Role::Legende, teinte))
            .width(Length::Fixed(CASE))
            .height(Length::Fixed(CASE))
            .align_x(Alignment::Center)
            .align_y(Alignment::Center),
    )
    .padding(0)
    .on_press(choisir(date))
    .style(move |_theme, statut| {
        let mut style = if choisi {
            styles::bouton_plein(palette, palette.accent, palette.accent_contraste, statut)
        } else {
            styles::bouton_fantome(palette, teinte, statut)
        };
        // Le jour courant garde un contour, même non sélectionné : c'est le
        // repère qui permet de se situer sans lire l'en-tête.
        if est_aujourdhui && !choisi {
            style.border = iced::Border {
                color: palette.accent,
                width: 1.0,
                radius: Rayon::SM.into(),
            };
        }
        style
    })
    .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Le 1er août 2026 est un samedi : la première semaine porte cinq cases
    /// vides avant lui.
    #[test]
    fn la_grille_decale_le_premier_jour_sur_le_bon_jour_de_semaine() {
        let grille = grille_du_mois(2026, 8);
        assert_eq!(grille[0][..5], [None, None, None, None, None]);
        assert_eq!(grille[0][5], Some(1));
        assert_eq!(grille[0][6], Some(2));
    }

    /// La semaine commence le lundi, pas le dimanche.
    #[test]
    fn la_semaine_commence_le_lundi() {
        assert_eq!(JOURS[0], "L");
        assert_eq!(JOURS[6], "D");

        // Le 1er juin 2026 est un lundi : aucune case vide en tête.
        let grille = grille_du_mois(2026, 6);
        assert_eq!(grille[0][0], Some(1));
    }

    /// Toutes les semaines sont complètes, et tous les jours du mois présents
    /// exactement une fois.
    #[test]
    fn la_grille_couvre_le_mois_sans_trou_ni_doublon() {
        for (annee, mois, attendus) in [
            (2026, 1, 31),
            (2026, 2, 28),
            (2024, 2, 29),
            (2026, 4, 30),
            (2026, 12, 31),
        ] {
            let grille = grille_du_mois(annee, mois);
            assert!(grille.iter().all(|semaine| semaine.len() == 7));

            let mut jours: Vec<u32> = grille.iter().flatten().flatten().copied().collect();
            jours.sort_unstable();
            assert_eq!(jours, (1..=attendus).collect::<Vec<u32>>());
        }
    }

    /// Un mois invalide ne doit pas faire paniquer le rendu.
    #[test]
    fn un_mois_invalide_produit_une_grille_vide() {
        assert!(grille_du_mois(2026, 0).is_empty());
        assert!(grille_du_mois(2026, 13).is_empty());
    }
}
