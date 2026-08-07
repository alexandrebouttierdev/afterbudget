//! Styles thémés des widgets Iced.
//!
//! Toutes les fermetures de style de l'application passent par ici, et toutes
//! exploitent le paramètre `Status` : survol, focus, pression et désactivation
//! ne sont jamais ignorés.

use iced::widget::{button, container, pick_list, scrollable, text_input};
use iced::{Background, Border, Color};

use super::espacements::{Ombre, Rayon};
use super::palette::{avec_alpha, melange, Palette};

/// Niveau d'élévation d'une surface.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Elevation {
    /// Encoche : plus basse que la surface qui la porte.
    Basse,
    /// Surface courante d'une carte.
    Plate,
    /// Surface détachée : menu, infobulle, notification.
    Flottante,
    /// Modale.
    Modale,
}

impl Elevation {
    fn fond(self, palette: Palette) -> Color {
        match self {
            Self::Basse => palette.surface_basse,
            Self::Plate => palette.surface,
            Self::Flottante | Self::Modale => palette.surface_haute,
        }
    }

    fn ombre(self, palette: Palette) -> iced::Shadow {
        match self {
            Self::Basse => iced::Shadow::default(),
            Self::Plate => Ombre::carte(palette),
            Self::Flottante => Ombre::flottant(palette),
            Self::Modale => Ombre::modale(palette),
        }
    }
}

/// Surface neutre : le socle de toutes les cartes et panneaux.
pub fn surface(palette: Palette, elevation: Elevation, rayon: f32) -> container::Style {
    container::Style {
        background: Some(Background::Color(elevation.fond(palette))),
        border: Border {
            color: palette.bordure,
            width: 1.0,
            radius: rayon.into(),
        },
        shadow: elevation.ombre(palette),
        text_color: Some(palette.texte),
    }
}

/// Surface sans contour, pour les zones qui n'ont pas à être cadrées.
pub fn surface_nue(_palette: Palette, couleur: Color, rayon: f32) -> container::Style {
    container::Style {
        background: Some(Background::Color(couleur)),
        border: Border {
            radius: rayon.into(),
            ..Border::default()
        },
        ..container::Style::default()
    }
}

/// Surface teintée par une couleur sémantique, avec un liseré de la même
/// famille : sert aux bandeaux d'alerte et aux badges.
pub fn surface_semantique(
    _palette: Palette,
    fond: Color,
    contour: Color,
    rayon: f32,
) -> container::Style {
    container::Style {
        background: Some(Background::Color(fond)),
        border: Border {
            color: avec_alpha(contour, 0.35),
            width: 1.0,
            radius: rayon.into(),
        },
        ..container::Style::default()
    }
}

// ── Boutons ────────────────────────────────────────────────────────────────

/// Bouton plein : fond coloré, texte contrasté.
pub fn bouton_plein(
    palette: Palette,
    fond: Color,
    texte: Color,
    statut: button::Status,
) -> button::Style {
    let (fond, contour) = match statut {
        button::Status::Active => (fond, Color::TRANSPARENT),
        button::Status::Hovered => (palette.accentuer(fond, 0.10), Color::TRANSPARENT),
        button::Status::Pressed => (palette.accentuer(fond, 0.20), Color::TRANSPARENT),
        button::Status::Disabled => (
            melange(palette.surface_basse, fond, 0.15),
            Color::TRANSPARENT,
        ),
    };
    let texte = if matches!(statut, button::Status::Disabled) {
        palette.texte_doux
    } else {
        texte
    };

    button::Style {
        background: Some(Background::Color(fond)),
        text_color: texte,
        border: Border {
            color: contour,
            width: 0.0,
            radius: Rayon::MD.into(),
        },
        shadow: iced::Shadow::default(),
    }
}

/// Bouton secondaire : surface neutre bordée.
pub fn bouton_contour(palette: Palette, texte: Color, statut: button::Status) -> button::Style {
    let (fond, contour) = match statut {
        button::Status::Active => (palette.surface, palette.bordure),
        button::Status::Hovered => (palette.surface_haute, palette.bordure_forte),
        button::Status::Pressed => (palette.surface_basse, palette.bordure_forte),
        button::Status::Disabled => (palette.surface_basse, palette.bordure),
    };
    let texte = if matches!(statut, button::Status::Disabled) {
        palette.texte_doux
    } else {
        texte
    };

    button::Style {
        background: Some(Background::Color(fond)),
        text_color: texte,
        border: Border {
            color: contour,
            width: 1.0,
            radius: Rayon::MD.into(),
        },
        shadow: iced::Shadow::default(),
    }
}

