-- Règles d'opérations récurrentes.
--
-- Une règle décrit une opération qui revient chaque mois (loyer, salaire,
-- abonnement). Elle ne remplace pas les transactions : à l'ouverture d'un mois,
-- chaque règle qui le couvre y matérialise une transaction « en attente », que
-- l'utilisateur reste libre de modifier ou de supprimer ligne par ligne.
CREATE TABLE IF NOT EXISTS recurring_rules (
    id TEXT PRIMARY KEY,
    kind TEXT NOT NULL CHECK(kind IN ('income', 'expense')),
    label TEXT NOT NULL,
    amount_cents INTEGER NOT NULL CHECK(amount_cents > 0),
    category_id TEXT NOT NULL REFERENCES categories(id),
    day_of_month INTEGER NOT NULL CHECK(day_of_month BETWEEN 1 AND 31),
    start_year INTEGER NOT NULL,
    start_month INTEGER NOT NULL CHECK(start_month BETWEEN 1 AND 12),
    end_year INTEGER,
    end_month INTEGER CHECK(end_month IS NULL OR end_month BETWEEN 1 AND 12),
    note TEXT,
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Rattachement d'une transaction à la règle qui l'a produite. Sert de garde-fou
-- anti-doublon : une règle ne matérialise qu'une occurrence par mois.
ALTER TABLE transactions ADD COLUMN recurring_rule_id TEXT REFERENCES recurring_rules(id);

CREATE INDEX IF NOT EXISTS idx_transactions_recurring ON transactions(recurring_rule_id);
CREATE INDEX IF NOT EXISTS idx_recurring_active ON recurring_rules(is_active);
