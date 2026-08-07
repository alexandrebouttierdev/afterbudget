//! Palette « Encre & Cuivre ».
//!
//! Source unique de toutes les couleurs de l'application. Chaque champ est un
//! **rôle**, jamais une teinte : une vue demande `palette.depense`, jamais
//! « le rouge ». Les deux thèmes définissent exactement les mêmes rôles, ce qui
//! rend structurellement impossible qu'un composant retombe sur une couleur de
//! l'autre thème.

use iced::Color;

pub use crate::app::state::ThemeMode;

/// Construit une couleur à partir d'un littéral hexadécimal 24 bits.
const fn hex(valeur: u32) -> Color {
    Color::from_rgb(
        ((valeur >> 16) & 0xFF) as f32 / 255.0,
        ((valeur >> 8) & 0xFF) as f32 / 255.0,
        (valeur & 0xFF) as f32 / 255.0,
    )
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Palette {
    /// Fond de la zone de contenu.
    pub fond: Color,
    /// Fond du rail de navigation.
    pub fond_rail: Color,
    /// Cartes, panneaux, lignes de liste.
    pub surface: Color,
    /// Modales, menus, survol d'une ligne.
    pub surface_haute: Color,
    /// Champs, encoches, pistes de jauge.
    pub surface_basse: Color,
    /// Séparateurs et contours au repos.
    pub bordure: Color,
    /// Contours au survol et contour de modale.
    pub bordure_forte: Color,
    /// Titres, montants, valeurs.
    pub texte_fort: Color,
    /// Corps de texte et libellés.
    pub texte: Color,
    /// Métadonnées, aides, unités.
    pub texte_doux: Color,
    /// Signature interactive : principal, actif, focus.
    pub accent: Color,
    /// Texte posé sur `accent`.
    pub accent_contraste: Color,
    /// Fond d'état actif ou sélectionné.
    pub accent_surface: Color,
    /// Sens : argent qui entre.
    pub revenu: Color,
    /// Sens : argent qui sort.
    pub depense: Color,
    /// État : confirmé, sain.
    pub succes: Color,
    /// État : à surveiller.
    pub attention: Color,
    /// État : risque, action destructive.
    pub danger: Color,
    pub succes_surface: Color,
    pub attention_surface: Color,
    pub danger_surface: Color,
    /// Voile posé derrière une modale.
    pub voile: Color,
    /// Teinte des ombres portées.
    pub ombre: Color,
}

impl Palette {
    const CLAIR: Self = Self {
        fond: hex(0xF4F0E9),
        fond_rail: hex(0xEDE7DC),
        surface: hex(0xFFFCF6),
        surface_haute: hex(0xFFFFFF),
        surface_basse: hex(0xEAE4D8),
        bordure: hex(0xE0D8C8),
        bordure_forte: hex(0xC9BDA8),
        texte_fort: hex(0x1C1813),
        texte: hex(0x4A4238),
        texte_doux: hex(0x7A7063),
        accent: hex(0x9C5A2E),
        accent_contraste: hex(0xFFFCF6),
        accent_surface: hex(0xF7E9DC),
        revenu: hex(0x1F6B4A),
        depense: hex(0xA32B33),
        succes: hex(0x1F6B4A),
        attention: hex(0x8A6208),
        danger: hex(0xA32B33),
        succes_surface: hex(0xDCEDE3),
        attention_surface: hex(0xFAEFD3),
        danger_surface: hex(0xFADEDE),
        voile: Color::from_rgba(0.11, 0.09, 0.07, 0.45),
        ombre: Color::from_rgba(0.29, 0.24, 0.18, 0.16),
    };

    const SOMBRE: Self = Self {
        fond: hex(0x14110D),
        fond_rail: hex(0x100D0A),
        surface: hex(0x1E1A15),
        surface_haute: hex(0x272219),
        surface_basse: hex(0x100D0A),
        bordure: hex(0x332C22),
        bordure_forte: hex(0x4A4032),
        texte_fort: hex(0xF6F1E7),
        texte: hex(0xD5CCBD),
        texte_doux: hex(0x9C9284),
        accent: hex(0xE09A5F),
        accent_contraste: hex(0x17130E),
        accent_surface: hex(0x382718),
        revenu: hex(0x6FC49A),
        depense: hex(0xF08A88),
        succes: hex(0x6FC49A),
        attention: hex(0xE0B45C),
        danger: hex(0xF08A88),
        succes_surface: hex(0x1B3327),
        attention_surface: hex(0x372B14),
        danger_surface: hex(0x3A1E1E),
        voile: Color::from_rgba(0.0, 0.0, 0.0, 0.60),
        ombre: Color::from_rgba(0.0, 0.0, 0.0, 0.45),
    };

    pub const fn pour(mode: ThemeMode) -> Self {
        match mode {
            ThemeMode::Light => Self::CLAIR,
            ThemeMode::Dark => Self::SOMBRE,
        }
    }

    /// Vrai pour le thème sombre. Sert aux rares réglages qui ne peuvent pas
    /// être exprimés par un rôle : sens d'un éclaircissement, force d'une ombre.
    pub fn est_sombre(&self) -> bool {
        self.fond.r < 0.5
    }

    /// Renforce une couleur interactive au survol : plus sombre en thème clair,
    /// plus claire en thème sombre, pour que le retour visuel aille toujours
    /// dans le sens d'un « rapprochement ».
    pub fn accentuer(&self, couleur: Color, force: f32) -> Color {
        let cible = if self.est_sombre() {
            Color::WHITE
        } else {
            Color::BLACK
        };
        melange(couleur, cible, force)
    }

    /// Fond faiblement teinté par une couleur sémantique, utilisable derrière
    /// du texte de cette même couleur.
    pub fn teinter_surface(&self, couleur: Color) -> Color {
        melange(
            self.surface,
            couleur,
            if self.est_sombre() { 0.18 } else { 0.10 },
        )
    }
}

/// Mélange linéaire de deux couleurs (`t = 0` renvoie `a`, `t = 1` renvoie `b`).
pub fn melange(a: Color, b: Color, t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    Color::from_rgba(
        a.r + (b.r - a.r) * t,
        a.g + (b.g - a.g) * t,
        a.b + (b.b - a.b) * t,
        a.a + (b.a - a.a) * t,
    )
}

/// Applique une transparence à une couleur.
pub fn avec_alpha(couleur: Color, alpha: f32) -> Color {
    Color {
        a: alpha.clamp(0.0, 1.0),
        ..couleur
    }
}

/// Luminance relative au sens de WCAG 2.
fn luminance(couleur: Color) -> f32 {
    fn canal(v: f32) -> f32 {
        if v <= 0.03928 {
            v / 12.92
        } else {
            ((v + 0.055) / 1.055).powf(2.4)
        }
    }
    0.2126 * canal(couleur.r) + 0.7152 * canal(couleur.g) + 0.0722 * canal(couleur.b)
}

/// Teinte d'une couleur, en degrés sur la roue chromatique.
///
/// Sert à vérifier que deux rôles porteurs de sens opposé restent séparés par
/// la teinte, et pas seulement par la luminosité.
pub fn teinte_degres(couleur: Color) -> f32 {
    let (r, g, b) = (couleur.r, couleur.g, couleur.b);
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;
    if delta <= f32::EPSILON {
        return 0.0;
    }
    let brute = if max == r {
        60.0 * (((g - b) / delta) % 6.0)
    } else if max == g {
        60.0 * ((b - r) / delta + 2.0)
    } else {
        60.0 * ((r - g) / delta + 4.0)
    };
    (brute + 360.0) % 360.0
}

/// Écart angulaire entre deux teintes, entre 0 et 180 degrés.
pub fn ecart_de_teinte(a: Color, b: Color) -> f32 {
    let ecart = (teinte_degres(a) - teinte_degres(b)).abs();
    ecart.min(360.0 - ecart)
}

/// Rapport de contraste WCAG entre deux couleurs opaques.
pub fn contraste(a: Color, b: Color) -> f32 {
    let (la, lb) = (luminance(a), luminance(b));
    let (clair, sombre) = if la > lb { (la, lb) } else { (lb, la) };
    (clair + 0.05) / (sombre + 0.05)
}

/// Convertit la couleur hexadécimale stockée d'une catégorie, puis
/// l'**harmonise** avec la palette.
///
/// Les teintes livrées avec les catégories par défaut sont des couleurs web
/// très saturées : posées telles quelles sur un fond papier ou sur un charbon
/// chaud, elles crèvent l'écran et cassent l'identité du produit. On conserve
/// donc la teinte — c'est elle qui identifie la catégorie — mais on ramène la
/// saturation et la clarté dans la plage du design system, différente selon le
/// thème.
pub fn couleur_categorie(valeur: &str, palette: Palette) -> Color {
    let valeur = valeur.trim_start_matches('#');
    if valeur.len() != 6 {
        return palette.texte_doux;
    }
    match u32::from_str_radix(valeur, 16) {
        Ok(brut) => harmoniser(hex(brut), palette),
        Err(_) => palette.texte_doux,
    }
}

/// Saturation maximale tolérée pour une teinte de catégorie.
const SATURATION_MAX: f32 = 0.46;
/// Clarté cible d'une teinte de catégorie, par thème.
const CLARTE_CLAIR: f32 = 0.40;
const CLARTE_SOMBRE: f32 = 0.66;

/// Ramène une couleur arbitraire dans la plage chromatique du design system.
pub fn harmoniser(couleur: Color, palette: Palette) -> Color {
    let (teinte, saturation, _) = vers_tsl(couleur);
    let saturation = saturation.min(SATURATION_MAX);
    let clarte = if palette.est_sombre() {
        CLARTE_SOMBRE
    } else {
        CLARTE_CLAIR
    };
    depuis_tsl(teinte, saturation, clarte)
}

/// Conversion RVB → teinte / saturation / clarté.
fn vers_tsl(couleur: Color) -> (f32, f32, f32) {
    let (r, g, b) = (couleur.r, couleur.g, couleur.b);
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let clarte = (max + min) / 2.0;
    let delta = max - min;

    if delta <= f32::EPSILON {
        return (0.0, 0.0, clarte);
    }

    let saturation = delta / (1.0 - (2.0 * clarte - 1.0).abs());
    (teinte_degres(couleur), saturation.clamp(0.0, 1.0), clarte)
}

/// Conversion teinte / saturation / clarté → RVB.
fn depuis_tsl(teinte: f32, saturation: f32, clarte: f32) -> Color {
    let c = (1.0 - (2.0 * clarte - 1.0).abs()) * saturation;
    let secteur = (teinte % 360.0) / 60.0;
    let x = c * (1.0 - (secteur % 2.0 - 1.0).abs());
    let m = clarte - c / 2.0;

    let (r, g, b) = match secteur as u32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };

    Color::from_rgb(r + m, g + m, b + m)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Tous les rôles de texte et de sens doivent rester lisibles sur la
    /// surface de leur thème. Ce test verrouille la promesse d'accessibilité.
    #[test]
    fn contrastes_minimaux_respectes() {
        for mode in [ThemeMode::Light, ThemeMode::Dark] {
            let p = Palette::pour(mode);
            let roles: [(&str, Color); 7] = [
                ("texte_fort", p.texte_fort),
                ("texte", p.texte),
                ("texte_doux", p.texte_doux),
                ("accent", p.accent),
                ("revenu", p.revenu),
                ("depense", p.depense),
                ("attention", p.attention),
            ];
            for (nom, couleur) in roles {
                let ratio = contraste(couleur, p.surface);
                assert!(
                    ratio >= 4.5,
                    "{nom} : contraste {ratio:.2} insuffisant sur la surface ({mode:?})"
                );
            }

            let sur_accent = contraste(p.accent_contraste, p.accent);
            assert!(
                sur_accent >= 4.5,
                "texte sur accent : {sur_accent:.2} ({mode:?})"
            );
        }
    }

    /// Le fond du rail et la surface doivent se distinguer du fond de page,
    /// sinon la structure de la fenêtre disparaît.
    #[test]
    fn les_surfaces_se_distinguent_du_fond() {
        for mode in [ThemeMode::Light, ThemeMode::Dark] {
            let p = Palette::pour(mode);
            assert_ne!(p.fond, p.surface);
            assert_ne!(p.fond, p.fond_rail);
            assert_ne!(p.surface, p.surface_haute);
        }
    }

    /// Le thème sombre n'est pas une inversion : ses surfaces s'éclaircissent
    /// avec l'élévation, alors que le thème clair fait l'inverse.
    #[test]
    fn lelevation_suit_le_sens_du_theme() {
        let clair = Palette::pour(ThemeMode::Light);
        let sombre = Palette::pour(ThemeMode::Dark);

        assert!(luminance(clair.surface_haute) > luminance(clair.surface));
        assert!(luminance(clair.surface) > luminance(clair.fond));

        assert!(luminance(sombre.surface_haute) > luminance(sombre.surface));
        assert!(luminance(sombre.surface) > luminance(sombre.fond));
        assert!(luminance(sombre.fond) < luminance(clair.fond));
    }

    /// Revenus et dépenses doivent rester séparés **par la teinte**.
    ///
    /// Leur luminance est volontairement proche — les deux montants ont le même
    /// poids visuel dans une colonne — donc un test de contraste ne dirait rien
    /// d'utile ici. La redondance non colorée (icône, signe) est vérifiée dans
    /// `ligne_transaction`.
    #[test]
    fn le_sens_de_largent_se_lit_a_la_teinte() {
        for mode in [ThemeMode::Light, ThemeMode::Dark] {
            let p = Palette::pour(mode);
            let ecart = ecart_de_teinte(p.revenu, p.depense);
            assert!(
                ecart >= 60.0,
                "revenu et dépense ne sont séparés que de {ecart:.0}° ({mode:?})"
            );
            assert_ne!(p.accent, p.revenu);
            assert_ne!(p.accent, p.depense);
        }
    }

    #[test]
    fn la_teinte_est_calculee_sur_la_roue_chromatique() {
        assert_eq!(teinte_degres(Color::from_rgb(1.0, 0.0, 0.0)).round(), 0.0);
        assert_eq!(teinte_degres(Color::from_rgb(0.0, 1.0, 0.0)).round(), 120.0);
        assert_eq!(teinte_degres(Color::from_rgb(0.0, 0.0, 1.0)).round(), 240.0);
        // Une couleur neutre n'a pas de teinte exploitable.
        assert_eq!(teinte_degres(Color::from_rgb(0.5, 0.5, 0.5)), 0.0);
    }

    #[test]
    fn lecart_de_teinte_prend_le_chemin_le_plus_court() {
        let rouge = Color::from_rgb(1.0, 0.0, 0.0);
        let magenta = Color::from_rgb(1.0, 0.0, 0.9);
        // 0° et ~306° : l'écart réel est de 54°, pas de 306°.
        assert!(ecart_de_teinte(rouge, magenta) <= 60.0);
        assert!(ecart_de_teinte(rouge, rouge) < 1.0);
    }

    #[test]
    fn accentuer_va_dans_le_sens_du_theme() {
        let clair = Palette::pour(ThemeMode::Light);
        let sombre = Palette::pour(ThemeMode::Dark);

        assert!(luminance(clair.accentuer(clair.accent, 0.1)) < luminance(clair.accent));
        assert!(luminance(sombre.accentuer(sombre.accent, 0.1)) > luminance(sombre.accent));
    }

    /// La teinte d'origine identifie la catégorie : elle doit être conservée,
    /// même si la saturation et la clarté sont ramenées dans la plage du thème.
    #[test]
    fn lharmonisation_conserve_la_teinte() {
        for mode in [ThemeMode::Light, ThemeMode::Dark] {
            let p = Palette::pour(mode);
            for reference in ["#3B82F6", "#22C55E", "#EF4444", "#A855F7", "#EAB308"] {
                let ecart = ecart_de_teinte(
                    couleur_categorie(reference, p),
                    hex(u32::from_str_radix(reference.trim_start_matches('#'), 16).unwrap()),
                );
                assert!(
                    ecart < 6.0,
                    "{reference} a dérivé de {ecart:.1}° ({mode:?})"
                );
            }
        }
    }

    /// Deux catégories de teintes différentes doivent rester différentes après
    /// harmonisation, sinon elles deviennent indistinguables dans un anneau.
    #[test]
    fn lharmonisation_ne_confond_pas_les_categories() {
        for mode in [ThemeMode::Light, ThemeMode::Dark] {
            let p = Palette::pour(mode);
            let bleu = couleur_categorie("#3B82F6", p);
            let vert = couleur_categorie("#22C55E", p);
            let rouge = couleur_categorie("#EF4444", p);
            assert!(ecart_de_teinte(bleu, vert) > 60.0);
            assert!(ecart_de_teinte(vert, rouge) > 60.0);
        }
    }

    /// Le préfixe croisillon est facultatif.
    #[test]
    fn le_croisillon_est_facultatif() {
        let clair = Palette::pour(ThemeMode::Light);
        assert_eq!(
            couleur_categorie("#3B82F6", clair),
            couleur_categorie("3B82F6", clair)
        );
    }

    #[test]
    fn une_couleur_de_categorie_invalide_retombe_sur_la_palette() {
        for mode in [ThemeMode::Light, ThemeMode::Dark] {
            let p = Palette::pour(mode);
            assert_eq!(couleur_categorie("", p), p.texte_doux);
            assert_eq!(couleur_categorie("#zzzzzz", p), p.texte_doux);
            assert_eq!(couleur_categorie("#abc", p), p.texte_doux);
        }
    }

    /// Une même catégorie est plus claire en thème sombre qu'en thème clair :
    /// c'est ce qui lui garde un contraste utile sur les deux fonds.
    #[test]
    fn les_categories_suivent_la_clarte_du_theme() {
        let clair = Palette::pour(ThemeMode::Light);
        let sombre = Palette::pour(ThemeMode::Dark);
        for reference in ["#3B82F6", "#22C55E", "#EAB308"] {
            assert!(
                luminance(couleur_categorie(reference, sombre))
                    > luminance(couleur_categorie(reference, clair)),
                "{reference} ne s'éclaircit pas en thème sombre"
            );
        }
    }

    /// Les teintes web très saturées doivent être calmées, sinon elles crèvent
    /// l'écran sur un fond papier.
    #[test]
    fn lharmonisation_calme_les_teintes_criardes() {
        for mode in [ThemeMode::Light, ThemeMode::Dark] {
            let p = Palette::pour(mode);
            let (_, saturation, _) = vers_tsl(couleur_categorie("#3B82F6", p));
            assert!(
                saturation <= SATURATION_MAX + 1e-3,
                "saturation {saturation}"
            );
        }
    }

    #[test]
    fn les_conversions_chromatiques_font_laller_retour() {
        for reference in [0x3B82F6u32, 0x22C55E, 0xEF4444, 0x808080] {
            let couleur = hex(reference);
            let (teinte, saturation, clarte) = vers_tsl(couleur);
            let retour = depuis_tsl(teinte, saturation, clarte);
            assert!((retour.r - couleur.r).abs() < 0.02, "{reference:#08x}");
            assert!((retour.g - couleur.g).abs() < 0.02, "{reference:#08x}");
            assert!((retour.b - couleur.b).abs() < 0.02, "{reference:#08x}");
        }
    }

    #[test]
    fn melange_aux_extremites() {
        let a = Color::BLACK;
        let b = Color::WHITE;
        assert_eq!(melange(a, b, 0.0), a);
        assert_eq!(melange(a, b, 1.0), b);
        assert_eq!(melange(a, b, -3.0), a);
        assert_eq!(melange(a, b, 4.0), b);
    }
}
