use chrono::NaiveDate;

use super::argent::Money;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TransactionKind {
    Income,
    Expense,
}

impl TransactionKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Income => "income",
            Self::Expense => "expense",
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "income" => Some(Self::Income),
            "expense" => Some(Self::Expense),
            _ => None,
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Income => "Revenu",
            Self::Expense => "Dépense",
        }
    }

    pub fn apply_to_balance(&self, balance: Money, amount: Money) -> Result<Money, String> {
        match self {
            Self::Income => balance
                .checked_add(amount)
                .ok_or_else(|| "Dépassement de montant dans le calcul du solde.".into()),
            Self::Expense => balance
                .checked_sub(amount)
                .ok_or_else(|| "Dépassement de montant dans le calcul du solde.".into()),
        }
    }
}

impl std::fmt::Display for TransactionKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Income => write!(f, "Revenu"),
            Self::Expense => write!(f, "Dépense"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TransactionStatus {
    Pending,
    Completed,
}

impl TransactionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Completed => "completed",
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "pending" => Some(Self::Pending),
            "completed" => Some(Self::Completed),
            _ => None,
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Pending => "En attente",
            Self::Completed => "Réalisé",
        }
    }
}

impl std::fmt::Display for TransactionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

pub type TransactionId = String;

#[derive(Debug, Clone)]
pub struct Transaction {
    pub id: TransactionId,
    pub kind: TransactionKind,
    pub label: String,
    pub amount: Money,
    pub transaction_date: NaiveDate,
    pub status: TransactionStatus,
    pub category_id: String,
    pub note: Option<String>,
    /// Règle récurrente qui a produit cette transaction, le cas échéant.
    pub recurring_rule_id: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl Transaction {
    /// Vrai si la transaction a été matérialisée par une règle récurrente.
    pub fn est_recurrente(&self) -> bool {
        self.recurring_rule_id.is_some()
    }

    pub fn signed_amount(&self) -> Result<Money, String> {
        match self.kind {
            TransactionKind::Income => Ok(self.amount),
            TransactionKind::Expense => self
                .amount
                .cents
                .checked_neg()
                .map(Money::from_cents)
                .ok_or_else(|| "Dépassement de montant.".to_string()),
        }
    }
}
