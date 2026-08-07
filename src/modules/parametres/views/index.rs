//! Écran Paramètres.
//!
//! Six sections nettement séparées, de la plus courante à la plus risquée. La
//! zone dangereuse est isolée dans sa propre carte, avec sa propre teinte, tout
//! en bas : impossible de la confondre avec un réglage ordinaire.

use iced::widget::{column, row, Space};
use iced::{Alignment, Element, Length};

use crate::app::message::{Message, Screen};
use crate::app::state::{AppState, ThemeMode};
use crate::domaine::parametres::AppSettings;
use crate::modules::parametres::composants::section::{reglage, reglage_large, section, Registre};
use crate::ui::composants::bouton::{Bouton, Taille as TailleBouton, Variante};
use crate::ui::composants::champ;
use crate::ui::composants::en_tete_ecran::en_tete as en_tete_generique;
use crate::ui::composants::etat::chargement;
use crate::ui::composants::icone::Icone;
use crate::ui::theme::espacements::Esp;
use crate::ui::theme::palette::Palette;
use crate::ui::theme::typographie::{texte_colore, Role};

// ── Options des sélecteurs ────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
struct OptionTheme(ThemeMode);

impl std::fmt::Display for OptionTheme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self.0 {
            ThemeMode::Light => "Clair",
            ThemeMode::Dark => "Sombre",
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct OptionDevise(&'static str);

impl OptionDevise {
    const TOUTES: [Self; 3] = [Self("EUR"), Self("USD"), Self("GBP")];

    fn depuis(code: &str) -> Self {
        Self::TOUTES
            .into_iter()
            .find(|option| option.0 == code)
            .unwrap_or(Self("EUR"))
    }
}

impl std::fmt::Display for OptionDevise {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self.0 {
            "USD" => "Dollar américain ($)",
            "GBP" => "Livre sterling (£)",
            _ => "Euro (€)",
        })
    }
}

pub fn en_tete(state: &AppState) -> Element<'_, Message> {
    let palette = Palette::pour(state.theme_mode);

    en_tete_generique(
        Screen::Settings.display_name(),
        Some(Screen::Settings.precision().into()),
        None,
        Vec::new(),
        palette,
    )
}

pub fn corps(state: &AppState) -> Element<'_, Message> {
    let palette = Palette::pour(state.theme_mode);

    let Some(parametres) = state.settings.as_ref() else {
        return chargement(
            "Lecture de tes paramètres…",
            state.phase_chargement,
            palette,
        );
    };

    column![
        section_budget(state, parametres, palette),
        section_apparence(state, parametres, palette),
        section_recurrences(state, palette),
        section_donnees(state, parametres, palette),
        section_a_propos(palette),
        section_dangereuse(palette),
        Space::with_height(Length::Fixed(f32::from(Esp::XL))),
    ]
    .spacing(Esp::LG)
    .padding([Esp::LG, 0])
    .width(Length::Fill)
    .into()
}

// ── Budget ────────────────────────────────────────────────────────────────

fn section_budget<'a>(
    state: &'a AppState,
    parametres: &'a AppSettings,
    palette: Palette,
) -> Element<'a, Message> {
    let solde = reglage_large(
        "Solde actuel du compte",
        format!(
            "Point de départ de la prévision. Une valeur négative est acceptée si tu es déjà à découvert. Mis à jour le {}.",
            date_lisible(&parametres.balance_updated_at)
        ),
        column![
            texte_colore(
                parametres.current_balance.format_fr(),
                Role::MontantFort,
                palette.texte_fort,
            ),
            row![
                champ::saisie_solde(
                    &state.settings_balance_str,
                    "Nouveau solde, ex : -360 ou 1 250,50",
                    Message::SetCurrentBalance,
                    Message::UpdateBalance,
                    state.settings_error.is_some(),
                    palette,
                )
                .width(Length::Fill),
                Bouton::nouveau("Mettre à jour", palette)
                    .variante(Variante::Principal)
                    .sur_clic(Message::UpdateBalance)
                    .vue(),
            ]
            .spacing(Esp::SM)
            .align_y(Alignment::Center),
        ]
        .spacing(Esp::SM),
        palette,
    );

    let decouvert = reglage_large(
        "Découvert autorisé",
        "Sert à savoir si la fin du mois passe, et de combien.",
        column![
            texte_colore(
                parametres.overdraft_limit.format_fr(),
                Role::MontantFort,
                palette.texte_fort,
            ),
            row![
                champ::saisie_montant(
                    &state.settings_overdraft_str,
                    "Nouveau plafond, ex : 500",
                    Message::SetOverdraftLimit,
                    Message::UpdateOverdraft,
                    false,
                    palette,
                )
                .width(Length::Fill),
                Bouton::nouveau("Mettre à jour", palette)
                    .variante(Variante::Secondaire)
                    .sur_clic(Message::UpdateOverdraft)
                    .vue(),
            ]
            .spacing(Esp::SM)
            .align_y(Alignment::Center),
        ]
        .spacing(Esp::SM),
        palette,
    );

    let mut reglages = vec![solde, decouvert];
    if let Some(erreur) = state.settings_error.as_deref() {
        reglages.push(champ::message_derreur(erreur, palette));
    }

    section(
        Icone::Portefeuille,
        "Budget",
        "Les deux valeurs qui déterminent ta prévision de fin de mois.",
        reglages,
        Registre::Courant,
        palette,
    )
}

