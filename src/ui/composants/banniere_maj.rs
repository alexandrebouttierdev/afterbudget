//! Bannière de mise à jour disponible.
//!
//! Posée entre l'en-tête et le contenu du shell, pleine largeur. Deux
//! actions : télécharger l'installeur (ou ouvrir la page de la release quand
//! le système n'est pas reconnu), ou ignorer — la version ignorée n'est plus
//! reproposée tant qu'elle reste la plus récente.

use iced::widget::{container, row, Space};
use iced::{Alignment, Element, Length};

use crate::app::message::Message;
use crate::app::state::AppState;
use crate::ui::composants::bouton::{Bouton, Taille as TailleBouton, Variante};
use crate::ui::composants::icone::Icone;
use crate::ui::theme::espacements::{Esp, Rayon};
use crate::ui::theme::palette::Palette;
use crate::ui::theme::styles;
use crate::ui::theme::typographie::{texte_colore, Role};

/// Bannière de mise à jour, si une version est proposée.
pub fn banniere(state: &AppState) -> Option<Element<'static, Message>> {
    let info = state.update_info.as_ref()?;
    let palette = Palette::pour(state.theme_mode);

    let message = texte_colore(
        format!("Nouvelle version {} disponible", info.version),
        Role::CorpsFort,
        palette.texte_fort,
    );

    let action = if state.update_downloading {
        Bouton::nouveau("Téléchargement…", palette)
            .variante(Variante::Secondaire)
            .taille(TailleBouton::Compacte)
            .vue()
    } else if info.asset.is_some() {
        Bouton::nouveau("Télécharger", palette)
            .variante(Variante::Principal)
            .avec_icone(Icone::Telechargement)
            .taille(TailleBouton::Compacte)
            .sur_clic(Message::DownloadUpdate)
            .vue()
    } else {
        // Système non reconnu : la page de la release sert de secours.
        Bouton::nouveau("Voir la page", palette)
            .variante(Variante::Principal)
            .avec_icone(Icone::Telechargement)
            .taille(TailleBouton::Compacte)
            .sur_clic(Message::DownloadUpdate)
            .vue()
    };

    let ignorer = Bouton::nouveau("Ignorer", palette)
        .variante(Variante::Discret)
        .taille(TailleBouton::Compacte)
        .sur_clic(Message::IgnoreUpdate)
        .vue();

    let contenu = row![message, Space::with_width(Length::Fill), action, ignorer]
        .spacing(Esp::MD)
        .align_y(Alignment::Center);

    Some(
        container(contenu)
            .width(Length::Fill)
            .padding([Esp::SM + Esp::XS, Esp::LG])
            .style(move |_theme| {
                styles::surface_semantique(
                    palette,
                    palette.accent_surface,
                    palette.accent,
                    Rayon::MD,
                )
            })
            .into(),
    )
}
