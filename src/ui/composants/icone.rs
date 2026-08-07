//! Jeu d'icônes vectorielles.
//!
//! Les icônes sont des SVG au trait embarqués dans le binaire. Elles sont
//! teintées à l'affichage, ce qu'un emoji couleur ne permet pas, et rendues de
//! façon identique sur les trois plateformes.

use iced::widget::svg;
use iced::{Color, Element, Length};

/// Taille d'icône. Trois seulement, alignées sur l'échelle typographique.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Taille {
    /// Accolée à un texte de légende.
    Petite,
    /// Cas courant : bouton, ligne de liste, rail.
    Normale,
    /// État vide, en-tête d'onboarding.
    Grande,
}

impl Taille {
    pub const fn pixels(self) -> f32 {
        match self {
            Self::Petite => 14.0,
            Self::Normale => 18.0,
            Self::Grande => 32.0,
        }
    }
}

macro_rules! jeu_dicones {
    ($($variante:ident => $fichier:literal),* $(,)?) => {
        /// Toutes les icônes disponibles.
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum Icone {
            $($variante),*
        }

        impl Icone {
            /// Octets SVG embarqués correspondants.
            const fn source(self) -> &'static [u8] {
                match self {
                    $(Self::$variante => {
                        include_bytes!(concat!("../../../assets/icones/", $fichier, ".svg"))
                    })*
                }
            }

            /// Liste exhaustive, utilisée par les tests.
            #[cfg(test)]
            pub const TOUTES: &'static [Self] = &[$(Self::$variante),*];
        }
    };
}

jeu_dicones! {
    Accueil => "accueil",
    Transactions => "transactions",
    Statistiques => "statistiques",
    Parametres => "parametres",

    Plus => "plus",
    Moins => "moins",
    Crayon => "crayon",
    Corbeille => "corbeille",
    Croix => "croix",
    Coche => "coche",
    Recherche => "recherche",
    Filtre => "filtre",
    Import => "import",
    Export => "export",
    ChevronGauche => "chevron-gauche",
    ChevronDroit => "chevron-droit",
    ChevronBas => "chevron-bas",
    Calendrier => "calendrier",

    FlecheEntrante => "fleche-entrante",
    FlecheSortante => "fleche-sortante",
    TendanceHaut => "tendance-haut",
    TendanceBas => "tendance-bas",

    Info => "info",
    Alerte => "alerte",
    Danger => "danger",
    Succes => "succes",
    Sablier => "sablier",
    Soleil => "soleil",
    Lune => "lune",
    Points => "points",
    Cadenas => "cadenas",

    Maison => "maison",
    Panier => "panier",
    Voiture => "voiture",
    Coeur => "coeur",
    Manette => "manette",
    Vetement => "vetement",
    Cycle => "cycle",
    Couverts => "couverts",
    Avion => "avion",
    Diplome => "diplome",
    Tirelire => "tirelire",
    Eclair => "eclair",
    Onde => "onde",
    Mobile => "mobile",
    Halteres => "halteres",
    Musique => "musique",
    Livre => "livre",
    Enfant => "enfant",
    Patte => "patte",
    Cle => "cle",
    Cadeau => "cadeau",
    Etincelles => "etincelles",
    CarteBancaire => "carte-bancaire",
    Cigarette => "cigarette",
    Bouclier => "bouclier",
    Portefeuille => "portefeuille",
    Pieces => "pieces",
    Etoile => "etoile",
    Retour => "retour",
    Etiquette => "etiquette",
    Mallette => "mallette",
}

