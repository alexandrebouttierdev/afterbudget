use crate::core::db::pool::DatabasePool;
use crate::domaine::argent::Money;
use crate::domaine::parametres::AppSettings;
use crate::modules::parametres::repository as repo;

pub fn obtenir_parametres(pool: &DatabasePool) -> Result<Option<AppSettings>, String> {
    repo::get_settings(pool)
}

pub fn marquer_dernier_export(pool: &DatabasePool) -> Result<(), String> {
    let mut settings = repo::get_settings(pool)?.unwrap_or_default();
    settings.last_export_date = Some(chrono::Utc::now().to_rfc3339());
    repo::update_settings(pool, &settings)
}

pub fn mettre_a_jour_solde(pool: &DatabasePool, solde: Money) -> Result<(), String> {
    let mut settings = repo::get_settings(pool)?.unwrap_or_default();
    settings.current_balance = solde;
    settings.balance_updated_at = chrono::Utc::now().to_rfc3339();
    repo::update_settings(pool, &settings)
}

pub fn mettre_a_jour_decouvert(pool: &DatabasePool, decouvert: Money) -> Result<(), String> {
    let mut settings = repo::get_settings(pool)?.unwrap_or_default();
    settings.overdraft_limit = decouvert;
    repo::update_settings(pool, &settings)
}

pub fn mettre_a_jour_theme(pool: &DatabasePool, theme: &str) -> Result<(), String> {
    let mut settings = repo::get_settings(pool)?.unwrap_or_default();
    settings.theme = theme.to_string();
    repo::update_settings(pool, &settings)
}

pub fn mettre_a_jour_devise(pool: &DatabasePool, devise: &str) -> Result<(), String> {
    let mut settings = repo::get_settings(pool)?.unwrap_or_default();
    settings.currency_code = devise.to_string();
    repo::update_settings(pool, &settings)
}

pub fn terminer_onboarding(
    pool: &DatabasePool,
    solde: Money,
    decouvert: Money,
    devise: &str,
) -> Result<(), String> {
    let now = chrono::Utc::now().to_rfc3339();
    let settings = AppSettings {
        current_balance: solde,
        overdraft_limit: decouvert,
        currency_code: devise.to_string(),
        locale: "fr-FR".into(),
        theme: "system".into(),
        balance_updated_at: now.clone(),
        onboarding_completed: true,
        last_export_date: None,
    };
    if repo::get_settings(pool)?.is_some() {
        repo::update_settings(pool, &settings)
    } else {
        repo::insert_settings(pool, &settings)
    }
}
