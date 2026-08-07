//! États vides, de chargement et d'erreur.
//!
//! Chaque état est composé comme un contenu à part entière : icône, titre,
//! explication, et le plus souvent une action. Jamais une phrase grise seule au
//! milieu d'un grand vide.

use std::borrow::Cow;

use iced::widget::{column, container, row, Space};
use iced::{Alignment, Element, Length};

use crate::ui::composants::icone::{icone, Icone, Taille as TailleIcone};
use crate::ui::theme::espacements::{Esp, Rayon};
use crate::ui::theme::palette::Palette;
use crate::ui::theme::styles;
use crate::ui::theme::typographie::{texte_colore, Role};

/// Contenu d'un état, quel qu'il soit.
pub struct Etat<'a, Message> {
    symbole: Icone,
    titre: Cow<'a, str>,
    explication: Cow<'a, str>,
    action: Option<Element<'a, Message>>,
    teinte: Option<iced::Color>,
    palette: Palette,
}

impl<'a, Message: 'a> Etat<'a, Message> {
    pub fn nouveau(
        symbole: Icone,
        titre: impl Into<Cow<'a, str>>,
        explication: impl Into<Cow<'a, str>>,
        palette: Palette,
    ) -> Self {
        Self {
            symbole,
            titre: titre.into(),
            explication: explication.into(),
            action: None,
            teinte: None,
            palette,
        }
    }

    pub fn avec_action(mut self, action: impl Into<Element<'a, Message>>) -> Self {
        self.action = Some(action.into());
        self
    }

    pub fn avec_teinte(mut self, teinte: iced::Color) -> Self {
        self.teinte = Some(teinte);
        self
    }

    pub fn vue(self) -> Element<'a, Message> {
        let palette = self.palette;
        let teinte = self.teinte.unwrap_or(palette.accent);

        let pastille = container(icone(self.symbole, TailleIcone::Grande, teinte))
            .padding(Esp::LG)
            .style(move |_theme| {
                styles::surface_nue(palette, palette.teinter_surface(teinte), Rayon::XL)
            });

        let mut bloc = column![
            pastille,
            Space::with_height(Length::Fixed(f32::from(Esp::SM))),
            texte_colore(self.titre, Role::TitreSection, palette.texte_fort),
            texte_colore(self.explication, Role::Corps, palette.texte_doux)
                .align_x(Alignment::Center),
        ]
        .align_x(Alignment::Center)
        .spacing(Esp::SM)
        .max_width(420);

        if let Some(action) = self.action {
            bloc = bloc.push(Space::with_height(Length::Fixed(f32::from(Esp::SM))));
            bloc = bloc.push(action);
        }

        container(bloc)
            .width(Length::Fill)
            .padding([Esp::XXXL, Esp::XL])
            .align_x(Alignment::Center)
            .into()
    }
}

/// Indicateur de chargement : quatre points dont l'intensité tourne. `phase`
/// est fournie par le tic de l'application, ce qui évite toute horloge propre
/// au composant.
pub fn chargement<'a, Message: 'a>(
    intitule: &'a str,
    phase: u8,
    palette: Palette,
) -> Element<'a, Message> {
    let mut points = row![].spacing(Esp::XS + 2).align_y(Alignment::Center);
    for index in 0..4u8 {
        let actif = index == phase % 4;
        let couleur = if actif {
            palette.accent
        } else {
            palette.bordure_forte
        };
        let cote = if actif { 8.0 } else { 6.0 };
        points = points.push(
            container(Space::new(Length::Fixed(0.0), Length::Fixed(0.0)))
                .width(Length::Fixed(cote))
                .height(Length::Fixed(cote))
                .style(move |_theme| styles::surface_nue(palette, couleur, Rayon::PLEIN)),
        );
    }

    container(
        column![
            points,
            texte_colore(intitule, Role::Legende, palette.texte_doux),
        ]
        .spacing(Esp::MD)
        .align_x(Alignment::Center),
    )
    .width(Length::Fill)
    .padding(Esp::XXL)
    .align_x(Alignment::Center)
    .into()
}

#[cfg(test)]
mod tests {
    /// La phase du chargement doit boucler sur quatre positions, quelle que
    /// soit la valeur du compteur global.
    #[test]
    fn la_phase_de_chargement_boucle() {
        let positions: Vec<u8> = (0u8..12).map(|tic| tic % 4).collect();
        assert_eq!(
            positions,
            vec![0, 1, 2, 3, 0, 1, 2, 3, 0, 1, 2, 3],
            "l'animation doit revenir à son point de départ"
        );
    }
}
