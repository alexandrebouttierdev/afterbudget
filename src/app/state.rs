use crate::core::db::pool::DatabasePool;
use crate::domaine::budget::{BudgetSummary, MonthlyStatistics};
use crate::domaine::categorie::Category;
use crate::domaine::parametres::AppSettings;
use crate::domaine::transaction::{Transaction, TransactionKind, TransactionStatus};
use crate::ui::composants::badge::Ton;
use crate::ui::theme::mise_en_page::MiseEnPage;
use chrono::Datelike;

use super::message::Screen;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeMode {
    Light,
    Dark,
}

impl ThemeMode {
    /// Valeur stockée en base.
    pub fn cle(self) -> &'static str {
        match self {
            Self::Light => "light",
            Self::Dark => "dark",
        }
    }

    pub fn depuis_cle(cle: &str) -> Self {
        match cle {
            "dark" => Self::Dark,
            _ => Self::Light,
        }
    }

    pub fn inverse(self) -> Self {
        match self {
            Self::Light => Self::Dark,
            Self::Dark => Self::Light,
        }
    }
}

/// Notification éphémère affichée en bas à droite.
#[derive(Debug, Clone)]
pub struct Notification {
    pub texte: String,
    pub ton: Ton,
    /// Tics d'une seconde restant avant fermeture automatique.
    pub restant: u8,
}

impl Notification {
    pub fn succes(texte: impl Into<String>) -> Self {
        Self::nouvelle(texte, Ton::Succes)
    }

    pub fn erreur(texte: impl Into<String>) -> Self {
        Self::nouvelle(texte, Ton::Danger)
    }

    pub fn information(texte: impl Into<String>) -> Self {
        Self::nouvelle(texte, Ton::Accent)
    }

