//! Modales.
//!
//! Une modale est un voile plus une carte dimensionnée. Le voile est cliquable
//! pour fermer, la carte ne dépasse jamais la fenêtre, l'action sûre est à
//! gauche et l'action engageante à droite.

use std::borrow::Cow;

use iced::widget::{column, container, mouse_area, opaque, row, scrollable, stack, Space};
use iced::{Alignment, Element, Length};

use crate::ui::composants::bouton::{Bouton, Taille as TailleBouton};
use crate::ui::composants::icone::Icone;
use crate::ui::composants::separateur;
use crate::ui::theme::espacements::{Esp, Rayon};
use crate::ui::theme::mise_en_page::MiseEnPage;
use crate::ui::theme::palette::Palette;
use crate::ui::theme::styles::{self, Elevation};
use crate::ui::theme::typographie::{texte_colore, Role};

/// Largeurs souhaitées, avant bornage par la fenêtre.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Largeur {
    /// Confirmation courte.
    Petite,
    /// Formulaire courant.
    Moyenne,
    /// Formulaire riche.
    Grande,
}

impl Largeur {
    fn souhaitee(self) -> f32 {
        match self {
            Self::Petite => 460.0,
            Self::Moyenne => 560.0,
            Self::Grande => 720.0,
        }
    }
}

/// Hauteur que l'entête et le pied occupent au plus, filets compris.
///
/// L'entête est un titre et son sous-titre, le pied une rangée de boutons :
/// deux blocs d'une ligne, mesurés puis arrondis vers le haut.
const ENTETE_ET_PIED: f32 = 96.0;

/// Hauteur soustraite à la modale pour obtenir la place offerte au corps.
///
/// Le flex d'iced sert les enfants d'une colonne dans l'ordre, en décrémentant
/// la hauteur restante : un corps en `Shrink` prend tout ce qui reste et le
/// pied se retrouve à zéro — les actions disparaissent. Borner le corps est ce
/// qui garantit qu'elles restent visibles. La réserve penche volontairement du
/// côté large : trop haute, la modale est un peu plus courte que nécessaire ;
/// trop basse, le pied est de nouveau écrasé.
const CHROME_VERTICAL: f32 = 2.0 * Esp::XL as f32   // padding de la carte
    + 4.0 * Esp::LG as f32                          // écarts entre les cinq blocs
    + 2.0                                           // les deux filets
    + ENTETE_ET_PIED;

/// Hauteur minimale laissée au corps, même dans une fenêtre très basse.
const CORPS_MINIMUM: f32 = 120.0;

/// Place offerte au corps défilant, chrome de la modale déduit.
fn hauteur_corps(mise: MiseEnPage) -> f32 {
    (mise.hauteur_modale() - CHROME_VERTICAL).max(CORPS_MINIMUM)
}

/// Modale en cours de construction.
pub struct Modale<'a, Message> {
    titre: Cow<'a, str>,
    sous_titre: Option<Cow<'a, str>>,
    corps: Element<'a, Message>,
    actions: Vec<Element<'a, Message>>,
    action_secondaire: Option<Element<'a, Message>>,
    largeur: Largeur,
    fermeture: Message,
    palette: Palette,
    mise: MiseEnPage,
}

