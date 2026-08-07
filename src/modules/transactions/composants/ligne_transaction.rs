//! Ligne de la liste des transactions.
//!
//! C'est une **ligne de tableau**, pas une carte : les colonnes sont alignées
//! d'une ligne à l'autre, le survol est matérialisé, et les actions
//! secondaires n'encombrent pas la ligne au repos.

use iced::widget::{button, column, container, row, Space};
use iced::{Alignment, Element, Length};

use crate::app::message::Message;
use crate::core::utils;
use crate::domaine::categorie::Category;
use crate::domaine::transaction::{Transaction, TransactionKind, TransactionStatus};
use crate::modules::categories::composants::pastille::{self, Taille as TaillePastille};
use crate::ui::composants::badge::Ton;
use crate::ui::composants::bouton::{Bouton, Taille as TailleBouton};
use crate::ui::composants::icone::{icone, Icone, Taille as TailleIcone};
use crate::ui::composants::infobulle::{infobulle, Position};
use crate::ui::theme::espacements::{Densite, Esp, Rayon};
use crate::ui::theme::palette::Palette;
use crate::ui::theme::styles;
use crate::ui::theme::typographie::{montant_aligne, texte_colore, Role};

/// Répartition des colonnes, partagée avec l'en-tête du tableau pour que les
/// deux restent alignés.
pub const PORTION_LIBELLE: u16 = 40;
pub const PORTION_CATEGORIE: u16 = 30;
pub const LARGEUR_DATE: f32 = 96.0;
pub const LARGEUR_STATUT: f32 = 118.0;
pub const LARGEUR_MONTANT: f32 = 132.0;
pub const LARGEUR_ACTIONS: f32 = 76.0;

/// Ton de badge correspondant au statut d'une transaction.
///
/// Isolé et testé : c'est le point de contact entre un état métier et une
/// variante visuelle.
pub fn ton_du_statut(statut: TransactionStatus) -> Ton {
    match statut {
        TransactionStatus::Completed => Ton::Succes,
        TransactionStatus::Pending => Ton::Attention,
    }
}

/// Icône, couleur et signe correspondant au sens d'une transaction.
///
/// Le sens est porté par **trois** signaux redondants : une icône de flèche, un
/// signe, et une couleur. Aucun n'est indispensable seul.
pub fn repere_du_sens(
    genre: TransactionKind,
    palette: Palette,
) -> (Icone, iced::Color, &'static str) {
    match genre {
        TransactionKind::Income => (Icone::FlecheEntrante, palette.revenu, "+"),
        TransactionKind::Expense => (Icone::FlecheSortante, palette.depense, "\u{2212}"),
    }
}

/// Montant signé, prêt à afficher dans une colonne alignée.
pub fn montant_signe(transaction: &Transaction) -> String {
    let signe = match transaction.kind {
        TransactionKind::Income => "+",
        // Signe moins typographique, plus lisible que le trait d'union.
        TransactionKind::Expense => "\u{2212}",
    };
    format!("{}{}", signe, transaction.amount.abs().format_fr())
}