/// Bouton discret : rien au repos, une surface au survol.
pub fn bouton_fantome(palette: Palette, texte: Color, statut: button::Status) -> button::Style {
    let fond = match statut {
        button::Status::Active | button::Status::Disabled => Color::TRANSPARENT,
        button::Status::Hovered => palette.surface_basse,
        button::Status::Pressed => palette.bordure,
    };
    let texte = match statut {
        button::Status::Disabled => palette.texte_doux,
        button::Status::Hovered | button::Status::Pressed => palette.accentuer(texte, 0.12),
        button::Status::Active => texte,
    };

    button::Style {
        background: Some(Background::Color(fond)),
        text_color: texte,
        border: Border {
            radius: Rayon::SM.into(),
            ..Border::default()
        },
        shadow: iced::Shadow::default(),
    }
}

/// Bouton servant de ligne cliquable dans une liste : aucun cadre, seulement un
/// changement de surface au survol.
pub fn bouton_ligne(palette: Palette, selectionnee: bool, statut: button::Status) -> button::Style {
    let fond = match statut {
        _ if selectionnee => palette.accent_surface,
        button::Status::Hovered => palette.surface_haute,
        button::Status::Pressed => palette.surface_basse,
        _ => Color::TRANSPARENT,
    };

    button::Style {
        background: Some(Background::Color(fond)),
        text_color: palette.texte,
        border: Border {
            radius: Rayon::SM.into(),
            ..Border::default()
        },
        shadow: iced::Shadow::default(),
    }
}

/// Tuile du rail de navigation.
///
/// L'état actif est porté par la pastille d'icône et par la couleur du
/// libellé ; la tuile elle-même reste discrète et ne sert qu'au retour de
/// survol. Iced ne propageant pas le survol d'un conteneur à ses enfants,
/// c'est le seul niveau où ce retour peut être rendu.
pub fn bouton_navigation(palette: Palette, actif: bool, statut: button::Status) -> button::Style {
    let fond = match statut {
        button::Status::Hovered => avec_alpha(palette.surface, 0.75),
        button::Status::Pressed => palette.surface,
        _ => Color::TRANSPARENT,
    };

    button::Style {
        background: Some(Background::Color(fond)),
        text_color: if actif { palette.accent } else { palette.texte },
        border: Border {
            radius: Rayon::MD.into(),
            ..Border::default()
        },
        shadow: iced::Shadow::default(),
    }
}

// ── Contrôles de saisie ────────────────────────────────────────────────────

/// Champ de texte. `en_erreur` bascule le contour en `danger`, indépendamment
/// du focus, afin que l'erreur reste visible pendant la correction.
pub fn champ(palette: Palette, statut: text_input::Status, en_erreur: bool) -> text_input::Style {
    let concentre = matches!(statut, text_input::Status::Focused);
    let survole = matches!(statut, text_input::Status::Hovered);
    let desactive = matches!(statut, text_input::Status::Disabled);

    let contour = if en_erreur {
        palette.danger
    } else if concentre {
        palette.accent
    } else if survole {
        palette.bordure_forte
    } else {
        palette.bordure
    };

    text_input::Style {
        background: Background::Color(palette.surface_basse),
        border: Border {
            color: contour,
            width: if concentre || en_erreur { 2.0 } else { 1.0 },
            radius: Rayon::MD.into(),
        },
        icon: palette.texte_doux,
        placeholder: palette.texte_doux,
        value: if desactive {
            palette.texte_doux
        } else {
            palette.texte_fort
        },
        selection: avec_alpha(palette.accent, 0.35),
    }
}

/// Sélecteur déroulant, aligné visuellement sur le champ de texte.
pub fn selecteur(palette: Palette, statut: pick_list::Status) -> pick_list::Style {
    let contour = match statut {
        pick_list::Status::Opened => palette.accent,
        pick_list::Status::Hovered => palette.bordure_forte,
        _ => palette.bordure,
    };

    pick_list::Style {
        text_color: palette.texte_fort,
        background: Background::Color(palette.surface_basse),
        placeholder_color: palette.texte_doux,
        handle_color: palette.texte_doux,
        border: Border {
            color: contour,
            width: if matches!(statut, pick_list::Status::Opened) {
                2.0
            } else {
                1.0
            },
            radius: Rayon::MD.into(),
        },
    }
}

/// Liste déroulante d'un sélecteur.
pub fn menu_selecteur(palette: Palette) -> iced::overlay::menu::Style {
    iced::overlay::menu::Style {
        background: Background::Color(palette.surface_haute),
        border: Border {
            color: palette.bordure_forte,
            width: 1.0,
            radius: Rayon::MD.into(),
        },
        text_color: palette.texte_fort,
        selected_background: Background::Color(palette.accent_surface),
        selected_text_color: palette.accent,
    }
}

