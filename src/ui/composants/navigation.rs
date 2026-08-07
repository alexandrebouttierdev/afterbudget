//! Rail de navigation.
//!
//! Motif retenu : **rail vertical à tuiles carrées**, icône au-dessus du
//! libellé — le patron des barres d'outils de logiciels de création (Affinity,
//! DaVinci) et du *navigation rail* de Material 3, plutôt que la liste de liens
//! des barres latérales web.
//!
//! L'état actif n'est pas un aplat pleine tuile : seule l'**icône** est posée
//! dans une pastille carrée cuivrée pleine, et le libellé passe en cuivre. La
//! masse colorée reste ainsi minuscule, ce qui garde le rail calme.
//!
//! Sur fenêtre étroite, la tuile reste carrée mais perd son libellé, rendu en
//! infobulle : l'identité carrée est conservée à toutes les tailles.

use iced::widget::{button, column, container, Space};
use iced::{Alignment, Border, Element, Length};

use crate::ui::composants::icone::{icone, Icone, Taille as TailleIcone};
use crate::ui::composants::infobulle::{infobulle, Position};
use crate::ui::composants::separateur;
use crate::ui::theme::espacements::{Esp, Rayon};
use crate::ui::theme::mise_en_page::MiseEnPage;
use crate::ui::theme::palette::Palette;
use crate::ui::theme::styles;
use crate::ui::theme::typographie::{texte_colore, Role};

/// Côté d'une tuile, libellé compris.
const TUILE_ETENDUE: f32 = 76.0;
/// Côté d'une tuile réduite à son icône.
const TUILE_COMPACTE: f32 = 52.0;
/// Côté de la pastille portant l'icône.
const PASTILLE: f32 = 32.0;

/// Une entrée du rail.
pub struct Entree<Message> {
    pub symbole: Icone,
    pub intitule: &'static str,
    /// Intitulé long, affiché en infobulle quand le libellé est masqué ou
    /// lorsqu'il abrège l'action réelle.
    pub description: &'static str,
    pub message: Message,
    pub active: bool,
}

/// Construit le rail complet.
pub fn rail<'a, Message: Clone + 'a>(
    entrees: Vec<Entree<Message>>,
    bascule_theme: Entree<Message>,
    palette: Palette,
    mise: MiseEnPage,
) -> Element<'a, Message> {
    let compact = mise.rail_compact();

    let mut tuiles = column![]
        .spacing(Esp::XS)
        .width(Length::Fill)
        .align_x(Alignment::Center);
    for entree in entrees {
        tuiles = tuiles.push(tuile(entree, compact, palette));
    }

    let contenu = column![
        marque(compact, palette),
        separateur::horizontal(palette),
        container(tuiles).padding([Esp::SM, Esp::XS]),
        Space::with_height(Length::Fill),
        separateur::horizontal(palette),
        container(tuile(bascule_theme, compact, palette)).padding([Esp::SM, Esp::XS]),
    ]
    .width(Length::Fill)
    .align_x(Alignment::Center);

    container(contenu)
        .width(Length::Fixed(mise.largeur_rail()))
        .height(Length::Fill)
        .style(move |_theme| container::Style {
            background: Some(iced::Background::Color(palette.fond_rail)),
            // Bord franc à droite : c'est ce qui fait lire le rail comme un
            // panneau d'application, et non comme un aplat de couleur.
            border: Border {
                color: palette.bordure,
                width: 1.0,
                radius: 0.0.into(),
            },
            ..container::Style::default()
        })
        .into()
}

/// En-tête du rail : la marque reprend le motif des tuiles, en plus discret.
fn marque<'a, Message: 'a>(compact: bool, palette: Palette) -> Element<'a, Message> {
    let mut bloc = column![icone(
        Icone::Portefeuille,
        TailleIcone::Normale,
        palette.accent
    )]
    .align_x(Alignment::Center)
    .spacing(Esp::XS);

    if !compact {
        bloc = bloc.push(texte_colore("AfterBudget", Role::Micro, palette.texte_fort));
    }

    container(bloc)
        .width(Length::Fill)
        .padding([Esp::LG, Esp::XS])
        .align_x(Alignment::Center)
        .into()
}

/// Fond, contour et teinte d'icône de la pastille, selon l'état.
///
/// La pastille inactive utilise `surface` et non `surface_basse` : sur le fond
/// du rail, `surface_basse` est indiscernable en thème clair et **strictement
/// identique** en thème sombre, ce qui ferait disparaître la forme carrée dès
/// qu'une entrée n'est pas active.
fn apparence_pastille(active: bool, palette: Palette) -> (iced::Color, iced::Color, iced::Color) {
    if active {
        (palette.accent, palette.accent, palette.accent_contraste)
    } else {
        (palette.surface, palette.bordure, palette.texte)
    }
}

