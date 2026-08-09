use rusqlite::params;

use crate::core::db::modeles::SettingsRow;
use crate::core::db::pool::DatabasePool;
use crate::domaine::parametres::AppSettings;

pub fn get_settings(pool: &DatabasePool) -> Result<Option<AppSettings>, String> {
    let result = pool.conn.query_row(
        "SELECT id, current_balance_cents, overdraft_limit_cents, currency_code, locale, theme,
                balance_updated_at, onboarding_completed, created_at, updated_at, last_export_date,
                ignored_update_version
         FROM app_settings WHERE id = 1",
        [],
        |row| {
            Ok(SettingsRow {
                id: row.get(0)?,
                current_balance_cents: row.get(1)?,
                overdraft_limit_cents: row.get(2)?,
                currency_code: row.get(3)?,
                locale: row.get(4)?,
                theme: row.get(5)?,
                balance_updated_at: row.get(6)?,
                onboarding_completed: row.get::<_, i32>(7)? != 0,
                created_at: row.get(8)?,
                updated_at: row.get(9)?,
                last_export_date: row.get(10)?,
                ignored_update_version: row.get(11)?,
            })
        },
    );

    match result {
        Ok(row) => Ok(Some(row_to_settings(&row))),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(format!("Erreur de lecture des paramètres : {}", e)),
    }
}

pub fn insert_settings(pool: &DatabasePool, s: &AppSettings) -> Result<(), String> {
    let now = chrono::Utc::now().to_rfc3339();
    pool.conn
        .execute(
            "INSERT INTO app_settings (id, current_balance_cents, overdraft_limit_cents, currency_code, locale, theme,
             balance_updated_at, onboarding_completed, last_export_date, ignored_update_version, created_at, updated_at)
             VALUES (1, ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?10)",
            params![
                s.current_balance.cents,
                s.overdraft_limit.cents,
                s.currency_code,
                s.locale,
                s.theme,
                s.balance_updated_at,
                s.onboarding_completed as i32,
                s.last_export_date,
                s.ignored_update_version,
                &now,
            ],
        )
        .map_err(|e| format!("Erreur d'insertion des paramètres : {}", e))?;
    Ok(())
}

pub fn update_settings(pool: &DatabasePool, s: &AppSettings) -> Result<(), String> {
    let now = chrono::Utc::now().to_rfc3339();
    let maj = pool
        .conn
        .execute(
            "UPDATE app_settings SET current_balance_cents = ?1, overdraft_limit_cents = ?2,
             currency_code = ?3, locale = ?4, theme = ?5, balance_updated_at = ?6,
             onboarding_completed = ?7, last_export_date = ?8, ignored_update_version = ?9,
             updated_at = ?10
             WHERE id = 1",
            params![
                s.current_balance.cents,
                s.overdraft_limit.cents,
                s.currency_code,
                s.locale,
                s.theme,
                s.balance_updated_at,
                s.onboarding_completed as i32,
                s.last_export_date,
                s.ignored_update_version,
                &now,
            ],
        )
        .map_err(|e| format!("Erreur de mise à jour des paramètres : {}", e))?;

    if maj == 0 {
        return Err("Aucun paramètre à mettre à jour : la ligne id=1 est absente.".into());
    }
    Ok(())
}

/// Efface toutes les données d'une transaction unique : règles récurrentes,
/// transactions, catégories personnalisées, paramètres. En cas d'erreur, tout
/// est annulé — jamais de suppression partielle (AB-004).
pub fn reset_all_data(pool: &DatabasePool) -> Result<(), String> {
    let tx = pool
        .conn
        .unchecked_transaction()
        .map_err(|e| format!("Démarrage de la transaction de réinitialisation : {e}"))?;

    tx.execute("DELETE FROM recurring_rules", [])
        .map_err(|e| format!("Erreur de réinitialisation des règles : {e}"))?;
    tx.execute("DELETE FROM transactions", [])
        .map_err(|e| format!("Erreur de réinitialisation des transactions : {e}"))?;
    tx.execute("DELETE FROM categories WHERE is_default = 0", [])
        .map_err(|e| format!("Erreur de réinitialisation des catégories : {e}"))?;
    tx.execute("DELETE FROM app_settings", [])
        .map_err(|e| format!("Erreur de réinitialisation des paramètres : {e}"))?;

    tx.commit()
        .map_err(|e| format!("Erreur de commit de la réinitialisation : {e}"))?;
    Ok(())
}

fn row_to_settings(row: &SettingsRow) -> AppSettings {
    use crate::domaine::argent::Money;

    AppSettings {
        current_balance: Money::from_cents(row.current_balance_cents),
        overdraft_limit: Money::from_cents(row.overdraft_limit_cents),
        currency_code: row.currency_code.clone(),
        locale: row.locale.clone(),
        theme: row.theme.clone(),
        balance_updated_at: row.balance_updated_at.clone(),
        onboarding_completed: row.onboarding_completed,
        last_export_date: row.last_export_date.clone(),
        ignored_update_version: row.ignored_update_version.clone(),
    }
}
