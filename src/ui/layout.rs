//! Assemblage de la fenêtre.
//!
//! Rail persistant à gauche, en-tête d'écran fixe, contenu défilant, et
//! notifications ancrées en bas à droite de la zone de contenu — jamais
//! par-dessus la navigation.

use iced::widget::{column, container, row, scrollable, stack, Space};
use iced::{Alignment, Element, Length};

use crate::app::message::{Message, Screen};
use crate::app::state::{AppState, ThemeMode};
use crate::ui::composants::banniere_maj;
use crate::ui::composants::icone::Icone;
use crate::ui::composants::navigation::{self, Entree};
use crate::ui::composants::notification;
use crate::ui::gestionnaire_ecrans;
use crate::ui::theme::espacements::Esp;
use crate::ui::theme::mise_en_page::LARGEUR_CONTENU_MAX;
use crate::ui::theme::palette::Palette;
use crate::ui::theme::styles;

pub fn fenetre(state: &AppState) -> Element<'_, Message> {
    let palette = Palette::pour(state.theme_mode);
    let mise = state.mise_en_page;

    let entrees = Screen::RAIL
        .iter()
        .map(|ecran| Entree {
            symbole: symbole_ecran(*ecran),
            intitule: ecran.display_name(),
            description: ecran.precision(),
            message: Message::NavigateTo(*ecran),
            active: state.screen == *ecran,
        })
        .collect();

    let sombre = state.theme_mode == ThemeMode::Dark;
    let rail = navigation::rail(
        entrees,
        navigation::Entree {
            symbole: if sombre { Icone::Soleil } else { Icone::Lune },
            intitule: if sombre { "Clair" } else { "Sombre" },
            description: if sombre {
                "Passer en thème clair"
            } else {
                "Passer en thème sombre"
            },
            message: Message::ToggleTheme,
            active: false,
        },
        palette,
        mise,
    );

    let (en_tete, corps) = gestionnaire_ecrans::vue_ecran_actif(state);

    let corps_defilant = scrollable(
        container(corps)
            .width(Length::Fill)
            .max_width(LARGEUR_CONTENU_MAX)
            .padding([0, Esp::XL]),
    )
    .style(move |_theme, statut| styles::defilement(palette, statut))
    .width(Length::Fill)
    .height(Length::Fill);

    let zone = column![
        en_tete,
        if let Some(banniere) = banniere_maj::banniere(state) {
            banniere
        } else {
            Space::with_height(0).into()
        },
        container(corps_defilant)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Alignment::Center),
    ]
    .width(Length::Fill)
    .height(Length::Fill);

    let page = container(row![rail, zone])
        .width(Length::Fill)
        .height(Length::Fill)
        .style(move |_theme| styles::surface_nue(palette, palette.fond, 0.0));

    match state.notification.as_ref() {
        Some(active) => {
            let toast = container(notification::notification(
                &active.texte,
                active.ton,
                Message::DismissNotification,
                palette,
            ))
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(Esp::XL)
            .align_x(Alignment::End)
            .align_y(Alignment::End);

            // La bande transparente à gauche laisse le rail cliquable.
            let ancrage = row![Space::with_width(Length::Fixed(mise.largeur_rail())), toast,]
                .height(Length::Fill);

            stack![page, ancrage].into()
        }
        None => page.into(),
    }
}

fn symbole_ecran(ecran: Screen) -> Icone {
    match ecran {
        Screen::Dashboard => Icone::Accueil,
        Screen::Transactions => Icone::Transactions,
        Screen::Statistics => Icone::Statistiques,
        Screen::Settings => Icone::Parametres,
        Screen::Onboarding => Icone::Etincelles,
    }
}
