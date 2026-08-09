use crate::core::db::pool::DatabasePool;
use crate::domaine::argent::Money;
use crate::domaine::parametres::AppSettings;
use crate::modules::parametres::repository as repo;
use crate::modules::parametres::service;
use crate::modules::recurrences::repository as regles_repo;
use crate::modules::transactions::repository as tx_repo;

mod commun;
use commun::base_temporaire;

/// Une connexion non inscriptible ne doit pas produire un faux succès : la
/// mise à jour échoue proprement et la valeur en base reste inchangée
/// (AB-001).
#[test]
fn une_connexion_non_inscriptible_echoue_sans_effet() {
    let pool = base_temporaire::creer_base_test();
    let initial = service::obtenir_parametres(&pool).unwrap();
    let _ = initial; // pas encore de ligne : insérons-en une.

    use crate::domaine::parametres::AppSettings;
    let parametres = AppSettings {
        current_balance: Money::from_cents(10000),
        ..Default::default()
    };
    crate::modules::parametres::repository::insert_settings(&pool, &parametres).unwrap();

    pool.conn
        .execute_batch("PRAGMA query_only = ON;")
        .expect("pragma");

    let erreur = service::mettre_a_jour_solde(&pool, Money::from_cents(5000));
    assert!(erreur.is_err(), "l'écriture devait échouer");

    pool.conn
        .execute_batch("PRAGMA query_only = OFF;")
        .expect("pragma");
    let relu = service::obtenir_parametres(&pool).unwrap().unwrap();
    assert_eq!(
        relu.current_balance.cents, 10000,
        "la valeur doit rester inchangée"
    );
}

/// Mettre à jour les paramètres d'une base sans ligne id=1 échoue (AB-001).
#[test]
fn mettre_a_jour_sans_ligne_de_parametres_echoue() {
    let pool = base_temporaire::creer_base_test();
    let parametres = crate::domaine::parametres::AppSettings {
        current_balance: Money::from_cents(1),
        ..Default::default()
    };
    assert!(crate::modules::parametres::repository::update_settings(&pool, &parametres).is_err());
}

/// La date du dernier export doit survivre au redémarrage (AB-012/P2).
#[test]
fn la_date_du_dernier_export_survit_a_la_reouverture() {
    let repertoire = tempfile::tempdir().unwrap();
    let chemin = repertoire.path().join("base.sqlite");

    let pool = DatabasePool::open(&chemin).unwrap();
    crate::core::db::migrations::run_migrations(&pool.conn).unwrap();
    let parametres = AppSettings {
        current_balance: Money::from_cents(10000),
        ..Default::default()
    };
    repo::insert_settings(&pool, &parametres).unwrap();
    service::marquer_dernier_export(&pool).unwrap();
    drop(pool);

    let relu = DatabasePool::open(&chemin).unwrap();
    let parametres = repo::get_settings(&relu).unwrap().unwrap();
    assert!(
        parametres.last_export_date.is_some(),
        "la date doit être persistée"
    );
}

fn inserer_scenario_avec_regle(pool: &DatabasePool) {
    // Catégorie personnalisée référencée par une règle ET une transaction :
    // le scénario exact de l'audit (AB-004).
    let maintenant = chrono::Utc::now().to_rfc3339();
    pool.conn
        .execute(
            "INSERT INTO categories (id, kind, name, icon, color, sort_order, is_default, is_active, created_at, updated_at)
             VALUES ('perso', 'expense', 'Perso', 'x', '#3B82F6', 99, 0, 1, ?1, ?1)",
            rusqlite::params![&maintenant],
        )
        .unwrap();

    let regle = crate::domaine::recurrence::RecurringRule {
        id: "regle-1".into(),
        kind: crate::domaine::transaction::TransactionKind::Expense,
        label: "Abonnement".into(),
        amount: Money::from_cents(99900),
        category_id: "perso".into(),
        day_of_month: 5,
        start: (2026, 8),
        end: None,
        note: None,
        is_active: true,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    regles_repo::creer(pool, &regle).unwrap();

    let tx = crate::tests_commun::donnees_test::transaction_test(
        "Perso achat",
        5000,
        crate::domaine::transaction::TransactionKind::Expense,
        crate::domaine::transaction::TransactionStatus::Pending,
        "2026-08-01",
        "perso",
    );
    tx_repo::insert(pool, &tx).unwrap();

    let parametres = AppSettings {
        current_balance: Money::from_cents(10000),
        ..Default::default()
    };
    repo::insert_settings(pool, &parametres).unwrap();
}

fn compter(pool: &DatabasePool, table: &str) -> i64 {
    pool.conn
        .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |r| r.get(0))
        .unwrap()
}

/// Le reset efface tout : transactions, paramètres, règles et catégories
/// personnalisées, dans une seule transaction (AB-004).
#[test]
fn le_reset_efface_tout_y_compris_les_regles() {
    let pool = base_temporaire::creer_base_test();
    inserer_scenario_avec_regle(&pool);

    repo::reset_all_data(&pool).unwrap();

    assert_eq!(compter(&pool, "transactions"), 0);
    assert_eq!(compter(&pool, "app_settings"), 0);
    assert_eq!(compter(&pool, "recurring_rules"), 0);
    // 35 catégories par défaut (26 dépenses + 9 revenus) ; aucune personnalisée.
    assert_eq!(compter(&pool, "categories"), 35);
}

/// Sur une base non inscriptible, le reset échoue sans rien effacer (AB-004).
#[test]
fn le_reset_echoue_sans_effet_sur_base_non_inscriptible() {
    let pool = base_temporaire::creer_base_test();
    inserer_scenario_avec_regle(&pool);

    pool.conn.execute_batch("PRAGMA query_only = ON;").unwrap();
    assert!(repo::reset_all_data(&pool).is_err());
    pool.conn.execute_batch("PRAGMA query_only = OFF;").unwrap();

    assert_eq!(compter(&pool, "transactions"), 1);
    assert_eq!(compter(&pool, "recurring_rules"), 1);
    assert_eq!(compter(&pool, "app_settings"), 1);
}
