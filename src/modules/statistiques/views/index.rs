//! Écran Statistiques.
//!
//! Chaque bloc répond à une question précise :
//! - « Ai-je dépensé plus ou moins que le mois dernier ? » → comparaison ;
//! - « Où part mon argent ? » → anneau de répartition et classement ;
//! - « Qu'ai-je réellement encaissé ? » → part réalisée.
//!
//! Aucun graphique n'est là pour décorer, et aucune donnée n'est portée par la
//! seule couleur : chaque segment de l'anneau est repris en légende, avec son
//! montant et sa part.

use iced::widget::{column, container, row, Space};
use iced::{Alignment, Element, Length};

use crate::app::message::{Message, Screen};
use crate::app::state::AppState;
use crate::domaine::argent::Money;
use crate::domaine::budget::{CategoryStats, MonthlyStatistics};
use crate::ui::composants::anneau::{anneau, preparer_segments, Segment};
use crate::ui::composants::badge::{badge_icone, Ton};
use crate::ui::composants::bouton::{Bouton, Variante};
use crate::ui::composants::carte;
use crate::ui::composants::en_tete_ecran::en_tete as en_tete_generique;
use crate::ui::composants::etat::Etat;
use crate::ui::composants::icone::{icone, Icone, Taille as TailleIcone};
use crate::ui::composants::jauge;
use crate::ui::composants::selecteur_mois::{selecteur_mois, Actions};
use crate::ui::composants::separateur;
use crate::ui::theme::espacements::Esp;
use crate::ui::theme::palette::{couleur_categorie, Palette};
use crate::ui::theme::typographie::{montant_aligne, texte_colore, Role};

pub fn en_tete(state: &AppState) -> Element<'_, Message> {
    let palette = Palette::pour(state.theme_mode);

    en_tete_generique(
        Screen::Statistics.display_name(),
        Some(Screen::Statistics.precision().into()),
        Some(selecteur_mois(
            state.current_year,
            state.current_month,
            state.is_current_month(),
            Actions {
                precedent: Message::PreviousMonth,
                suivant: Message::NextMonth,
                aujourdhui: Message::GoToCurrentMonth,
            },
            palette,
        )),
        Vec::new(),
        palette,
    )
}

pub fn corps(state: &AppState) -> Element<'_, Message> {
    let palette = Palette::pour(state.theme_mode);

    let vide = state
        .monthly_statistics
        .as_ref()
        .is_none_or(|s| s.transaction_count == 0);

    if vide {
        return container(carte::carte(
            Etat::nouveau(
                Icone::Statistiques,
                "Rien à analyser pour ce mois",
                "Les statistiques apparaissent dès la première transaction enregistrée.",
                palette,
            )
            .avec_action(
                Bouton::nouveau("Ajouter une dépense", palette)
                    .variante(Variante::Principal)
                    .avec_icone(Icone::Plus)
                    .sur_clic(Message::OpenAddExpense)
                    .vue(),
            )
            .vue(),
            carte::Variante::Plate,
            palette,
        ))
        .padding([Esp::LG, 0])
        .into();
    }

    let statistiques = state.monthly_statistics.as_ref().expect("statistiques");

    let repartitions: Element<'_, Message> = if state.mise_en_page.statistiques_deux_colonnes() {
        row![
            container(bloc_repartition(
                "Répartition des dépenses",
                &statistiques.expenses_by_category,
                statistiques.total_expenses,
                palette,
            ))
            .width(Length::FillPortion(1)),
            container(bloc_repartition(
                "Répartition des revenus",
                &statistiques.income_by_category,
                statistiques.total_income,
                palette,
            ))
            .width(Length::FillPortion(1)),
        ]
        .spacing(Esp::LG)
        .into()
    } else {
        column![
            bloc_repartition(
                "Répartition des dépenses",
                &statistiques.expenses_by_category,
                statistiques.total_expenses,
                palette,
            ),
            bloc_repartition(
                "Répartition des revenus",
                &statistiques.income_by_category,
                statistiques.total_income,
                palette,
            ),
        ]
        .spacing(Esp::LG)
        .into()
    };

    column![
        bandeau_comparaison(state, statistiques, palette),
        bloc_avancement(statistiques, palette),
        repartitions,
        Space::with_height(Length::Fixed(f32::from(Esp::XL))),
    ]
    .spacing(Esp::LG)
    .padding([Esp::LG, 0])
    .width(Length::Fill)
    .into()
}

// ── Comparaison avec le mois précédent ────────────────────────────────────

/// Écart relatif entre deux montants, en pourcentage.
///
/// Renvoie `None` quand la référence est nulle : « +100 % par rapport à zéro »
/// n'a aucun sens et serait trompeur.
pub fn ecart_relatif(actuel: Money, reference: Money) -> Option<f64> {
    if reference.cents == 0 {
        return None;
    }
    Some(
        actuel.cents.saturating_sub(reference.cents) as f64
            / reference.cents.saturating_abs() as f64
            * 100.0,
    )
}

