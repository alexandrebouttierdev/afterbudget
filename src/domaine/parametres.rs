use crate::domaine::argent::Money;

#[derive(Debug, Clone)]
pub struct AppSettings {
    pub current_balance: Money,
    pub overdraft_limit: Money,
    pub currency_code: String,
    pub locale: String,
    pub theme: String,
    pub balance_updated_at: String,
    pub onboarding_completed: bool,
    pub last_export_date: Option<String>,
    /// Dernière version de mise à jour ignorée par l'utilisateur : tant
    /// qu'elle est la plus récente, elle n'est plus proposée.
    pub ignored_update_version: Option<String>,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            current_balance: Money::ZERO,
            overdraft_limit: Money::ZERO,
            currency_code: "EUR".into(),
            locale: "fr-FR".into(),
            theme: "system".into(),
            balance_updated_at: chrono::Utc::now().to_rfc3339(),
            onboarding_completed: false,
            last_export_date: None,
            ignored_update_version: None,
        }
    }
}
