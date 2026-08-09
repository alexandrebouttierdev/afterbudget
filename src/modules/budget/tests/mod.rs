use crate::core::db::pool::DatabasePool;
use crate::domaine::argent::Money;
use crate::domaine::parametres::AppSettings;
use crate::domaine::transaction::{TransactionKind, TransactionStatus};
use crate::modules::budget::service as budget_service;
use crate::modules::parametres::repository as settings_repo;
use crate::modules::transactions::repository as tx_repo;

mod commun;
use commun::{base_temporaire, donnees_test};

fn setup_scenario(pool: &DatabasePool) {
    let settings = AppSettings {
        current_balance: Money::from_cents(-36000),
        overdraft_limit: Money::from_cents(50000),
        onboarding_completed: true,
        ..Default::default()
    };
    settings_repo::insert_settings(pool, &settings).unwrap();

    let revenu = donnees_test::transaction_test(
        "Salaire",
        120700,
        TransactionKind::Income,
        TransactionStatus::Pending,
        "2026-08-01",
        "salaire",
    );
    let loyer = donnees_test::transaction_test(
        "Loyer",
        85000,
        TransactionKind::Expense,
        TransactionStatus::Pending,
        "2026-08-01",
        "logement",
    );
    let courses = donnees_test::transaction_test(
        "Courses",
        12000,
        TransactionKind::Expense,
        TransactionStatus::Pending,
        "2026-08-01",
        "alimentation",
    );
    let assurance = donnees_test::transaction_test(
        "Assurance",
        15500,
        TransactionKind::Expense,
        TransactionStatus::Pending,
        "2026-08-01",
        "assurance",
    );
    let energie = donnees_test::transaction_test(
        "Énergie",
        20000,
        TransactionKind::Expense,
        TransactionStatus::Pending,
        "2026-08-01",
        "energie",
    );
    tx_repo::insert(pool, &revenu).unwrap();
    tx_repo::insert(pool, &loyer).unwrap();
    tx_repo::insert(pool, &courses).unwrap();
    tx_repo::insert(pool, &assurance).unwrap();
    tx_repo::insert(pool, &energie).unwrap();
}

#[test]
fn test_parcours_budget_complet() {
    let pool = base_temporaire::creer_base_test();
    setup_scenario(&pool);

    let budget = budget_service::calculer_resume_budget(&pool, 2026, 8).unwrap();

    assert_eq!(budget.current_balance.cents, -36000);
    assert_eq!(budget.pending_income.cents, 120700);
    assert_eq!(budget.pending_expenses.cents, 85000 + 12000 + 15500 + 20000);

    assert_eq!(budget.projected_balance.cents, -47800);
    assert_eq!(budget.remaining_overdraft_margin.cents, 2200);

    use crate::domaine::budget::FinancialStatus;
    assert_eq!(budget.financial_status, FinancialStatus::Warning);
}

#[test]
fn test_persistance_donnees() {
    let pool = base_temporaire::creer_base_test();
    setup_scenario(&pool);

    let revenus = tx_repo::find_all_by_month(&pool, 2026, 8).unwrap();
    assert_eq!(revenus.len(), 5);

    let settings = settings_repo::get_settings(&pool).unwrap().unwrap();
    assert_eq!(settings.current_balance.cents, -36000);
}