    fn nouvelle(texte: impl Into<String>, ton: Ton) -> Self {
        Self {
            texte: texte.into(),
            ton,
            restant: crate::ui::composants::notification::DUREE_TICS,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TransactionFormState {
    pub is_edit: bool,
    pub edit_id: Option<String>,
    pub kind: TransactionKind,
    pub label: String,
    pub amount_str: String,
    /// Date au format de saisie français JJ/MM/AAAA.
    pub date_str: String,
    /// Calendrier déplié sous le champ date.
    pub calendrier_ouvert: bool,
    /// Mois affiché par le calendrier, indépendant de la date saisie.
    pub calendrier_annee: i32,
    pub calendrier_mois: u32,
    pub category_id: String,
    pub status: TransactionStatus,
    pub note: String,
    /// Transformer la saisie en règle mensuelle plutôt qu'en transaction seule.
    pub recurrent: bool,
    pub label_error: Option<String>,
    pub amount_error: Option<String>,
    pub date_error: Option<String>,
    pub category_error: Option<String>,
}

impl Default for TransactionFormState {
    fn default() -> Self {
        Self {
            is_edit: false,
            edit_id: None,
            kind: TransactionKind::Expense,
            label: String::new(),
            amount_str: String::new(),
            date_str: crate::core::utils::format_date_saisie(&chrono::Utc::now().date_naive()),
            calendrier_ouvert: false,
            calendrier_annee: chrono::Utc::now().year(),
            calendrier_mois: chrono::Utc::now().month(),
            category_id: String::new(),
            status: TransactionStatus::Pending,
            note: String::new(),
            recurrent: false,
            label_error: None,
            amount_error: None,
            date_error: None,
            category_error: None,
        }
    }
}

impl TransactionFormState {
    /// Date effectivement saisie, si elle est complète et valide.
    pub fn date(&self) -> Option<chrono::NaiveDate> {
        crate::core::utils::analyser_date_saisie(&self.date_str)
    }
}

pub struct AppState {
    pub db: Option<DatabasePool>,
    pub db_path: String,

    pub screen: Screen,
    pub current_year: i32,
    pub current_month: u32,

    pub settings: Option<AppSettings>,
    pub budget: Option<BudgetSummary>,
    pub transactions: Vec<Transaction>,
    pub categories: Vec<Category>,
    pub recurrences: Vec<crate::domaine::recurrence::RecurringRule>,
    pub recent_transactions: Vec<Transaction>,

    pub theme_mode: ThemeMode,
    /// Décisions de mise en page dérivées de la taille de fenêtre.
    pub mise_en_page: MiseEnPage,
    pub notification: Option<Notification>,
    /// Phase de l'indicateur de chargement, avancée par le tic.
    pub phase_chargement: u8,

    pub show_transaction_form: bool,
    pub transaction_form: TransactionFormState,

    pub show_delete_confirm: bool,
    pub delete_transaction: Option<Transaction>,

    pub filter_kind: Option<TransactionKind>,
    pub filter_status: Option<TransactionStatus>,
    pub filter_category: Option<String>,
    pub search_query: String,

    pub onboarding_balance_str: String,
    pub onboarding_overdraft_str: String,
    pub onboarding_currency: String,
    pub onboarding_error: Option<String>,

    pub settings_balance_str: String,
    pub settings_overdraft_str: String,
    pub settings_error: Option<String>,
    /// Édition rapide du solde dépliée sur l'accueil.
    pub edition_solde_ouverte: bool,

    pub show_import_confirm: bool,
    pub import_file_path: Option<String>,
    pub import_error: Option<String>,

    pub show_reset_confirm: bool,

    pub monthly_statistics: Option<MonthlyStatistics>,
    /// Statistiques du mois précédent, pour les comparaisons.
    pub previous_statistics: Option<MonthlyStatistics>,
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

impl AppState {
    pub fn new() -> Self {
        let now = chrono::Utc::now();
        let db_path = crate::core::config::database_path();

        Self {
            db: None,
            db_path: db_path.to_string_lossy().to_string(),
            screen: Screen::Onboarding,
            current_year: now.year(),
            current_month: now.month(),
            settings: None,
            budget: None,
            transactions: Vec::new(),
            categories: Vec::new(),
            recurrences: Vec::new(),
            recent_transactions: Vec::new(),
            theme_mode: ThemeMode::Light,
            mise_en_page: MiseEnPage::default(),
            notification: None,
            phase_chargement: 0,
            show_transaction_form: false,
            transaction_form: TransactionFormState::default(),
            show_delete_confirm: false,
            delete_transaction: None,
            filter_kind: None,
            filter_status: None,
            filter_category: None,
            search_query: String::new(),
            onboarding_balance_str: String::new(),
            onboarding_overdraft_str: String::new(),
            onboarding_currency: "EUR".to_string(),
            onboarding_error: None,
            settings_balance_str: String::new(),
            settings_overdraft_str: String::new(),
            settings_error: None,
            edition_solde_ouverte: false,
            show_import_confirm: false,
            import_file_path: None,
            import_error: None,
            show_reset_confirm: false,
            monthly_statistics: None,
            previous_statistics: None,
        }
    }

    pub fn load_data(&mut self) -> Result<(), String> {
        let pool = self.db.as_ref().ok_or("Base de données non initialisée.")?;

        self.settings = crate::modules::parametres::commandes::obtenir_parametres(pool)?;

        if let Some(ref settings) = self.settings {
            if settings.onboarding_completed {
                self.screen = Screen::Dashboard;
            }
            self.theme_mode = ThemeMode::depuis_cle(&settings.theme);
        }

        if self
            .settings
            .as_ref()
            .is_some_and(|s| s.onboarding_completed)
        {
            self.categories = crate::modules::categories::commandes::lister(pool)?;
            self.recurrences = crate::modules::recurrences::commandes::lister(pool)?;
        }

        self.load_month_data()?;

        Ok(())
    }

    pub fn load_month_data(&mut self) -> Result<(), String> {
        let pool = self.db.as_ref().ok_or("Base de données non initialisée.")?;

        // Les règles récurrentes matérialisent leurs occurrences à l'ouverture
        // du mois, avant tout calcul : elles doivent entrer dans la prévision.
        // L'opération est idempotente, la rejouer est sans effet.
        crate::modules::recurrences::commandes::generer_pour_mois(
            pool,
            self.current_year,
            self.current_month,
        )?;

        self.budget = Some(crate::modules::budget::commandes::calculer_budget(
            pool,
            self.current_year,
            self.current_month,
        )?);

        let kind_str = self.filter_kind.map(|k| k.as_str().to_string());
        let status_str = self.filter_status.map(|s| s.as_str().to_string());
        let cat_filter = self.filter_category.clone();
        let search = if self.search_query.is_empty() {
            None
        } else {
            Some(self.search_query.clone())
        };

        self.transactions = crate::modules::transactions::commandes::lister(
            pool,
            self.current_year,
            self.current_month,
            kind_str.as_deref(),
            status_str.as_deref(),
            cat_filter.as_deref(),
            search.as_deref(),
        )?;

        self.recent_transactions = crate::modules::transactions::commandes::recentes(
            pool,
            self.current_year,
            self.current_month,
            6,
        )?;

        self.monthly_statistics = crate::modules::statistiques::commandes::calculer(
            pool,
            self.current_year,
            self.current_month,
        )
        .ok();

        let (annee_precedente, mois_precedent) =
            mois_precedent(self.current_year, self.current_month);
        self.previous_statistics = crate::modules::statistiques::commandes::calculer(
            pool,
            annee_precedente,
            mois_precedent,
        )
        .ok();

        Ok(())
    }

    pub fn is_current_month(&self) -> bool {
        let now = chrono::Utc::now();
        self.current_year == now.year() && self.current_month == now.month()
    }

    pub fn default_category_for(&self, kind: TransactionKind) -> Option<&Category> {
        self.categories
            .iter()
            .find(|c| c.kind == kind && c.is_active)
    }

    /// Vrai dès qu'un filtre ou une recherche restreint la liste : sert à
    /// choisir entre « aucune transaction » et « aucun résultat ».
    pub fn filtres_actifs(&self) -> bool {
        self.filter_kind.is_some()
            || self.filter_status.is_some()
            || self.filter_category.is_some()
            || !self.search_query.trim().is_empty()
    }

    /// Vrai lorsqu'une couche modale est ouverte : les raccourcis d'écran sont
    /// alors désactivés au profit de ceux de la modale.
    pub fn couche_modale_ouverte(&self) -> bool {
        self.show_transaction_form
            || self.show_delete_confirm
            || self.show_import_confirm
            || self.show_reset_confirm
    }

    /// Devise à afficher à côté des montants saisis.
    pub fn symbole_devise(&self) -> &'static str {
        match self.settings.as_ref().map(|s| s.currency_code.as_str()) {
            Some("USD") => "$",
            Some("GBP") => "£",
            _ => "€",
        }
    }
}

/// Mois précédant celui donné, en gérant le passage d'année.
pub fn mois_precedent(annee: i32, mois: u32) -> (i32, u32) {
    if mois == 1 {
        (annee - 1, 12)
    } else {
        (annee, mois - 1)
    }
}

/// Mois suivant celui donné, en gérant le passage d'année.
pub fn mois_suivant(annee: i32, mois: u32) -> (i32, u32) {
    if mois == 12 {
        (annee + 1, 1)
    } else {
        (annee, mois + 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn la_navigation_entre_mois_gere_le_passage_dannee() {
        assert_eq!(mois_precedent(2026, 1), (2025, 12));
        assert_eq!(mois_precedent(2026, 8), (2026, 7));
        assert_eq!(mois_suivant(2026, 12), (2027, 1));
        assert_eq!(mois_suivant(2026, 8), (2026, 9));
    }

    /// Aller au mois suivant puis revenir doit ramener au point de départ,
    /// y compris aux bornes d'année.
    #[test]
    fn la_navigation_entre_mois_est_reversible() {
        for (annee, mois) in [(2026, 1), (2026, 6), (2026, 12), (2025, 12)] {
            let (a, m) = mois_suivant(annee, mois);
            assert_eq!(mois_precedent(a, m), (annee, mois));
        }
    }

    #[test]
    fn le_mode_de_theme_fait_laller_retour_avec_sa_cle() {
        for mode in [ThemeMode::Light, ThemeMode::Dark] {
            assert_eq!(ThemeMode::depuis_cle(mode.cle()), mode);
        }
    }

    /// Une valeur de thème inconnue en base ne doit pas empêcher l'application
    /// de s'afficher.
    #[test]
    fn une_cle_de_theme_inconnue_retombe_sur_le_clair() {
        assert_eq!(ThemeMode::depuis_cle("system"), ThemeMode::Light);
        assert_eq!(ThemeMode::depuis_cle(""), ThemeMode::Light);
    }

    #[test]
    fn la_bascule_de_theme_alterne() {
        assert_eq!(ThemeMode::Light.inverse(), ThemeMode::Dark);
        assert_eq!(ThemeMode::Dark.inverse(), ThemeMode::Light);
    }

    /// La distinction « rien à afficher » / « rien ne correspond » repose sur
    /// cette règle : elle détermine quel état vide est proposé.
    #[test]
    fn les_filtres_actifs_sont_detectes() {
        let mut etat = AppState::new();
        assert!(!etat.filtres_actifs());

        etat.search_query = "   ".into();
        assert!(!etat.filtres_actifs(), "une recherche vide ne filtre rien");

        etat.search_query = "loyer".into();
        assert!(etat.filtres_actifs());

        etat.search_query.clear();
        etat.filter_kind = Some(TransactionKind::Income);
        assert!(etat.filtres_actifs());

        etat.filter_kind = None;
        etat.filter_category = Some("logement".into());
        assert!(etat.filtres_actifs());
    }

    #[test]
    fn la_couche_modale_est_detectee() {
        let mut etat = AppState::new();
        assert!(!etat.couche_modale_ouverte());

        etat.show_transaction_form = true;
        assert!(etat.couche_modale_ouverte());

        etat.show_transaction_form = false;
        etat.show_reset_confirm = true;
        assert!(etat.couche_modale_ouverte());
    }

    #[test]
    fn le_symbole_de_devise_suit_les_parametres() {
        let mut etat = AppState::new();
        assert_eq!(etat.symbole_devise(), "€");

        let mut parametres = AppSettings {
            currency_code: "USD".into(),
            ..Default::default()
        };
        etat.settings = Some(parametres.clone());
        assert_eq!(etat.symbole_devise(), "$");

        parametres.currency_code = "JPY".into();
        etat.settings = Some(parametres);
        assert_eq!(etat.symbole_devise(), "€");
    }

    /// Une notification naît toujours avec un compte à rebours : sans cela,
    /// elle resterait indéfiniment à l'écran.
    #[test]
    fn une_notification_nait_avec_un_compte_a_rebours() {
        let notification = Notification::succes("Enregistré.");
        assert!(notification.restant > 0);
        assert_eq!(notification.ton, Ton::Succes);
        assert_eq!(Notification::erreur("Raté.").ton, Ton::Danger);
    }
}