/// Une ligne du tableau.
pub fn ligne<'a>(
    transaction: &'a Transaction,
    categories: &'a [Category],
    colonne_statut: bool,
    palette: Palette,
) -> Element<'a, Message> {
    let categorie = pastille::trouver(categories, &transaction.category_id);
    let (symbole, teinte, _) = repere_du_sens(transaction.kind, palette);

    // ── Libellé : sens, intitulé, et statut replié ici sur fenêtre étroite ──
    let mut identite = column![texte_colore(
        transaction.label.as_str(),
        Role::CorpsFort,
        palette.texte_fort,
    )
    // Sans retour à la ligne : un libellé long est coupé plutôt que de faire
    // varier la hauteur de la ligne et de casser l'alignement des colonnes.
    .wrapping(iced::widget::text::Wrapping::None)]
    .spacing(Esp::XXS);

    if !colonne_statut {
        identite = identite.push(bascule_statut(transaction, palette));
    }

    let libelle = row![icone(symbole, TailleIcone::Normale, teinte), identite,]
        .spacing(Esp::SM)
        .align_y(Alignment::Center);

    // ── Cellules ───────────────────────────────────────────────────────────
    let mut cellules = row![
        container(libelle).width(Length::FillPortion(PORTION_LIBELLE)),
        container(pastille::pastille_nommee(
            categorie,
            TaillePastille::Ligne,
            palette
        ))
        .width(Length::FillPortion(PORTION_CATEGORIE)),
        container(texte_colore(
            utils::format_date_fr(&transaction.transaction_date),
            Role::Legende,
            palette.texte_doux,
        ))
        .width(Length::Fixed(LARGEUR_DATE)),
    ]
    .align_y(Alignment::Center)
    .spacing(Esp::MD);

    if colonne_statut {
        cellules = cellules.push(
            container(bascule_statut(transaction, palette)).width(Length::Fixed(LARGEUR_STATUT)),
        );
    }

    cellules = cellules.push(
        container(
            row![
                Space::with_width(Length::Fill),
                montant_aligne(montant_signe(transaction), Role::CorpsFort, teinte),
            ]
            .align_y(Alignment::Center),
        )
        .width(Length::Fixed(LARGEUR_MONTANT)),
    );

    cellules = cellules
        .push(container(actions(transaction, palette)).width(Length::Fixed(LARGEUR_ACTIONS)));

    // La ligne reste un bouton pour obtenir un état de survol — Iced 0.13 ne
    // l'expose pas sur `container` — mais elle n'a plus d'action propre : un
    // clic global volait les clics destinés au contrôle de statut. L'édition
    // passe par son icône dédiée, en fin de ligne.
    button(
        container(cellules)
            .width(Length::Fill)
            .padding([Densite::Normale.padding_vertical(), Esp::LG]),
    )
    .width(Length::Fill)
    .padding(0)
    .style(move |_theme, statut| styles::bouton_ligne(palette, false, statut))
    .into()
}

/// Contrôle de statut.
///
/// Une case à cocher suivie de son libellé, et non un badge décoratif : le
/// motif est universellement compris comme cliquable, ce qu'un badge n'est
/// pas. Un clic bascule réalisé / en attente sans ouvrir de fenêtre.
pub fn bascule_statut<'a>(transaction: &'a Transaction, palette: Palette) -> Element<'a, Message> {
    let realise = transaction.status == TransactionStatus::Completed;
    let ton = ton_du_statut(transaction.status);
    let teinte = ton.teinte(palette);

    let case = container(if realise {
        icone(Icone::Coche, TailleIcone::Petite, palette.accent_contraste)
    } else {
        Space::new(Length::Fixed(0.0), Length::Fixed(0.0)).into()
    })
    .width(Length::Fixed(16.0))
    .height(Length::Fixed(16.0))
    .align_x(Alignment::Center)
    .align_y(Alignment::Center)
    .style(move |_theme| container::Style {
        background: Some(iced::Background::Color(if realise {
            palette.succes
        } else {
            palette.surface_basse
        })),
        border: iced::Border {
            color: if realise {
                palette.succes
            } else {
                palette.bordure_forte
            },
            width: 1.0,
            radius: Rayon::XS.into(),
        },
        ..container::Style::default()
    });

    let contenu = row![
        case,
        texte_colore(transaction.status.display_name(), Role::Legende, teinte),
    ]
    .spacing(Esp::XS + 2)
    .align_y(Alignment::Center);

    let bouton = button(contenu)
        .padding([Esp::XS - 1, Esp::SM - 2])
        .on_press(Message::ToggleTransactionStatus(transaction.id.clone()))
        .style(move |_theme, statut| styles::bouton_contour(palette, teinte, statut));

    infobulle(
        bouton,
        if realise {
            "Repasser en attente"
        } else {
            "Marquer comme réalisé"
        },
        Position::Top,
        palette,
    )
}

