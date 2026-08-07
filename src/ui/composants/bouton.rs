//! Boutons.
//!
//! Cinq variantes, trois tailles. Aucun écran ne redéfinit son propre bouton :
//! l'ordre d'importance visuelle est décidé une fois pour toutes ici.

use std::borrow::Cow;

use iced::widget::{button, row, text, Button};
use iced::{Alignment, Element, Length};

use crate::ui::composants::icone::{icone, Icone, Taille as TailleIcone};
use crate::ui::theme::espacements::Esp;
use crate::ui::theme::palette::Palette;
use crate::ui::theme::styles;
use crate::ui::theme::typographie::{police, Role};

/// Poids d'un bouton dans la hiérarchie d'un écran.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Variante {
    /// Action dominante. Une seule par zone.
    Principal,
    /// Action alternative de même niveau.
    Secondaire,
    /// Action de service, sans cadre.
    Discret,
    /// Action irréversible.
    Destructif,
    /// Bouton d'icône nu, pour les actions de ligne.
    Fantome,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Taille {
    Normale,
    Compacte,
    /// Carré, pour une icône seule. Respecte la cible minimale de 32 px.
    Icone,
}

impl Taille {
    fn padding(self) -> [u16; 2] {
        match self {
            Self::Normale => [Esp::MD - 2, Esp::LG],
            Self::Compacte => [Esp::SM - 1, Esp::MD],
            Self::Icone => [Esp::SM - 1, Esp::SM - 1],
        }
    }

    fn role(self) -> Role {
        match self {
            Self::Normale => Role::CorpsFort,
            Self::Compacte | Self::Icone => Role::Libelle,
        }
    }

    fn taille_icone(self) -> TailleIcone {
        match self {
            Self::Normale | Self::Icone => TailleIcone::Normale,
            Self::Compacte => TailleIcone::Petite,
        }
    }
}

/// Bouton en cours de construction.
pub struct Bouton<'a, Message> {
    libelle: Option<Cow<'a, str>>,
    icone: Option<Icone>,
    variante: Variante,
    taille: Taille,
    message: Option<Message>,
    pleine_largeur: bool,
    palette: Palette,
}

impl<'a, Message: Clone + 'a> Bouton<'a, Message> {
    pub fn nouveau(libelle: impl Into<Cow<'a, str>>, palette: Palette) -> Self {
        Self {
            libelle: Some(libelle.into()),
            icone: None,
            variante: Variante::Secondaire,
            taille: Taille::Normale,
            message: None,
            pleine_largeur: false,
            palette,
        }
    }

    /// Bouton réduit à une icône, destiné à être enveloppé dans une infobulle
    /// qui en donne l'intitulé.
    pub fn icone(symbole: Icone, palette: Palette) -> Self {
        Self {
            libelle: None,
            icone: Some(symbole),
            variante: Variante::Fantome,
            taille: Taille::Icone,
            message: None,
            pleine_largeur: false,
            palette,
        }
    }

    pub fn variante(mut self, variante: Variante) -> Self {
        self.variante = variante;
        self
    }

    pub fn taille(mut self, taille: Taille) -> Self {
        self.taille = taille;
        self
    }

    pub fn avec_icone(mut self, symbole: Icone) -> Self {
        self.icone = Some(symbole);
        self
    }

    /// Sans message, le bouton est rendu désactivé par Iced.
    pub fn sur_clic(mut self, message: Message) -> Self {
        self.message = Some(message);
        self
    }

    pub fn pleine_largeur(mut self) -> Self {
        self.pleine_largeur = true;
        self
    }

    pub fn vue(self) -> Element<'a, Message> {
        self.construire().into()
    }

    fn construire(self) -> Button<'a, Message> {
        let palette = self.palette;
        let variante = self.variante;
        let teinte = couleur_texte(variante, palette);

        let mut contenu = row![].align_y(Alignment::Center).spacing(Esp::SM);
        if let Some(symbole) = self.icone {
            contenu = contenu.push(icone(symbole, self.taille.taille_icone(), teinte));
        }
        if let Some(libelle) = self.libelle {
            contenu = contenu.push(
                text(libelle)
                    .size(self.taille.role().taille())
                    .font(police(self.taille.role().graisse()))
                    .color(teinte),
            );
        }

        let mut bouton = button(contenu)
            .padding(self.taille.padding())
            .style(move |_theme, statut| style(variante, palette, statut));

        if self.pleine_largeur {
            bouton = bouton.width(Length::Fill);
        }
        if let Some(message) = self.message {
            bouton = bouton.on_press(message);
        }
        bouton
    }
}

fn couleur_texte(variante: Variante, palette: Palette) -> iced::Color {
    match variante {
        Variante::Principal => palette.accent_contraste,
        Variante::Secondaire => palette.texte_fort,
        Variante::Discret | Variante::Fantome => palette.texte_doux,
        Variante::Destructif => palette.danger,
    }
}

fn style(variante: Variante, palette: Palette, statut: button::Status) -> button::Style {
    match variante {
        Variante::Principal => {
            styles::bouton_plein(palette, palette.accent, palette.accent_contraste, statut)
        }
        Variante::Secondaire => styles::bouton_contour(palette, palette.texte_fort, statut),
        Variante::Destructif => styles::bouton_contour(palette, palette.danger, statut),
        Variante::Discret | Variante::Fantome => {
            styles::bouton_fantome(palette, palette.texte_doux, statut)
        }
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

    /// Le bouton principal est le seul à porter la couleur d'accent en aplat :
    /// c'est ce qui garantit qu'il n'y a qu'une action dominante visible.
    #[test]
    fn seul_le_principal_est_en_aplat_daccent() {
        for p in palettes() {
            let principal = style(Variante::Principal, p, button::Status::Active);
            assert_eq!(
                principal.background,
                Some(iced::Background::Color(p.accent))
            );

            for variante in [
                Variante::Secondaire,
                Variante::Discret,
                Variante::Destructif,
                Variante::Fantome,
            ] {
                let autre = style(variante, p, button::Status::Active);
                assert_ne!(autre.background, Some(iced::Background::Color(p.accent)));
            }
        }
    }

    /// L'action destructive doit se lire à sa couleur, sans devenir l'action
    /// dominante de l'écran.
    #[test]
    fn le_destructif_porte_la_couleur_de_danger() {
        for p in palettes() {
            assert_eq!(couleur_texte(Variante::Destructif, p), p.danger);
            assert_eq!(
                style(Variante::Destructif, p, button::Status::Active).text_color,
                p.danger
            );
        }
    }

    /// Toutes les variantes doivent réagir au survol, sans exception.
    #[test]
    fn toutes_les_variantes_reagissent_au_survol() {
        for p in palettes() {
            for variante in [
                Variante::Principal,
                Variante::Secondaire,
                Variante::Discret,
                Variante::Destructif,
                Variante::Fantome,
            ] {
                assert_ne!(
                    style(variante, p, button::Status::Active).background,
                    style(variante, p, button::Status::Hovered).background,
                    "{variante:?} ne réagit pas au survol"
                );
            }
        }
    }

    /// Un bouton d'icône doit rester au-dessus de la cible cliquable minimale.
    #[test]
    fn le_bouton_dicone_respecte_la_cible_minimale() {
        let [vertical, horizontal] = Taille::Icone.padding();
        let cote = TailleIcone::Normale.pixels() + 2.0 * f32::from(vertical.min(horizontal));
        assert!(cote >= 32.0, "cible de {cote} px trop petite");
    }
}