/// Barre de défilement : discrète au repos, visible au survol.
pub fn defilement(palette: Palette, statut: scrollable::Status) -> scrollable::Style {
    let survole = matches!(
        statut,
        scrollable::Status::Hovered { .. } | scrollable::Status::Dragged { .. }
    );

    let rail = scrollable::Rail {
        background: Some(Background::Color(Color::TRANSPARENT)),
        border: Border::default(),
        scroller: scrollable::Scroller {
            color: if survole {
                palette.bordure_forte
            } else {
                avec_alpha(palette.bordure_forte, 0.55)
            },
            border: Border {
                radius: Rayon::PLEIN.into(),
                ..Border::default()
            },
        },
    };

    scrollable::Style {
        container: container::Style::default(),
        vertical_rail: rail,
        horizontal_rail: rail,
        gap: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::theme::palette::ThemeMode;

    fn palettes() -> [Palette; 2] {
        [
            Palette::pour(ThemeMode::Light),
            Palette::pour(ThemeMode::Dark),
        ]
    }

    /// Le survol doit toujours produire un fond différent du repos : c'est la
    /// seule micro-interaction dont Iced nous garantit le déclenchement.
    #[test]
    fn le_survol_modifie_toujours_le_fond() {
        for p in palettes() {
            let repos = bouton_plein(p, p.accent, p.accent_contraste, button::Status::Active);
            let survol = bouton_plein(p, p.accent, p.accent_contraste, button::Status::Hovered);
            assert_ne!(repos.background, survol.background);

            let repos = bouton_contour(p, p.texte, button::Status::Active);
            let survol = bouton_contour(p, p.texte, button::Status::Hovered);
            assert_ne!(repos.background, survol.background);

            let repos = bouton_fantome(p, p.texte, button::Status::Active);
            let survol = bouton_fantome(p, p.texte, button::Status::Hovered);
            assert_ne!(repos.background, survol.background);

            let repos = bouton_ligne(p, false, button::Status::Active);
            let survol = bouton_ligne(p, false, button::Status::Hovered);
            assert_ne!(repos.background, survol.background);
        }
    }

    /// Le focus d'un champ doit rester visible : contour d'accent et épaisseur
    /// doublée, dans les deux thèmes.
    #[test]
    fn le_focus_est_toujours_visible() {
        for p in palettes() {
            let repos = champ(p, text_input::Status::Active, false);
            let focus = champ(p, text_input::Status::Focused, false);
            assert_eq!(focus.border.color, p.accent);
            assert!(focus.border.width > repos.border.width);
        }
    }

    /// Une erreur prime sur le focus, sinon corriger un champ ferait disparaître
    /// le signal d'erreur.
    #[test]
    fn lerreur_prime_sur_le_focus() {
        for p in palettes() {
            let en_erreur = champ(p, text_input::Status::Focused, true);
            assert_eq!(en_erreur.border.color, p.danger);
        }
    }

    /// Un bouton désactivé ne doit jamais garder la couleur d'appel à l'action.
    #[test]
    fn le_desactive_perd_lappel_a_laction() {
        for p in palettes() {
            let desactive = bouton_plein(p, p.accent, p.accent_contraste, button::Status::Disabled);
            assert_eq!(desactive.text_color, p.texte_doux);
            assert_ne!(desactive.background, Some(Background::Color(p.accent)));
        }
    }

    /// L'élévation choisit bien une surface distincte par niveau.
    #[test]
    fn lelevation_choisit_des_surfaces_distinctes() {
        for p in palettes() {
            assert_ne!(
                surface(p, Elevation::Basse, Rayon::MD).background,
                surface(p, Elevation::Plate, Rayon::MD).background
            );
            assert_ne!(
                surface(p, Elevation::Plate, Rayon::MD).background,
                surface(p, Elevation::Modale, Rayon::MD).background
            );
            assert!(
                surface(p, Elevation::Modale, Rayon::MD).shadow.blur_radius
                    > surface(p, Elevation::Plate, Rayon::MD).shadow.blur_radius
            );
        }
    }

    /// La tuile du rail ne porte plus le remplissage de l'état actif — c'est
    /// la pastille d'icône qui s'en charge (voir `composants::navigation`).
    /// Elle doit en revanche porter la couleur du libellé et réagir au survol.
    #[test]
    fn la_tuile_du_rail_porte_le_libelle_et_le_survol() {
        for p in palettes() {
            let actif = bouton_navigation(p, true, button::Status::Active);
            let inactif = bouton_navigation(p, false, button::Status::Active);
            assert_eq!(actif.text_color, p.accent);
            assert_ne!(actif.text_color, inactif.text_color);

            assert_ne!(
                inactif.background,
                bouton_navigation(p, false, button::Status::Hovered).background
            );
        }
    }
}