/// Actions de ligne. Elles restent discrètes : icônes fantômes, sans fond.
fn actions<'a>(transaction: &'a Transaction, palette: Palette) -> Element<'a, Message> {
    row![
        infobulle(
            Bouton::icone(Icone::Crayon, palette)
                .taille(TailleBouton::Icone)
                .sur_clic(Message::OpenEditTransaction(transaction.id.clone()))
                .vue(),
            "Modifier",
            Position::Top,
            palette,
        ),
        infobulle(
            Bouton::icone(Icone::Corbeille, palette)
                .taille(TailleBouton::Icone)
                .sur_clic(Message::DeleteTransaction(transaction.id.clone()))
                .vue(),
            "Supprimer",
            Position::Top,
            palette,
        ),
    ]
    .spacing(Esp::XXS)
    .align_y(Alignment::Center)
    .into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domaine::argent::Money;
    use crate::ui::theme::palette::ThemeMode;
    use chrono::NaiveDate;

    fn transaction(genre: TransactionKind, centimes: i64) -> Transaction {
        Transaction {
            id: "t1".into(),
            kind: genre,
            label: "Courses".into(),
            amount: Money::from_cents(centimes),
            transaction_date: NaiveDate::from_ymd_opt(2026, 8, 12).unwrap(),
            status: TransactionStatus::Pending,
            category_id: "alimentation".into(),
            note: None,
            recurring_rule_id: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }

    /// Le statut doit se traduire par des tons différents, sinon le badge
    /// n'apporte rien.
    #[test]
    fn chaque_statut_a_son_ton() {
        assert_eq!(ton_du_statut(TransactionStatus::Completed), Ton::Succes);
        assert_eq!(ton_du_statut(TransactionStatus::Pending), Ton::Attention);
        assert_ne!(
            ton_du_statut(TransactionStatus::Completed),
            ton_du_statut(TransactionStatus::Pending)
        );
    }

    /// Le sens doit être porté par l'icône et le signe, pas seulement par la
    /// couleur : c'est la règle d'accessibilité principale de l'écran.
    #[test]
    fn le_sens_ne_depend_pas_de_la_seule_couleur() {
        for mode in [ThemeMode::Light, ThemeMode::Dark] {
            let p = Palette::pour(mode);
            let (icone_revenu, couleur_revenu, signe_revenu) =
                repere_du_sens(TransactionKind::Income, p);
            let (icone_depense, couleur_depense, signe_depense) =
                repere_du_sens(TransactionKind::Expense, p);

            assert_ne!(icone_revenu, icone_depense);
            assert_ne!(signe_revenu, signe_depense);
            assert_ne!(couleur_revenu, couleur_depense);
            assert_eq!(couleur_revenu, p.revenu);
            assert_eq!(couleur_depense, p.depense);
        }
    }

    /// Le montant affiché porte le signe du sens et jamais un signe moins issu
    /// du formatage.
    #[test]
    fn le_montant_porte_le_signe_du_sens() {
        let revenu = montant_signe(&transaction(TransactionKind::Income, 42000));
        let depense = montant_signe(&transaction(TransactionKind::Expense, 42000));

        assert!(revenu.starts_with('+'), "{revenu}");
        assert!(depense.starts_with('\u{2212}'), "{depense}");
        assert!(revenu.ends_with('€'));
        assert!(!depense.contains("--"), "double signe : {depense}");
    }

    /// Les colonnes doivent rester compatibles avec la largeur minimale de
    /// contenu, sinon la ligne déborde.
    #[test]
    fn les_colonnes_tiennent_dans_la_largeur_minimale() {
        use crate::ui::theme::mise_en_page::MiseEnPage;

        let etroite = MiseEnPage::depuis_largeur(crate::ui::theme::mise_en_page::LARGEUR_MINIMALE);
        let fixes = LARGEUR_DATE + LARGEUR_MONTANT + LARGEUR_ACTIONS;
        assert!(
            fixes < etroite.largeur_contenu(),
            "les colonnes fixes ({fixes}) dépassent la largeur disponible"
        );
        assert!(
            !etroite.colonne_statut_visible(),
            "la colonne statut doit se replier sur une fenêtre étroite"
        );
    }
}
