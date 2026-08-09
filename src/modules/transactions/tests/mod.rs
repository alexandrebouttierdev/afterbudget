use crate::domaine::transaction::{TransactionKind, TransactionStatus};
use crate::modules::transactions::repository as repo;

mod commun;
use commun::{base_temporaire, donnees_test};

#[test]
fn test_creer_et_lire_transaction() {
    let pool = base_temporaire::creer_base_test();
    let tx = donnees_test::transaction_test(
        "Loyer",
        85000,
        TransactionKind::Expense,
        TransactionStatus::Pending,
        "2026-08-01",
        "logement",
    );
    repo::insert(&pool, &tx).unwrap();
    let found = repo::find_by_id(&pool, &tx.id).unwrap().unwrap();
    assert_eq!(found.label, "Loyer");
    assert_eq!(found.amount.cents, 85000);
}

#[test]
fn test_modifier_transaction() {
    let pool = base_temporaire::creer_base_test();
    let mut tx = donnees_test::transaction_test(
        "Courses",
        5000,
        TransactionKind::Expense,
        TransactionStatus::Pending,
        "2026-08-01",
        "alimentation",
    );
    repo::insert(&pool, &tx).unwrap();
    tx.label = "Courses modifiées".into();
    tx.amount = crate::domaine::argent::Money::from_cents(6000);
    repo::update(&pool, &tx).unwrap();
    let found = repo::find_by_id(&pool, &tx.id).unwrap().unwrap();
    assert_eq!(found.label, "Courses modifiées");
    assert_eq!(found.amount.cents, 6000);
}

#[test]
fn test_supprimer_transaction() {
    let pool = base_temporaire::creer_base_test();
    let tx = donnees_test::transaction_test(
        "Test",
        1000,
        TransactionKind::Expense,
        TransactionStatus::Pending,
        "2026-08-01",
        "autre",
    );
    repo::insert(&pool, &tx).unwrap();
    repo::delete_by_id(&pool, &tx.id).unwrap();
    assert!(repo::find_by_id(&pool, &tx.id).unwrap().is_none());
}

#[test]
fn test_filtre_par_mois() {
    let pool = base_temporaire::creer_base_test();
    let tx1 = donnees_test::transaction_test(
        "Août",
        10000,
        TransactionKind::Expense,
        TransactionStatus::Pending,
        "2026-08-15",
        "autre",
    );
    let tx2 = donnees_test::transaction_test(
        "Septembre",
        20000,
        TransactionKind::Expense,
        TransactionStatus::Pending,
        "2026-09-01",
        "autre",
    );
    repo::insert(&pool, &tx1).unwrap();
    repo::insert(&pool, &tx2).unwrap();
    let aout = repo::find_all_by_month(&pool, 2026, 8).unwrap();
    let sept = repo::find_all_by_month(&pool, 2026, 9).unwrap();
    assert_eq!(aout.len(), 1);
    assert_eq!(sept.len(), 1);
}

#[test]
fn test_somme_pending() {
    let pool = base_temporaire::creer_base_test();
    let tx1 = donnees_test::transaction_test(
        "P1",
        10000,
        TransactionKind::Expense,
        TransactionStatus::Pending,
        "2026-08-01",
        "autre",
    );
    let tx2 = donnees_test::transaction_test(
        "P2",
        20000,
        TransactionKind::Expense,
        TransactionStatus::Completed,
        "2026-08-01",
        "autre",
    );
    repo::insert(&pool, &tx1).unwrap();
    repo::insert(&pool, &tx2).unwrap();
    let sum = repo::sum_pending_by_kind(&pool, 2026, 8, "expense").unwrap();
    assert_eq!(sum, 10000);
}

#[test]
fn test_contrainte_cle_etrangere() {
    let pool = base_temporaire::creer_base_test();
    let tx = donnees_test::transaction_test(
        "Test",
        1000,
        TransactionKind::Expense,
        TransactionStatus::Pending,
        "2026-08-01",
        "categorie_inexistante",
    );
    assert!(repo::insert(&pool, &tx).is_err());
}

/// Modifier ou supprimer une transaction inexistante doit échouer : un
/// succès silencieux ferait croire à une persistance qui n'a pas eu lieu
/// (AB-001, AB-009).
#[test]
fn modifier_une_transaction_absente_echoue() {
    let pool = base_temporaire::creer_base_test();
    let tx = donnees_test::transaction_test(
        "Fantôme",
        1000,
        TransactionKind::Expense,
        TransactionStatus::Pending,
        "2026-08-01",
        "autre",
    );
    assert!(repo::update(&pool, &tx).is_err());
    assert!(repo::delete_by_id(&pool, "id-inexistant").is_err());
}
