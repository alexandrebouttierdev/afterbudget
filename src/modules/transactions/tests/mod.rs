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

/// Une ligne SQLite invalide doit faire échouer le chargement avec un
/// message contextualisé, jamais être transformée en valeurs par défaut
/// (AB-008).
#[test]
fn une_ligne_invalide_fait_echouer_le_chargement() {
    let pool = base_temporaire::creer_base_test();
    // Les CHECK du schéma bloquent déjà kind/statut invalides à l'insertion ;
    // la date et les horodatages, eux, passent : c'est sur eux que le
    // mapping strict doit échouer (AB-008).
    pool.conn
        .execute(
            "INSERT INTO transactions (id, kind, label, amount_cents, transaction_date, status,
                category_id, note, created_at, updated_at)
             VALUES ('ligne-corrompue', 'expense', 'X', 1000, '2026-99-99', 'pending',
                'autre', NULL, 'pas-un-horodatage', '2026-08-01T00:00:00Z')",
            [],
        )
        .unwrap();

    // find_by_id n'applique pas de filtre de date : le WHERE du mois
    // écarterait la ligne corrompue (date « 2026-99-99 » hors de la plage)
    // avant qu'elle n'atteigne le mapping strict (AB-008).
    let erreur = repo::find_by_id(&pool, "ligne-corrompue").unwrap_err();
    assert!(
        erreur.contains("ligne-corrompue"),
        "l'erreur doit nommer la ligne : {erreur}"
    );
}

/// Une transaction ne peut référencer qu'une catégorie existante, active
/// et du même sens (AB-009).
#[test]
fn une_transaction_verifie_la_categorie() {
    let pool = base_temporaire::creer_base_test();

    // Catégorie income désactivée et catégorie expense active.
    let maintenant = chrono::Utc::now().to_rfc3339();
    pool.conn
        .execute(
            "INSERT INTO categories (id, kind, name, icon, color, sort_order, is_default, is_active, created_at, updated_at)
             VALUES ('income-inactive', 'income', 'Bonus inactif', 'x', '#3B82F6', 99, 0, 0, ?1, ?1)",
            rusqlite::params![&maintenant],
        )
        .unwrap();

    use crate::modules::transactions::service;
    use crate::domaine::argent::Money;

    assert!(service::creer_transaction(
        &pool,
        TransactionKind::Expense,
        "Courses",
        Money::from_cents(5000),
        chrono::NaiveDate::from_ymd_opt(2026, 8, 1).unwrap(),
        "alimentation",
        TransactionStatus::Pending,
        None,
    )
    .is_ok());

    // Catégorie inconnue.
    assert!(service::creer_transaction(
        &pool,
        TransactionKind::Expense,
        "Courses",
        Money::from_cents(5000),
        chrono::NaiveDate::from_ymd_opt(2026, 8, 1).unwrap(),
        "n-existe-pas",
        TransactionStatus::Pending,
        None,
    )
    .is_err());

    // Catégorie inactive.
    assert!(service::creer_transaction(
        &pool,
        TransactionKind::Income,
        "Bonus",
        Money::from_cents(5000),
        chrono::NaiveDate::from_ymd_opt(2026, 8, 1).unwrap(),
        "income-inactive",
        TransactionStatus::Pending,
        None,
    )
    .is_err());

    // Catégorie du mauvais sens.
    assert!(service::creer_transaction(
        &pool,
        TransactionKind::Expense,
        "Bonus",
        Money::from_cents(5000),
        chrono::NaiveDate::from_ymd_opt(2026, 8, 1).unwrap(),
        "salaire",
        TransactionStatus::Pending,
        None,
    )
    .is_err());

    // Rien d'autre n'a été persisté.
    let lignes = repo::find_all_by_month(&pool, 2026, 8).unwrap();
    assert_eq!(lignes.len(), 1);
}

/// Le comptage SQL doit recouper le chargement complet : les statistiques
/// ne doivent plus charger toutes les lignes (AB-011).
#[test]
fn le_comptage_sql_recoupe_le_chargement() {
    let pool = base_temporaire::creer_base_test();
    for i in 0..50 {
        let kind = if i % 2 == 0 {
            TransactionKind::Income
        } else {
            TransactionKind::Expense
        };
        let tx = donnees_test::transaction_test(
            &format!("Opération {i}"),
            1000,
            kind,
            TransactionStatus::Pending,
            "2026-08-01",
            if kind == TransactionKind::Income { "salaire" } else { "autre" },
        );
        repo::insert(&pool, &tx).unwrap();
    }
    let lignes = repo::find_all_by_month(&pool, 2026, 8).unwrap();
    let par_kind = |k: &str| repo::count_by_kind(&pool, 2026, 8, k).unwrap();
    assert_eq!(par_kind("income") as usize, lignes.iter().filter(|t| t.kind == TransactionKind::Income).count());
    assert_eq!(par_kind("expense") as usize, lignes.iter().filter(|t| t.kind == TransactionKind::Expense).count());
}

/// Benchmark volumineux : 100 000 transactions dans le mois. Hors CI par
/// défaut (`cargo test -- --ignored`) ; seuil large pour rester stable.
#[test]
#[ignore = "benchmark volumineux : lancer avec cargo test -- --ignored"]
fn perf_grosses_bases() {
    let pool = base_temporaire::creer_base_test();
    let maintenant = "2026-08-01T00:00:00Z";

    {
        let tx = pool.conn.unchecked_transaction().expect("transaction");
        {
            let mut stmt = tx
                .prepare(
                    "INSERT INTO transactions (id, kind, label, amount_cents, transaction_date, status,
                        category_id, note, recurring_rule_id, created_at, updated_at)
                     VALUES (?1, 'expense', ?2, ?3, '2026-08-15', 'pending',
                        'autre', NULL, NULL, ?4, ?4)",
                )
                .expect("préparation");
            for i in 0..100_000i64 {
                stmt.execute(rusqlite::params![
                    uuid::Uuid::new_v4().to_string(),
                    format!("Dépense {i}"),
                    100 + (i % 5000),
                    maintenant,
                ])
                .expect("insertion");
            }
        }
        tx.commit().expect("commit");
    }

    let debut_lecture = std::time::Instant::now();
    let lignes = repo::find_all_by_month(&pool, 2026, 8).expect("lecture");
    let duree_lecture = debut_lecture.elapsed();
    assert_eq!(lignes.len(), 100_000);
    assert!(
        duree_lecture.as_secs() < 5,
        "lecture trop lente : {duree_lecture:?}"
    );

    let debut_stats = std::time::Instant::now();
    let stats = crate::modules::statistiques::service::calculer_statistiques_mensuelles(
        &pool, 2026, 8,
    )
    .expect("statistiques");
    let duree_stats = debut_stats.elapsed();
    assert_eq!(stats.transaction_count, 100_000);
    assert!(
        duree_stats.as_secs() < 5,
        "statistiques trop lentes : {duree_stats:?}"
    );
}
