//! Onboarding.
//!
//! Deux volets : à gauche, ce que fait le produit et pourquoi il est
//! rassurant ; à droite, les deux seules valeurs réellement nécessaires pour
//! calculer une première prévision. Ce n'est pas un formulaire administratif :
//! c'est une présentation qui se termine par une saisie.

use iced::widget::{column, container, row, Space};
use iced::{Alignment, Element, Length};

use crate::app::message::Message;
use crate::app::state::AppState;
use crate::ui::composants::bouton::{Bouton, Variante};
use crate::ui::composants::carte;
use crate::ui::composants::champ;
use crate::ui::composants::icone::{icone, Icone, Taille as TailleIcone};
use crate::ui::theme::espacements::{Esp, Rayon};
use crate::ui::theme::palette::Palette;
use crate::ui::theme::styles;
use crate::ui::theme::typographie::{texte_colore, Role};

/// Promesses du produit, présentées avant toute saisie.
pub const PROMESSES: &[(Icone, &str, &str)] = &[
    (
        Icone::TendanceHaut,
        "Une seule question, tout le temps",
        "Combien te restera-t-il à la fin du mois, une fois tout payé ?",
    ),
    (
        Icone::Sablier,
        "Le prévu compte autant que le réel",
        "Marque une dépense « en attente » : elle est déduite de ta prévision immédiatement.",
    ),
    (
        Icone::Cadenas,
        "Rien ne quitte ton ordinateur",
        "Pas de compte, pas de cloud, pas de connexion. Un simple fichier local, exportable.",
    ),
];

pub fn view(state: &AppState) -> Element<'_, Message> {
    let palette = Palette::pour(state.theme_mode);
    let deux_volets = state.mise_en_page.largeur_fenetre >= 980.0;

    let contenu: Element<'_, Message> = if deux_volets {
        row![
            container(presentation(palette)).width(Length::FillPortion(48)),
            container(saisie(state, palette)).width(Length::FillPortion(52)),
        ]
        .spacing(Esp::XXXL)
        .align_y(Alignment::Center)
        .into()
    } else {
        column![presentation(palette), saisie(state, palette)]
            .spacing(Esp::XL)
            .into()
    };

    container(
        container(contenu)
            .max_width(1040)
            .width(Length::Fill)
            .padding(Esp::XL),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .align_x(Alignment::Center)
    .align_y(Alignment::Center)
    .style(move |_theme| styles::surface_nue(palette, palette.fond, 0.0))
    .into()
}

fn presentation<'a>(palette: Palette) -> Element<'a, Message> {
    let mut bloc = column![
        row![
            container(icone(
                Icone::Portefeuille,
                TailleIcone::Grande,
                palette.accent
            ))
            .padding(Esp::MD)
            .style(move |_theme| {
                styles::surface_nue(palette, palette.accent_surface, Rayon::LG)
            }),
            column![
                texte_colore("AfterBudget", Role::TitreEcran, palette.texte_fort),
                texte_colore(
                    "Ton budget, sans mauvaise surprise",
                    Role::Corps,
                    palette.texte_doux
                ),
            ]
            .spacing(Esp::XXS),
        ]
        .spacing(Esp::MD)
        .align_y(Alignment::Center),
        Space::with_height(Length::Fixed(f32::from(Esp::SM))),
    ]
    .spacing(Esp::LG)
    .width(Length::Fill);

    for (symbole, titre, explication) in PROMESSES {
        bloc = bloc.push(
            row![
                container(icone(*symbole, TailleIcone::Normale, palette.accent))
                    .padding(Esp::SM)
                    .style(move |_theme| {
                        styles::surface_nue(palette, palette.accent_surface, Rayon::MD)
                    }),
                column![
                    texte_colore(*titre, Role::CorpsFort, palette.texte_fort),
                    texte_colore(*explication, Role::Corps, palette.texte_doux),
                ]
                .spacing(Esp::XXS),
            ]
            .spacing(Esp::MD)
            .align_y(Alignment::Start),
        );
    }

    bloc.into()
}

fn saisie<'a>(state: &'a AppState, palette: Palette) -> Element<'a, Message> {
    let mut formulaire = column![
        texte_colore(
            "Deux chiffres pour démarrer",
            Role::TitreSection,
            palette.texte_fort
        ),
        texte_colore(
            "Tu pourras les modifier à tout moment dans les paramètres.",
            Role::Legende,
            palette.texte_doux,
        ),
        Space::with_height(Length::Fixed(f32::from(Esp::SM))),
        champ::champ_montant(
            "Solde actuel de ton compte",
            &state.onboarding_balance_str,
            "€",
            Message::SetOnboardingBalance,
            Message::CompleteOnboarding,
            state.onboarding_error.as_deref(),
            palette,
        ),
        champ::champ(
            "Découvert autorisé",
            champ::saisie(
                &state.onboarding_overdraft_str,
                "Ex : 500",
                Message::SetOnboardingOverdraft,
                false,
                palette,
            ),
            Some("Mets 0 si ta banque ne t'en accorde pas."),
            None,
            palette,
        ),
    ]
    .spacing(Esp::MD)
    .width(Length::Fill);

    formulaire = formulaire.push(Space::with_height(Length::Fixed(f32::from(Esp::SM))));
    formulaire = formulaire.push(
        Bouton::nouveau("Commencer", palette)
            .variante(Variante::Principal)
            .avec_icone(Icone::ChevronDroit)
            .sur_clic(Message::CompleteOnboarding)
            .pleine_largeur()
            .vue(),
    );

    carte::carte(formulaire, carte::Variante::Heros, palette).into()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// L'onboarding doit expliquer le produit avant de demander quoi que ce
    /// soit : au moins trois promesses, toutes renseignées.
    #[test]
    fn les_promesses_sont_completes() {
        assert!(PROMESSES.len() >= 3);
        for (_, titre, explication) in PROMESSES {
            assert!(!titre.is_empty());
            assert!(!explication.is_empty());
        }
    }

    /// Chaque promesse porte une icône différente.
    #[test]
    fn chaque_promesse_a_son_icone() {
        let icones: std::collections::HashSet<_> =
            PROMESSES.iter().map(|(symbole, _, _)| symbole).collect();
        assert_eq!(icones.len(), PROMESSES.len());
    }
}
