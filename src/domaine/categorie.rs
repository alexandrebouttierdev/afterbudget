use super::transaction::TransactionKind;
use std::fmt;

pub type CategoryId = String;

#[derive(Debug, Clone, PartialEq)]
pub struct Category {
    pub id: CategoryId,
    pub kind: TransactionKind,
    pub name: String,
    pub icon: String,
    pub color: String,
    pub sort_order: i32,
    pub is_default: bool,
    pub is_active: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl fmt::Display for Category {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[derive(Debug, Clone)]
pub struct DefaultCategory {
    pub id: &'static str,
    pub name: &'static str,
    pub icon: &'static str,
    pub color: &'static str,
    pub kind: TransactionKind,
}

impl DefaultCategory {
    pub const fn new(
        id: &'static str,
        name: &'static str,
        icon: &'static str,
        color: &'static str,
        kind: TransactionKind,
    ) -> Self {
        Self {
            id,
            name,
            icon,
            color,
            kind,
        }
    }
}

pub const DEFAULT_EXPENSE_CATEGORIES: &[DefaultCategory] = &[
    DefaultCategory::new(
        "logement",
        "Logement",
        "Home",
        "#3B82F6",
        TransactionKind::Expense,
    ),
    DefaultCategory::new(
        "alimentation",
        "Alimentation",
        "ShoppingCart",
        "#22C55E",
        TransactionKind::Expense,
    ),
    DefaultCategory::new(
        "transport",
        "Transport",
        "Car",
        "#EAB308",
        TransactionKind::Expense,
    ),
    DefaultCategory::new(
        "sante",
        "Santé",
        "Heart",
        "#EF4444",
        TransactionKind::Expense,
    ),
    DefaultCategory::new(
        "loisirs",
        "Loisirs",
        "Gamepad2",
        "#A855F7",
        TransactionKind::Expense,
    ),
    DefaultCategory::new(
        "vetements",
        "Vêtements",
        "Shirt",
        "#EC4899",
        TransactionKind::Expense,
    ),
    DefaultCategory::new(
        "abonnements",
        "Abonnements",
        "RefreshCw",
        "#6366F1",
        TransactionKind::Expense,
    ),
    DefaultCategory::new(
        "restaurants",
        "Restaurants",
        "Utensils",
        "#F97316",
        TransactionKind::Expense,
    ),
    DefaultCategory::new(
        "voyages",
        "Voyages",
        "Plane",
        "#06B6D4",
        TransactionKind::Expense,
    ),
    DefaultCategory::new(
        "education",
        "Éducation",
        "GraduationCap",
        "#14B8A6",
        TransactionKind::Expense,
    ),
    DefaultCategory::new(
        "epargne",
        "Épargne",
        "PiggyBank",
        "#10B981",
        TransactionKind::Expense,
    ),
    DefaultCategory::new(
        "energie",
        "Énergie",
        "Zap",
        "#F59E0B",
        TransactionKind::Expense,
    ),
    DefaultCategory::new(
        "internet",
        "Internet",
        "Wifi",
        "#0EA5E9",
        TransactionKind::Expense,
    ),
    DefaultCategory::new(
        "telephone",
        "Téléphone",
        "Smartphone",
        "#8B5CF6",
        TransactionKind::Expense,
    ),
    DefaultCategory::new(
        "sport",
        "Sport",
        "Dumbbell",
        "#84CC16",
        TransactionKind::Expense,
    ),
    DefaultCategory::new(
        "musique",
        "Musique",
        "Music",
        "#F43F5E",
        TransactionKind::Expense,
    ),
    DefaultCategory::new(
        "livres",
        "Livres",
        "BookOpen",
        "#78716C",
        TransactionKind::Expense,
    ),
    DefaultCategory::new(
        "enfants",
        "Enfants",
        "Baby",
        "#D946EF",
        TransactionKind::Expense,
    ),
    DefaultCategory::new(
        "animaux",
        "Animaux",
        "PawPrint",
        "#CA8A04",
        TransactionKind::Expense,
    ),
    DefaultCategory::new(
        "bricolage",
        "Bricolage",
        "Wrench",
        "#6B7280",
        TransactionKind::Expense,
    ),
    DefaultCategory::new(
        "cadeaux",
        "Cadeaux",
        "Gift",
        "#F472B6",
        TransactionKind::Expense,
    ),
    DefaultCategory::new(
        "bien-etre",
        "Bien-être",
        "Sparkles",
        "#C084FC",
        TransactionKind::Expense,
    ),
    DefaultCategory::new(
        "credit",
        "Crédit",
        "CreditCard",
        "#FB923C",
        TransactionKind::Expense,
    ),
    DefaultCategory::new(
        "tabac",
        "Tabac",
        "Cigarette",
        "#A8A29E",
        TransactionKind::Expense,
    ),
    DefaultCategory::new(
        "assurance",
        "Assurance",
        "Shield",
        "#38BDF8",
        TransactionKind::Expense,
    ),
    DefaultCategory::new(
        "autre",
        "Autre",
        "MoreHorizontal",
        "#9CA3AF",
        TransactionKind::Expense,
    ),
];

pub const DEFAULT_INCOME_CATEGORIES: &[DefaultCategory] = &[
    DefaultCategory::new(
        "salaire",
        "Salaire",
        "Wallet",
        "#22C55E",
        TransactionKind::Income,
    ),
    DefaultCategory::new(
        "allocation",
        "Allocation",
        "HandCoins",
        "#10B981",
        TransactionKind::Income,
    ),
    DefaultCategory::new("prime", "Prime", "Star", "#F59E0B", TransactionKind::Income),
    DefaultCategory::new(
        "remboursement",
        "Remboursement",
        "Undo2",
        "#3B82F6",
        TransactionKind::Income,
    ),
    DefaultCategory::new("vente", "Vente", "Tag", "#EC4899", TransactionKind::Income),
    DefaultCategory::new(
        "pension",
        "Pension",
        "HeartHandshake",
        "#A855F7",
        TransactionKind::Income,
    ),
    DefaultCategory::new(
        "freelance",
        "Revenu indépendant",
        "Briefcase",
        "#6366F1",
        TransactionKind::Income,
    ),
    DefaultCategory::new(
        "interets",
        "Intérêts",
        "TrendingUp",
        "#14B8A6",
        TransactionKind::Income,
    ),
    DefaultCategory::new(
        "autre-revenu",
        "Autre",
        "MoreHorizontal",
        "#9CA3AF",
        TransactionKind::Income,
    ),
];
