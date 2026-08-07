//! Confirmation de réinitialisation complète.
//!
//! Action la plus destructive de l'application : l'écran rappelle exactement ce
//! qui sera perdu et propose l'export avant de continuer.

use iced::widget::{column, row};
use iced::{Alignment, Element};

use crate::app::message::Message;
use crate::app::state::AppState;
use crate::ui::composants::bouton::{Bouton, Variante};
use crate::ui::composants::icone::{icone, Icone, Taille as TailleIcone};
use crate::ui::composants::modale::{Largeur, Modale};
use crate::ui::theme::espacements::Esp;
use crate::ui::theme::palette::Palette;
use crate::ui::theme::typographie::{texte_colore, Role};

/// Ce que la réinitialisation efface, énuméré explicitement.
pub const PERTES: &[&str] = &[
    "toutes les transactions, tous mois confondus",
    "le solde actuel et le découvert autorisé",
    "les catégories et leurs personnalisations",
    "les préférences d'affichage",
];

pub fn modale<'a>(state: &'a AppState, fenetre: Element<'a, Message>) -> Element<'a, Message> {
    let palette = Palette::pour(state.theme_mode);

    let mut corps = column![texte_colore(
        "Cette action supprime définitivement :",
        Role::Corps,
        palette.texte,
    )]
    .spacing(Esp::SM);

    for perte in PERTES {
        corps = corps.push(
            row![
                icone(Icone::Croix, TailleIcone::Petite, palette.danger),
                texte_colore(*perte, Role::Corps, palette.texte),
            ]
            .spacing(Esp::SM)
            .align_y(Alignment::Center),
        );
    }

    corps = corps.push(texte_colore(
        "Exporte tes données avant de continuer si tu souhaites pouvoir les retrouver.",
        Role::Legende,
        palette.texte_doux,
    ));

    Modale::nouvelle(
        "Tout effacer ?",
        corps,
        Message::CancelReset,
        palette,
        state.mise_en_page,
    )
    .sous_titre("Action irréversible")
    .largeur(Largeur::Petite)
    .action_a_gauche(
        Bouton::nouveau("Exporter d'abord", palette)
            .variante(Variante::Secondaire)
            .avec_icone(Icone::Export)
            .sur_clic(Message::ExportDatabase)
            .vue(),
    )
    .action(
        Bouton::nouveau("Annuler", palette)
            .variante(Variante::Discret)
            .sur_clic(Message::CancelReset)
            .vue(),
    )
    .action(
        Bouton::nouveau("Tout effacer", palette)
            .variante(Variante::Destructif)
            .avec_icone(Icone::Corbeille)
            .sur_clic(Message::ConfirmResetData)
            .vue(),
    )
    .poser_sur(fenetre)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// L'énumération doit rester complète et non vide : c'est elle qui rend la
    /// conséquence de l'action compréhensible.
    #[test]
    fn les_pertes_sont_enumerees() {
        assert!(PERTES.len() >= 4);
        assert!(PERTES.iter().all(|perte| !perte.is_empty()));
    }
}
