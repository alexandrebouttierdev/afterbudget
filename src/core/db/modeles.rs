pub struct TransactionRow {
    pub id: String,
    pub kind: String,
    pub label: String,
    pub amount_cents: i64,
    pub transaction_date: String,
    pub status: String,
    pub category_id: String,
    pub note: Option<String>,
    pub recurring_rule_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

pub struct CategoryRow {
    pub id: String,
    pub kind: String,
    pub name: String,
    pub icon: String,
    pub color: String,
    pub sort_order: i32,
    pub is_default: bool,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

pub struct SettingsRow {
    pub id: i64,
    pub current_balance_cents: i64,
    pub overdraft_limit_cents: i64,
    pub currency_code: String,
    pub locale: String,
    pub theme: String,
    pub balance_updated_at: String,
    pub onboarding_completed: bool,
    pub created_at: String,
    pub updated_at: String,
    pub last_export_date: Option<String>,
    pub ignored_update_version: Option<String>,
}
