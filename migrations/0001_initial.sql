CREATE TABLE IF NOT EXISTS categories (
    id TEXT PRIMARY KEY,
    kind TEXT NOT NULL CHECK(kind IN ('income', 'expense')),
    name TEXT NOT NULL,
    icon TEXT NOT NULL,
    color TEXT NOT NULL CHECK(length(color) = 7 AND substr(color, 1, 1) = '#'),
    sort_order INTEGER NOT NULL DEFAULT 0,
    is_default INTEGER NOT NULL DEFAULT 1,
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    UNIQUE(kind, name)
);

CREATE TABLE IF NOT EXISTS transactions (
    id TEXT PRIMARY KEY,
    kind TEXT NOT NULL CHECK(kind IN ('income', 'expense')),
    label TEXT NOT NULL,
    amount_cents INTEGER NOT NULL CHECK(amount_cents > 0),
    transaction_date TEXT NOT NULL,
    status TEXT NOT NULL CHECK(status IN ('pending', 'completed')),
    category_id TEXT NOT NULL REFERENCES categories(id),
    note TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS app_settings (
    id INTEGER PRIMARY KEY CHECK(id = 1),
    current_balance_cents INTEGER NOT NULL DEFAULT 0,
    overdraft_limit_cents INTEGER NOT NULL DEFAULT 0 CHECK(overdraft_limit_cents >= 0),
    currency_code TEXT NOT NULL DEFAULT 'EUR',
    locale TEXT NOT NULL DEFAULT 'fr-FR',
    theme TEXT NOT NULL DEFAULT 'system' CHECK(theme IN ('system', 'light', 'dark')),
    balance_updated_at TEXT NOT NULL,
    onboarding_completed INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_transactions_date ON transactions(transaction_date);
CREATE INDEX IF NOT EXISTS idx_transactions_kind_status ON transactions(kind, status);
CREATE INDEX IF NOT EXISTS idx_transactions_category ON transactions(category_id);
CREATE INDEX IF NOT EXISTS idx_categories_kind ON categories(kind);