/// Libellé d'une comparaison de dépenses avec le mois précédent.
pub fn libelle_ecart(ecart: Option<f64>) -> String {
    match ecart {
        None => "Pas de comparaison possible".to_string(),
        Some(valeur) if valeur.abs() < 0.5 => "Stable par rapport au mois dernier".to_string(),
        Some(valeur) if valeur > 0.0 => {
            format!("{:.0} % de plus que le mois dernier", valeur)
        }
        Some(valeur) => format!("{:.0} % de moins que le mois dernier", valeur.abs()),
    }
}

/// Pour des dépenses, une hausse est un signal d'attention, une baisse un
/// succès. L'inverse vaudrait pour des revenus.
pub fn ton_de_lecart(ecart: Option<f64>, hausse_favorable: bool) -> Ton {
    match ecart {
        None => Ton::Neutre,
        Some(valeur) if valeur.abs() < 0.5 => Ton::Neutre,
        Some(valeur) => {
            let hausse = valeur > 0.0;
            if hausse == hausse_favorable {
                Ton::Succes
            } else {
                Ton::Attention
            }
        }
    }
}

fn bandeau_comparaison<'a>(
    state: &'a AppState,
    statistiques: &'a MonthlyStatistics,
    palette: Palette,
) -> Element<'a, Message> {
    let precedent = state.previous_statistics.as_ref();

    let ecart_depenses =
        precedent.and_then(|p| ecart_relatif(statistiques.total_expenses, p.total_expenses));
    let ecart_revenus =
        precedent.and_then(|p| ecart_relatif(statistiques.total_income, p.total_income));

    let solde = statistiques.balance;
    let couleur_solde = if solde.is_negative() {
        palette.depense
    } else {
        palette.revenu
    };

    row![
        chiffre_cle(
            "Revenus",
            statistiques.total_income.format_fr(),
            libelle_ecart(ecart_revenus),
            ton_de_lecart(ecart_revenus, true),
            palette.revenu,
            palette,
        ),
        chiffre_cle(
            "Dépenses",
            statistiques.total_expenses.format_fr(),
            libelle_ecart(ecart_depenses),
            ton_de_lecart(ecart_depenses, false),
            palette.depense,
            palette,
        ),
        chiffre_cle(
            "Solde du mois",
            solde.format_fr_signed(),
            format!("{} mouvements", statistiques.transaction_count),
            Ton::Neutre,
            couleur_solde,
            palette,
        ),
    ]
    .spacing(Esp::LG)
    .width(Length::Fill)
    .into()
}

fn chiffre_cle<'a>(
    intitule: &'a str,
    montant: String,
    precision: String,
    ton: Ton,
    teinte: iced::Color,
    palette: Palette,
) -> Element<'a, Message> {
    let symbole = match ton {
        Ton::Succes => Icone::TendanceHaut,
        Ton::Attention => Icone::TendanceBas,
        _ => Icone::Info,
    };

    carte::carte(
        column![
            texte_colore(intitule, Role::Micro, palette.texte_doux),
            texte_colore(montant, Role::MontantFort, teinte),
            badge_icone(precision, symbole, ton, palette),
        ]
        .spacing(Esp::SM)
        .align_x(Alignment::Start),
        carte::Variante::Plate,
        palette,
    )
    .width(Length::FillPortion(1))
    .into()
}

// ── Avancement du mois ────────────────────────────────────────────────────

/// Part déjà réalisée d'un total, entre 0 et 1.
pub fn part_realisee(realise: Money, total: Money) -> f32 {
    if total.cents <= 0 {
        return 0.0;
    }
    (realise.cents as f32 / total.cents as f32).clamp(0.0, 1.0)
}

fn bloc_avancement<'a>(
    statistiques: &'a MonthlyStatistics,
    palette: Palette,
) -> Element<'a, Message> {
    let ligne =
        |intitule: &'a str, realise: Money, total: Money, restant: Money, teinte: iced::Color| {
            let part = part_realisee(realise, total);
            column![
                row![
                    texte_colore(intitule, Role::Corps, palette.texte),
                    Space::with_width(Length::Fill),
                    montant_aligne(
                        format!("{} sur {}", realise.format_fr(), total.format_fr()),
                        Role::Corps,
                        palette.texte_fort,
                    ),
                ]
                .align_y(Alignment::Center),
                jauge::jauge(part, teinte, palette),
                texte_colore(
                    format!(
                        "{:.0} % réalisé · {} restant",
                        part * 100.0,
                        restant.format_fr()
                    ),
                    Role::Legende,
                    palette.texte_doux,
                ),
            ]
            .spacing(Esp::XS + 2)
        };

    carte::carte(
        column![
            texte_colore("Avancement du mois", Role::TitreSection, palette.texte_fort),
            ligne(
                "Revenus encaissés",
                statistiques.completed_income,
                statistiques.total_income,
                statistiques.pending_income,
                palette.revenu,
            ),
            ligne(
                "Dépenses payées",
                statistiques.completed_expenses,
                statistiques.total_expenses,
                statistiques.pending_expenses,
                palette.depense,
            ),
        ]
        .spacing(Esp::LG),
        carte::Variante::Plate,
        palette,
    )
    .into()
}

// ── Répartition par catégorie ─────────────────────────────────────────────