// ── Apparence ─────────────────────────────────────────────────────────────

fn section_apparence<'a>(
    state: &'a AppState,
    parametres: &'a AppSettings,
    palette: Palette,
) -> Element<'a, Message> {
    section(
        Icone::Soleil,
        "Apparence",
        "Thème et devise d'affichage.",
        vec![
            reglage(
                "Thème",
                "Le thème sombre est conçu séparément, pas inversé.",
                champ::selecteur(
                    vec![OptionTheme(ThemeMode::Light), OptionTheme(ThemeMode::Dark)],
                    Some(OptionTheme(state.theme_mode)),
                    |choix: OptionTheme| Message::SetTheme(choix.0.cle().to_string()),
                    palette,
                )
                .width(Length::Fill),
                palette,
            ),
            reglage(
                "Devise",
                "Symbole affiché à côté des montants saisis.",
                champ::selecteur(
                    OptionDevise::TOUTES.to_vec(),
                    Some(OptionDevise::depuis(&parametres.currency_code)),
                    |choix: OptionDevise| Message::SetCurrency(choix.0.to_string()),
                    palette,
                )
                .width(Length::Fill),
                palette,
            ),
        ],
        Registre::Courant,
        palette,
    )
}

// ── Opérations récurrentes ────────────────────────────────────────────────

fn section_recurrences(state: &AppState, palette: Palette) -> Element<'_, Message> {
    let actives = state.recurrences.iter().filter(|r| r.is_active).count();

    let explication = if actives == 0 {
        "Aucune pour l'instant. Coche « Répéter chaque mois » en créant une opération.".to_string()
    } else {
        format!(
            "{} règle{} mensuelle{}, matérialisée{} à l'ouverture de chaque mois.",
            actives,
            if actives > 1 { "s" } else { "" },
            if actives > 1 { "s" } else { "" },
            if actives > 1 { "s" } else { "" },
        )
    };

    let mut reglages: Vec<Element<'_, Message>> = Vec::new();
    for regle in &state.recurrences {
        reglages.push(ligne_de_recurrence(regle, &state.categories, palette));
    }

    section(
        Icone::Cycle,
        "Opérations récurrentes",
        explication,
        reglages,
        Registre::Courant,
        palette,
    )
}

/// Une règle : sens, libellé, périodicité, montant et retrait.
fn ligne_de_recurrence<'a>(
    regle: &'a crate::domaine::recurrence::RecurringRule,
    categories: &'a [crate::domaine::categorie::Category],
    palette: Palette,
) -> Element<'a, Message> {
    let (symbole, teinte, signe) = match regle.kind {
        crate::domaine::transaction::TransactionKind::Income => {
            (Icone::FlecheEntrante, palette.revenu, "+")
        }
        crate::domaine::transaction::TransactionKind::Expense => {
            (Icone::FlecheSortante, palette.depense, "\u{2212}")
        }
    };

    let categorie =
        crate::modules::categories::composants::pastille::nom(categories, &regle.category_id);

    row![
        crate::ui::composants::icone::icone(
            symbole,
            crate::ui::composants::icone::Taille::Normale,
            teinte
        ),
        column![
            texte_colore(regle.label.as_str(), Role::CorpsFort, palette.texte_fort),
            texte_colore(
                format!("{} · {}", regle.periodicite(), categorie),
                Role::Legende,
                palette.texte_doux,
            ),
        ]
        .spacing(Esp::XXS)
        .width(Length::Fill),
        texte_colore(
            format!("{signe}{}", regle.amount.format_fr()),
            Role::CorpsFort,
            teinte
        ),
        Bouton::icone(Icone::Corbeille, palette)
            .sur_clic(Message::DeleteRecurringRule(regle.id.clone()))
            .vue(),
    ]
    .spacing(Esp::MD)
    .align_y(Alignment::Center)
    .into()
}

// ── Données locales ───────────────────────────────────────────────────────

