//! En-tête de tableau.
//!
//! Donne aux listes denses des colonnes nommées, ce qui distingue un tableau
//! desktop d'une simple succession de cartes.

use iced::widget::{container, row, text, Space};
use iced::{Alignment, Element, Length};

use crate::ui::theme::espacements::Esp;
use crate::ui::theme::palette::Palette;
use crate::ui::theme::typographie::{police, Role};

/// Alignement d'une colonne.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Alignement {
    Gauche,
    Droite,
}

/// Description d'une colonne : intitulé, part de largeur, alignement.
pub struct Colonne<'a> {
    pub intitule: &'a str,
    pub portion: u16,
    pub largeur_fixe: Option<f32>,
    pub alignement: Alignement,
}

impl<'a> Colonne<'a> {
    pub fn flexible(intitule: &'a str, portion: u16) -> Self {
        Self {
            intitule,
            portion,
            largeur_fixe: None,
            alignement: Alignement::Gauche,
        }
    }

    pub fn fixe(intitule: &'a str, largeur: f32) -> Self {
        Self {
            intitule,
            portion: 0,
            largeur_fixe: Some(largeur),
            alignement: Alignement::Gauche,
        }
    }

    pub fn a_droite(mut self) -> Self {
        self.alignement = Alignement::Droite;
        self
    }

    pub fn longueur(&self) -> Length {
        match self.largeur_fixe {
            Some(largeur) => Length::Fixed(largeur),
            None => Length::FillPortion(self.portion),
        }
    }
}

/// Ligne d'en-tête d'un tableau.
pub fn entete<'a, Message: 'a>(
    colonnes: Vec<Colonne<'a>>,
    palette: Palette,
) -> Element<'a, Message> {
    let mut ligne = row![].align_y(Alignment::Center).spacing(Esp::MD);

    for colonne in colonnes {
        let longueur = colonne.longueur();
        let intitule = text(colonne.intitule.to_uppercase())
            .size(Role::Micro.taille())
            .font(police(Role::Micro.graisse()))
            .color(palette.texte_doux);

        let cellule: Element<'a, Message> = match colonne.alignement {
            Alignement::Gauche => intitule.into(),
            Alignement::Droite => row![Space::with_width(Length::Fill), intitule].into(),
        };

        ligne = ligne.push(container(cellule).width(longueur));
    }

    container(ligne)
        .padding([Esp::SM, Esp::LG])
        .width(Length::Fill)
        .into()
}