/// Une tuile carrée : pastille d'icône au-dessus, libellé dessous.
fn tuile<'a, Message: Clone + 'a>(
    entree: Entree<Message>,
    compact: bool,
    palette: Palette,
) -> Element<'a, Message> {
    let active = entree.active;
    let cote = if compact {
        TUILE_COMPACTE
    } else {
        TUILE_ETENDUE
    };

    let (fond_pastille, contour, teinte_icone) = apparence_pastille(active, palette);

    let pastille = container(icone(entree.symbole, TailleIcone::Normale, teinte_icone))
        .width(Length::Fixed(PASTILLE))
        .height(Length::Fixed(PASTILLE))
        .align_x(Alignment::Center)
        .align_y(Alignment::Center)
        .style(move |_theme| container::Style {
            background: Some(iced::Background::Color(fond_pastille)),
            border: Border {
                color: contour,
                width: 1.0,
                radius: Rayon::MD.into(),
            },
            ..container::Style::default()
        });

    let mut contenu = column![pastille]
        .align_x(Alignment::Center)
        .spacing(Esp::XS + 2);

    if !compact {
        contenu = contenu.push(
            texte_colore(
                entree.intitule,
                Role::Micro,
                if active {
                    palette.accent
                } else {
                    palette.texte
                },
            )
            // Le libellé ne doit jamais faire grandir la tuile : il est coupé
            // plutôt que replié, et l'infobulle donne l'intitulé complet.
            .wrapping(iced::widget::text::Wrapping::None),
        );
    }

    let bouton = button(
        container(contenu)
            .width(Length::Fixed(cote))
            .height(Length::Fixed(cote))
            .align_x(Alignment::Center)
            .align_y(Alignment::Center),
    )
    .on_press(entree.message)
    .padding(0)
    .style(move |_theme, statut| styles::bouton_navigation(palette, active, statut));

    infobulle(bouton, entree.description, Position::Right, palette)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::theme::palette::{contraste, ThemeMode};

    fn palettes() -> [Palette; 2] {
        [
            Palette::pour(ThemeMode::Light),
            Palette::pour(ThemeMode::Dark),
        ]
    }

    /// L'entrée active se distingue par **deux** signaux indépendants : le
    /// remplissage de la pastille et la teinte de son icône. La couleur seule
    /// ne porte jamais l'information.
    #[test]
    fn lentree_active_se_distingue_doublement() {
        for p in palettes() {
            let (fond_actif, _, icone_active) = apparence_pastille(true, p);
            let (fond_inactif, _, icone_inactive) = apparence_pastille(false, p);

            assert_ne!(fond_actif, fond_inactif);
            assert_ne!(icone_active, icone_inactive);
            assert_eq!(fond_actif, p.accent);
        }
    }

    /// La pastille inactive doit rester visible sur le fond du rail, sinon la
    /// forme carrée disparaît pour toutes les entrées sauf une.
    #[test]
    fn la_pastille_inactive_reste_visible_sur_le_rail() {
        for p in palettes() {
            let (fond, contour, _) = apparence_pastille(false, p);
            assert_ne!(fond, p.fond_rail, "pastille inactive noyée dans le rail");
            assert!(
                contraste(fond, p.fond_rail) > 1.05 || contraste(contour, p.fond_rail) > 1.2,
                "pastille inactive indiscernable du rail"
            );
        }
    }

    /// L'icône reste lisible dans sa pastille, active comme inactive.
    #[test]
    fn licone_reste_lisible_dans_sa_pastille() {
        for p in palettes() {
            for active in [true, false] {
                let (fond, _, teinte) = apparence_pastille(active, p);
                let ratio = contraste(teinte, fond);
                assert!(ratio >= 4.5, "contraste {ratio:.2} (active = {active})");
            }
        }
    }

    /// Cohérence des côtés, vérifiée à la compilation : une future retouche des
    /// constantes échoue immédiatement plutôt qu'au moment des tests.
    const _COTES_COHERENTS: () = {
        assert!(TUILE_COMPACTE >= 32.0, "cible cliquable minimale");
        assert!(TUILE_COMPACTE < TUILE_ETENDUE, "ordre des tailles");
        assert!(
            PASTILLE < TUILE_COMPACTE,
            "pastille plus grande que la tuile"
        );
    };

    /// Le rail doit rester assez large pour accueillir la tuile et ses marges,
    /// sinon le libellé serait rogné.
    #[test]
    fn le_rail_accueille_la_tuile() {
        use crate::ui::theme::mise_en_page::MiseEnPage;

        let etendu = MiseEnPage::depuis_largeur(1240.0);
        assert!(!etendu.rail_compact());
        assert!(etendu.largeur_rail() >= TUILE_ETENDUE + 2.0 * f32::from(Esp::XS));

        let compact = MiseEnPage::depuis_largeur(900.0);
        assert!(compact.rail_compact());
        assert!(compact.largeur_rail() >= TUILE_COMPACTE + 2.0 * f32::from(Esp::XS));
    }
}