fn bloc_repartition<'a>(
    titre: &'a str,
    categories: &'a [CategoryStats],
    total: Money,
    palette: Palette,
) -> Element<'a, Message> {
    if categories.is_empty() {
        return carte::carte(
            column![
                texte_colore(titre, Role::TitreSection, palette.texte_fort),
                texte_colore(
                    "Aucun mouvement de ce type ce mois-ci.",
                    Role::Corps,
                    palette.texte_doux,
                ),
            ]
            .spacing(Esp::SM),
            carte::Variante::Plate,
            palette,
        )
        .into();
    }

    let segments = preparer_segments(
        categories
            .iter()
            .map(|c| {
                (
                    c.category_name.clone(),
                    c.total.cents,
                    couleur_categorie(&c.category_color, palette),
                )
            })
            .collect(),
        palette.texte_doux,
    );

    let centre = column![
        texte_colore("Total", Role::Micro, palette.texte_doux),
        texte_colore(total.format_fr(), Role::TitreSection, palette.texte_fort),
    ]
    .align_x(Alignment::Center)
    .spacing(Esp::XXS);

    let graphique = iced::widget::stack![
        anneau(segments.clone(), 168.0, palette),
        container(centre)
            .width(Length::Fixed(168.0))
            .height(Length::Fixed(168.0))
            .align_x(Alignment::Center)
            .align_y(Alignment::Center),
    ];

    let mut legende = column![].spacing(Esp::SM).width(Length::Fill);
    for segment in &segments {
        legende = legende.push(ligne_de_legende(segment, palette));
    }

    carte::carte(
        column![
            texte_colore(titre, Role::TitreSection, palette.texte_fort),
            container(graphique)
                .width(Length::Fill)
                .align_x(Alignment::Center),
            separateur::horizontal(palette),
            legende,
        ]
        .spacing(Esp::LG),
        carte::Variante::Plate,
        palette,
    )
    .into()
}

fn ligne_de_legende<'a>(segment: &Segment, palette: Palette) -> Element<'a, Message> {
    let couleur = segment.couleur;

    column![
        row![
            icone(Icone::Etiquette, TailleIcone::Petite, couleur),
            texte_colore(segment.intitule.clone(), Role::Corps, palette.texte),
            Space::with_width(Length::Fill),
            montant_aligne(
                Money::from_cents(segment.montant_centimes).format_fr(),
                Role::Corps,
                palette.texte_fort,
            ),
            container(texte_colore(
                format!("{:.0} %", segment.part * 100.0),
                Role::Legende,
                palette.texte_doux,
            ))
            .width(Length::Fixed(48.0))
            .align_x(Alignment::End),
        ]
        .spacing(Esp::SM)
        .align_y(Alignment::Center),
        jauge::jauge(segment.part, couleur, palette),
    ]
    .spacing(Esp::XS)
    .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lecart_relatif_est_calcule_sur_la_reference() {
        let ecart = ecart_relatif(Money::from_cents(15000), Money::from_cents(10000));
        assert_eq!(ecart, Some(50.0));

        let baisse = ecart_relatif(Money::from_cents(8000), Money::from_cents(10000));
        assert_eq!(baisse, Some(-20.0));
    }

    /// Comparer à un mois sans données produirait un pourcentage absurde.
    #[test]
    fn aucune_comparaison_sans_reference() {
        assert_eq!(ecart_relatif(Money::from_cents(15000), Money::ZERO), None);
        assert_eq!(libelle_ecart(None), "Pas de comparaison possible");
    }

    #[test]
    fn le_libelle_decart_est_explicite() {
        assert_eq!(
            libelle_ecart(Some(23.4)),
            "23 % de plus que le mois dernier"
        );
        assert_eq!(
            libelle_ecart(Some(-23.4)),
            "23 % de moins que le mois dernier"
        );
        assert_eq!(
            libelle_ecart(Some(0.2)),
            "Stable par rapport au mois dernier"
        );
    }

    /// Une hausse de dépenses appelle l'attention ; une hausse de revenus est
    /// une bonne nouvelle. Le même écart n'a donc pas le même ton.
    #[test]
    fn le_ton_depend_du_sens_favorable() {
        assert_eq!(ton_de_lecart(Some(20.0), false), Ton::Attention);
        assert_eq!(ton_de_lecart(Some(20.0), true), Ton::Succes);
        assert_eq!(ton_de_lecart(Some(-20.0), false), Ton::Succes);
        assert_eq!(ton_de_lecart(Some(-20.0), true), Ton::Attention);
        assert_eq!(ton_de_lecart(None, false), Ton::Neutre);
        assert_eq!(ton_de_lecart(Some(0.1), false), Ton::Neutre);
    }

    #[test]
    fn la_part_realisee_est_bornee() {
        assert_eq!(
            part_realisee(Money::from_cents(5000), Money::from_cents(10000)),
            0.5
        );
        assert_eq!(part_realisee(Money::from_cents(5000), Money::ZERO), 0.0);
        assert_eq!(
            part_realisee(Money::from_cents(20000), Money::from_cents(10000)),
            1.0
        );
    }
}
