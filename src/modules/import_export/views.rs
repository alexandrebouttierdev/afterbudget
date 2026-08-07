//! Confirmation d'import.
//!
//! L'import remplace toutes les données existantes : l'écran nomme le fichier
//! choisi, annonce la sauvegarde automatique et distingue nettement l'action
//! sûre de l'action engageante.

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

/// Nom de fichier seul, pour ne pas noyer la modale sous un chemin absolu.
pub fn nom_de_fichier(chemin: &str) -> String {
    std::path::Path::new(chemin)
        .file_name()
        .map(|nom| nom.to_string_lossy().to_string())
        .unwrap_or_else(|| chemin.to_string())
}

pub fn modale_import<'a>(
    state: &'a AppState,
    fenetre: Element<'a, Message>,
) -> Element<'a, Message> {
    let palette = Palette::pour(state.theme_mode);
    let chemin = state.import_file_path.clone().unwrap_or_default();

    let corps = column![
        row![
            icone(Icone::Import, TailleIcone::Normale, palette.accent),
            column![
                texte_colore(nom_de_fichier(&chemin), Role::CorpsFort, palette.texte_fort),
                texte_colore(chemin.clone(), Role::Legende, palette.texte_doux),
            ]
            .spacing(Esp::XXS),
        ]
        .spacing(Esp::MD)
        .align_y(Alignment::Center),
        texte_colore(
            "Le contenu de ce fichier remplacera entièrement tes données actuelles.",
            Role::Corps,
            palette.texte,
        ),
        texte_colore(
            "Une sauvegarde de la base actuelle est créée automatiquement avant l'import.",
            Role::Legende,
            palette.texte_doux,
        ),
    ]
    .spacing(Esp::MD);

    Modale::nouvelle(
        "Importer ces données ?",
        corps,
        Message::CancelImport,
        palette,
        state.mise_en_page,
    )
    .sous_titre("Remplacement complet")
    .largeur(Largeur::Moyenne)
    .action(
        Bouton::nouveau("Annuler", palette)
            .variante(Variante::Discret)
            .sur_clic(Message::CancelImport)
            .vue(),
    )
    .action(
        Bouton::nouveau("Importer", palette)
            .variante(Variante::Principal)
            .avec_icone(Icone::Import)
            .sur_clic(Message::ConfirmImport(chemin))
            .vue(),
    )
    .poser_sur(fenetre)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn le_nom_de_fichier_est_extrait_du_chemin() {
        assert_eq!(
            nom_de_fichier("/home/alex/sauvegardes/afterbudget-2026.sqlite"),
            "afterbudget-2026.sqlite"
        );
        assert_eq!(nom_de_fichier("sauvegarde.db"), "sauvegarde.db");
    }

    /// Un chemin inhabituel ne doit pas produire une modale sans nom.
    #[test]
    fn un_chemin_sans_fichier_reste_affichable() {
        assert_eq!(nom_de_fichier(""), "");
        assert_eq!(nom_de_fichier("/"), "/");
    }
}