fn section_donnees<'a>(
    state: &'a AppState,
    parametres: &'a AppSettings,
    palette: Palette,
) -> Element<'a, Message> {
    let mut reglages = vec![
        reglage_large(
            "Emplacement de la base",
            "Tout est stocké dans ce fichier, sur ton ordinateur.",
            texte_colore(state.db_path.as_str(), Role::Legende, palette.texte_doux),
            palette,
        ),
        reglage(
            "Sauvegarde",
            match parametres.last_export_date.as_deref() {
                Some(date) => format!("Dernier export : {}", date_lisible(date)),
                None => "Aucun export réalisé pour l'instant.".to_string(),
            },
            row![
                Bouton::nouveau("Importer", palette)
                    .variante(Variante::Secondaire)
                    .avec_icone(Icone::Import)
                    .sur_clic(Message::InitiateImport)
                    .vue(),
                Bouton::nouveau("Exporter", palette)
                    .variante(Variante::Principal)
                    .avec_icone(Icone::Export)
                    .sur_clic(Message::ExportDatabase)
                    .vue(),
            ]
            .spacing(Esp::SM),
            palette,
        ),
    ];

    if let Some(erreur) = state.import_error.as_deref() {
        reglages.push(champ::message_derreur(erreur, palette));
    }

    section(
        Icone::Cadenas,
        "Données locales",
        "Aucun compte, aucun serveur : tes données ne quittent pas la machine.",
        reglages,
        Registre::Courant,
        palette,
    )
}

// ── À propos ──────────────────────────────────────────────────────────────

fn section_a_propos<'a>(palette: Palette) -> Element<'a, Message> {
    section(
        Icone::Info,
        "À propos",
        "AfterBudget, gestion de budget personnel hors ligne.",
        vec![
            reglage(
                "Version",
                "Application desktop native, écrite en Rust.",
                texte_colore(
                    env!("CARGO_PKG_VERSION"),
                    Role::CorpsFort,
                    palette.texte_fort,
                ),
                palette,
            ),
            reglage(
                "Site officiel",
                crate::app::update::SITE_WEB,
                Bouton::nouveau("Ouvrir le site", palette)
                    .variante(Variante::Secondaire)
                    .taille(TailleBouton::Compacte)
                    .avec_icone(Icone::Export)
                    .sur_clic(Message::OpenWebsite)
                    .vue(),
                palette,
            ),
        ],
        Registre::Courant,
        palette,
    )
}

// ── Zone dangereuse ───────────────────────────────────────────────────────

fn section_dangereuse<'a>(palette: Palette) -> Element<'a, Message> {
    section(
        Icone::Alerte,
        "Zone dangereuse",
        "Ces actions ne peuvent pas être annulées.",
        vec![reglage(
            "Réinitialiser toutes les données",
            "Supprime transactions, catégories, budgets et paramètres.",
            Bouton::nouveau("Tout effacer", palette)
                .variante(Variante::Destructif)
                .taille(TailleBouton::Compacte)
                .avec_icone(Icone::Corbeille)
                .sur_clic(Message::OpenResetConfirm)
                .vue(),
            palette,
        )],
        Registre::Dangereux,
        palette,
    )
}

/// Met une date ISO au format français, sans planter sur une valeur inattendue.
pub fn date_lisible(brut: &str) -> String {
    chrono::DateTime::parse_from_rfc3339(brut)
        .map(|date| date.format("%d/%m/%Y à %H:%M").to_string())
        .unwrap_or_else(|_| brut.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn une_date_iso_est_mise_au_format_francais() {
        let lisible = date_lisible("2026-08-03T14:05:00+00:00");
        assert!(lisible.starts_with("03/08/2026"), "{lisible}");
        assert!(lisible.contains(" à "), "{lisible}");
    }

    /// Une valeur illisible ne doit pas faire disparaître l'information.
    #[test]
    fn une_date_invalide_est_rendue_telle_quelle() {
        assert_eq!(date_lisible("jamais"), "jamais");
        assert_eq!(date_lisible(""), "");
    }

    /// La devise stockée doit toujours retomber sur une option valide.
    #[test]
    fn une_devise_inconnue_retombe_sur_leuro() {
        assert_eq!(OptionDevise::depuis("USD"), OptionDevise("USD"));
        assert_eq!(OptionDevise::depuis("JPY"), OptionDevise("EUR"));
        assert_eq!(OptionDevise::depuis(""), OptionDevise("EUR"));
    }

    /// Les intitulés du sélecteur de devise sont explicites, pas des codes.
    #[test]
    fn les_devises_sont_nommees() {
        assert_eq!(OptionDevise("EUR").to_string(), "Euro (€)");
        assert_eq!(OptionDevise("USD").to_string(), "Dollar américain ($)");
        assert_eq!(OptionDevise("GBP").to_string(), "Livre sterling (£)");
    }
}