impl Icone {
    /// Résout le nom d'icône stocké sur une catégorie.
    ///
    /// Les catégories par défaut portent des noms issus de la bibliothèque
    /// Lucide. Une catégorie créée avec un nom inconnu reste lisible grâce au
    /// repli sur `Points` : aucune catégorie ne se retrouve sans icône.
    pub fn depuis_nom(nom: &str) -> Self {
        match nom {
            "Home" => Self::Maison,
            "ShoppingCart" => Self::Panier,
            "Car" => Self::Voiture,
            "Heart" | "HeartHandshake" => Self::Coeur,
            "Gamepad2" => Self::Manette,
            "Shirt" => Self::Vetement,
            "RefreshCw" => Self::Cycle,
            "Utensils" => Self::Couverts,
            "Plane" => Self::Avion,
            "GraduationCap" => Self::Diplome,
            "PiggyBank" => Self::Tirelire,
            "Zap" => Self::Eclair,
            "Wifi" => Self::Onde,
            "Smartphone" => Self::Mobile,
            "Dumbbell" => Self::Halteres,
            "Music" => Self::Musique,
            "BookOpen" => Self::Livre,
            "Baby" => Self::Enfant,
            "PawPrint" => Self::Patte,
            "Wrench" => Self::Cle,
            "Gift" => Self::Cadeau,
            "Sparkles" => Self::Etincelles,
            "CreditCard" => Self::CarteBancaire,
            "Cigarette" => Self::Cigarette,
            "Shield" => Self::Bouclier,
            "Wallet" => Self::Portefeuille,
            "HandCoins" => Self::Pieces,
            "Star" => Self::Etoile,
            "Undo2" => Self::Retour,
            "Tag" => Self::Etiquette,
            "Briefcase" => Self::Mallette,
            "TrendingUp" => Self::TendanceHaut,
            _ => Self::Points,
        }
    }
}

/// Rend une icône teintée.
pub fn icone<'a, Message: 'a>(
    icone: Icone,
    taille: Taille,
    couleur: Color,
) -> Element<'a, Message> {
    svg(svg::Handle::from_memory(icone.source()))
        .width(Length::Fixed(taille.pixels()))
        .height(Length::Fixed(taille.pixels()))
        .style(move |_theme, _statut| svg::Style {
            color: Some(couleur),
        })
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domaine::categorie::{DEFAULT_EXPENSE_CATEGORIES, DEFAULT_INCOME_CATEGORIES};

    /// Chaque catégorie livrée par défaut doit obtenir une icône dédiée, sinon
    /// l'écran des catégories retombe sur une rangée de points identiques.
    #[test]
    fn toutes_les_categories_par_defaut_ont_une_icone_dediee() {
        for categorie in DEFAULT_EXPENSE_CATEGORIES
            .iter()
            .chain(DEFAULT_INCOME_CATEGORIES.iter())
        {
            let resolue = Icone::depuis_nom(categorie.icon);
            if categorie.icon == "MoreHorizontal" {
                assert_eq!(resolue, Icone::Points);
            } else {
                assert_ne!(
                    resolue,
                    Icone::Points,
                    "la catégorie « {} » ({}) n'a pas d'icône dédiée",
                    categorie.name,
                    categorie.icon
                );
            }
        }
    }

    /// Un nom inconnu ou vide ne doit jamais faire disparaître l'icône.
    #[test]
    fn un_nom_inconnu_retombe_sur_licone_generique() {
        assert_eq!(Icone::depuis_nom(""), Icone::Points);
        assert_eq!(Icone::depuis_nom("Inexistant"), Icone::Points);
        assert_eq!(Icone::depuis_nom("home"), Icone::Points);
    }

    /// Toutes les sources SVG doivent être présentes et non vides : une erreur
    /// de chemin serait sinon invisible jusqu'à l'exécution.
    #[test]
    fn toutes_les_sources_sont_valides() {
        for icone in Icone::TOUTES {
            let source = icone.source();
            assert!(!source.is_empty(), "{icone:?} est vide");
            let texte = std::str::from_utf8(source).expect("SVG en UTF-8");
            assert!(texte.contains("<svg"), "{icone:?} n'est pas un SVG");
            assert!(
                texte.contains("currentColor"),
                "{icone:?} ne peut pas être teintée"
            );
        }
    }

    #[test]
    fn les_tailles_sont_croissantes() {
        assert!(Taille::Petite.pixels() < Taille::Normale.pixels());
        assert!(Taille::Normale.pixels() < Taille::Grande.pixels());
    }
}
