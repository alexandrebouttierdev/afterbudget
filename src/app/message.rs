use crate::domaine::transaction::{TransactionKind, TransactionStatus};

/// Tous les messages de l'application.
#[derive(Debug, Clone)]
pub enum Message {
    // ---- Navigation ----
    NavigateTo(Screen),
    PreviousMonth,
    NextMonth,
    GoToCurrentMonth,

    // ---- Onboarding ----
    SetOnboardingBalance(String),
    SetOnboardingOverdraft(String),
    SetOnboardingCurrency(String),
    CompleteOnboarding,

    // ---- Paramètres ----
    SetCurrentBalance(String),
    UpdateBalance,
    OpenBalanceEdit,
    CancelBalanceEdit,
    SetOverdraftLimit(String),
    UpdateOverdraft,
    SetTheme(String),
    ToggleTheme,
    SetCurrency(String),

    // ---- Transactions ----
    OpenAddIncome,
    OpenAddExpense,
    OpenEditTransaction(String),
    CloseTransactionForm,
    DeleteTransaction(String),
    ConfirmDeleteTransaction,
    CancelDelete,

    // Champs du formulaire
    SetFormKind(TransactionKind),
    SetFormLabel(String),
    SetFormAmount(String),
    SetFormDate(String),
    ToggleFormCalendar,
    FormCalendarPreviousMonth,
    FormCalendarNextMonth,
    PickFormDate(chrono::NaiveDate),
    SetFormCategory(String),
    SetFormStatus(TransactionStatus),
    SetFormNote(String),
    SetFormRecurrent(bool),
    DeleteRecurringRule(String),
    SubmitTransactionForm,

    // Filtres
    SetFilterKind(Option<TransactionKind>),
    SetFilterStatus(Option<TransactionStatus>),
    SetFilterCategory(Option<String>),
    SetSearchQuery(String),
    ClearFilters,

    // Changement rapide de statut
    ToggleTransactionStatus(String),

    // ---- Import/Export ----
    ExportDatabase,
    InitiateImport,
    ConfirmImport(String),
    CancelImport,

    // ---- Réinitialisation ----
    OpenResetConfirm,
    ConfirmResetData,
    CancelReset,

    // ---- Raccourcis clavier ----
    FocusSearch,
    KeyboardEscape,
    KeyboardSave,

    // ---- Divers ----
    OpenWebsite,
    DismissNotification,
    WindowResized(f32, f32),
    Tick,
    /// Message sans effet, utilisé comme cible neutre d'une tâche.
    Ignore,
}

/// Écrans de l'application.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Onboarding,
    Dashboard,
    Transactions,
    Statistics,
    Settings,
}

impl Screen {
    /// Écrans atteignables depuis le rail, dans l'ordre d'affichage.
    pub const RAIL: &'static [Self] = &[
        Self::Dashboard,
        Self::Transactions,
        Self::Statistics,
        Self::Settings,
    ];

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Dashboard => "Accueil",
            Self::Transactions => "Transactions",
            Self::Statistics => "Statistiques",
            Self::Settings => "Paramètres",
            Self::Onboarding => "Bienvenue",
        }
    }

    /// Précision affichée sous le titre d'écran.
    pub fn precision(&self) -> &'static str {
        match self {
            Self::Dashboard => "Ce qu'il te restera à la fin du mois",
            Self::Transactions => "Tous les mouvements du mois",
            Self::Statistics => "Où part ton argent",
            Self::Settings => "Budget, apparence et données",
            Self::Onboarding => "Configuration initiale",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Le rail expose les quatre écrans principaux, et jamais l'onboarding.
    #[test]
    fn le_rail_expose_les_ecrans_principaux() {
        assert_eq!(Screen::RAIL.len(), 4);
        assert!(!Screen::RAIL.contains(&Screen::Onboarding));
    }

    /// Chaque écran doit avoir un intitulé et une précision non vides : ce sont
    /// eux qui portent l'en-tête d'écran.
    #[test]
    fn chaque_ecran_est_nomme_et_situe() {
        for ecran in [
            Screen::Onboarding,
            Screen::Dashboard,
            Screen::Transactions,
            Screen::Statistics,
            Screen::Settings,
        ] {
            assert!(!ecran.display_name().is_empty());
            assert!(!ecran.precision().is_empty());
        }
    }
}
