//! Notifications éphémères.
//!
//! Ancrées en bas à droite de la zone de contenu, jamais par-dessus la
//! navigation, et fermées automatiquement au bout de quelques secondes.

use iced::widget::{column, container, row, stack, Space};
use iced::{Alignment, Element, Length};

use crate::ui::composants::badge::Ton;
use crate::ui::composants::bouton::{Bouton, Taille as TailleBouton};
use crate::ui::composants::icone::{icone, Icone, Taille as TailleIcone};
use crate::ui::theme::espacements::{Esp, Rayon};
use crate::ui::theme::palette::Palette;
use crate::ui::theme::styles::{self, Elevation};
use crate::ui::theme::typographie::{texte_colore, Role};

/// Durée d'affichage, exprimée en tics d'une seconde.
pub const DUREE_TICS: u8 = 5;

/// Épaisseur du liséré de ton, à gauche du toast.
const LARGEUR_LISERE: f32 = 3.0;

impl Ton {
    /// Icône associée au ton, pour que le sens ne repose pas sur la couleur.
    pub fn symbole(self) -> Icone {
        match self {
            Self::Succes => Icone::Succes,
            Self::Attention => Icone::Alerte,
            Self::Danger => Icone::Danger,
            Self::Neutre | Self::Accent => Icone::Info,
        }
    }
}

/// Toast complet.
pub fn notification<'a, Message: Clone + 'a>(
    intitule: &'a str,
    ton: Ton,
    fermeture: Message,
    palette: Palette,
) -> Element<'a, Message> {
    let teinte = ton.teinte(palette);

    let corps = column![texte_colore(intitule, Role::Corps, palette.texte_fort)]
        .spacing(Esp::XXS)
        .max_width(320);

    // Le liséré réserve sa place ici, mais n'est pas peint dans la rangée : un
    // enfant `Fill` en axe transverse prend toute la hauteur *offerte*, pas
    // celle mesurée sur ses voisins, et le toast s'étirait alors sur la fenêtre
    // entière.
    let contenu = row![
        Space::with_width(Length::Fixed(LARGEUR_LISERE)),
        icone(ton.symbole(), TailleIcone::Normale, teinte),
        corps,
        Space::with_width(Length::Fixed(f32::from(Esp::SM))),
        Bouton::icone(Icone::Croix, palette)
            .taille(TailleBouton::Icone)
            .sur_clic(fermeture)
            .vue(),
    ]
    .spacing(Esp::MD)
    .align_y(Alignment::Center);

    // Posé en surimpression, le liséré hérite de la hauteur que la rangée vient
    // de fixer : il la couvre entièrement sans jamais la dicter.
    let lisere = container(
        container(Space::new(Length::Fixed(LARGEUR_LISERE), Length::Fill))
            .width(Length::Fixed(LARGEUR_LISERE))
            .height(Length::Fill)
            .style(move |_theme| styles::surface_nue(palette, teinte, Rayon::PLEIN)),
    )
    .padding([Esp::MD, Esp::MD])
    .align_x(Alignment::Start);

    container(stack![
        container(contenu).padding([Esp::MD, Esp::MD]),
        lisere,
    ])
    .style(move |_theme| styles::surface(palette, Elevation::Flottante, Rayon::LG))
    .into()
}