impl<'a, Message: Clone + 'a> Modale<'a, Message> {
    pub fn nouvelle(
        titre: impl Into<Cow<'a, str>>,
        corps: impl Into<Element<'a, Message>>,
        fermeture: Message,
        palette: Palette,
        mise: MiseEnPage,
    ) -> Self {
        Self {
            titre: titre.into(),
            sous_titre: None,
            corps: corps.into(),
            actions: Vec::new(),
            action_secondaire: None,
            largeur: Largeur::Moyenne,
            fermeture,
            palette,
            mise,
        }
    }

    pub fn sous_titre(mut self, sous_titre: impl Into<Cow<'a, str>>) -> Self {
        self.sous_titre = Some(sous_titre.into());
        self
    }

    pub fn largeur(mut self, largeur: Largeur) -> Self {
        self.largeur = largeur;
        self
    }

    /// Action alignée à gauche du pied, séparée des actions principales : sert
    /// aux actions destructives d'un formulaire d'édition.
    pub fn action_a_gauche(mut self, action: impl Into<Element<'a, Message>>) -> Self {
        self.action_secondaire = Some(action.into());
        self
    }

    /// Actions du pied, dans l'ordre de lecture : la dernière est la principale.
    pub fn action(mut self, action: impl Into<Element<'a, Message>>) -> Self {
        self.actions.push(action.into());
        self
    }

    /// Superpose la modale au contenu de la fenêtre.
    pub fn poser_sur(self, contenu: impl Into<Element<'a, Message>>) -> Element<'a, Message> {
        let palette = self.palette;
        let fermeture = self.fermeture.clone();
        let largeur = self.mise.largeur_modale(self.largeur.souhaitee());
        let hauteur_corps = hauteur_corps(self.mise);

        let voile = mouse_area(
            container(Space::new(Length::Fill, Length::Fill))
                .width(Length::Fill)
                .height(Length::Fill)
                .style(move |_theme| styles::surface_nue(palette, palette.voile, 0.0)),
        )
        .on_press(fermeture.clone());

        let entete = row![
            column![
                texte_colore(self.titre, Role::TitreSection, palette.texte_fort),
                match self.sous_titre {
                    Some(sous_titre) =>
                        Element::from(texte_colore(sous_titre, Role::Legende, palette.texte_doux)),
                    None => Space::new(0, 0).into(),
                },
            ]
            .spacing(Esp::XXS)
            .width(Length::Fill),
            Bouton::icone(Icone::Croix, palette)
                .taille(TailleBouton::Icone)
                .sur_clic(fermeture)
                .vue(),
        ]
        .align_y(Alignment::Start)
        .spacing(Esp::MD);

        let mut pied = row![].spacing(Esp::SM).align_y(Alignment::Center);
        if let Some(secondaire) = self.action_secondaire {
            pied = pied.push(secondaire);
        }
        pied = pied.push(Space::with_width(Length::Fill));
        for action in self.actions {
            pied = pied.push(action);
        }

        let carte = container(
            column![
                entete,
                separateur::horizontal(palette),
                container(
                    scrollable(container(self.corps).padding([Esp::XS, 0]))
                        .style(move |_theme, statut| styles::defilement(palette, statut))
                        .height(Length::Shrink),
                )
                .max_height(hauteur_corps),
                separateur::horizontal(palette),
                pied,
            ]
            .spacing(Esp::LG),
        )
        .padding(Esp::XL)
        .width(Length::Fixed(largeur))
        .max_height(self.mise.hauteur_modale())
        .style(move |_theme| styles::surface(palette, Elevation::Modale, Rayon::XL));

        let centre = container(carte)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(Esp::XXXL)
            .align_x(Alignment::Center)
            .align_y(Alignment::Center);

        stack![contenu.into(), opaque(voile), opaque(centre)].into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::theme::mise_en_page::{HAUTEUR_MINIMALE, LARGEUR_MINIMALE};

    /// Le corps ne doit jamais réclamer la place de l'entête et du pied : c'est
    /// ce qui écrasait les actions du formulaire à zéro pixel de haut.
    #[test]
    fn le_corps_laisse_toujours_la_place_a_lentete_et_au_pied() {
        for hauteur in [HAUTEUR_MINIMALE, 680.0, 780.0, 1080.0, 2160.0] {
            let mise = MiseEnPage::depuis_taille(LARGEUR_MINIMALE, hauteur);
            assert!(
                hauteur_corps(mise) + ENTETE_ET_PIED <= mise.hauteur_modale(),
                "à {hauteur} px de fenêtre, le corps déborde sur le pied"
            );
        }
    }

    /// Même dans la fenêtre la plus basse autorisée, le corps reste lisible.
    #[test]
    fn le_corps_garde_une_hauteur_utilisable() {
        let minimale = MiseEnPage::depuis_taille(LARGEUR_MINIMALE, HAUTEUR_MINIMALE);
        assert!(hauteur_corps(minimale) >= CORPS_MINIMUM);
    }

    /// Une fenêtre plus haute doit profiter au corps, pas au chrome.
    #[test]
    fn le_corps_grandit_avec_la_fenetre() {
        let basse = MiseEnPage::depuis_taille(LARGEUR_MINIMALE, 700.0);
        let haute = MiseEnPage::depuis_taille(LARGEUR_MINIMALE, 1200.0);
        assert!(hauteur_corps(haute) > hauteur_corps(basse));
    }
}
