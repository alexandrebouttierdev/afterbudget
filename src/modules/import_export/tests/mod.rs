//! Tests d'intégration pour le module import_export (AB-002, AB-003).

use crate::core::db::migrations;
use crate::core::db::pool::DatabasePool;
use crate::modules::import_export::service;

fn ecrire_base_v1(chemin: &std::path::Path) {
    let conn = rusqlite::Connection::open(chemin).unwrap();
    conn.execute_batch("CREATE TABLE IF NOT EXISTS schema_migrations (
        version INTEGER PRIMARY KEY,
        name TEXT NOT NULL,
        applied_at TEXT NOT NULL
    );")
    .unwrap();
    conn.execute_batch(include_str!("../../../../migrations/0001_initial.sql"))
        .unwrap();
    conn.execute(
        "INSERT INTO schema_migrations VALUES (1, 'v1', ?1)",
        rusqlite::params![chrono::Utc::now().to_rfc3339()],
    )
    .unwrap();
}

fn ecrire_base_v2(chemin: &std::path::Path) {
    ecrire_base_v1(chemin);
    let conn = rusqlite::Connection::open(chemin).unwrap();
    conn.execute_batch(
        "ALTER TABLE transactions ADD COLUMN recurring_rule_id TEXT REFERENCES recurring_rules(id);",
    )
    .unwrap();
    conn.execute_batch(include_str!("../../../../migrations/0002_recurrences.sql"))
        .unwrap();
    conn.execute(
        "INSERT INTO schema_migrations VALUES (2, 'v2', ?1)",
        rusqlite::params![chrono::Utc::now().to_rfc3339()],
    )
    .unwrap();
}

fn somme_checksum(chemin: &std::path::Path) -> u64 {
    std::fs::read(chemin)
        .unwrap()
        .iter()
        .fold(0u64, |acc, octet| acc.wrapping_add(*octet as u64))
}

#[test]
fn une_base_v1_et_une_base_v2_sont_acceptees() {
    let repertoire = tempfile::tempdir().unwrap();

    let v1 = repertoire.path().join("v1.sqlite");
    ecrire_base_v1(&v1);
    assert!(service::valider_import(&v1).is_ok(), "v1 doit être acceptée");

    let v2 = repertoire.path().join("v2.sqlite");
    ecrire_base_v2(&v2);
    assert!(service::valider_import(&v2).is_ok(), "v2 doit être acceptée");
}

#[test]
fn une_version_future_est_refusee() {
    let repertoire = tempfile::tempdir().unwrap();
    let chemin = repertoire.path().join("future.sqlite");
    ecrire_base_v2(&chemin);
    let conn = rusqlite::Connection::open(&chemin).unwrap();
    conn.execute(
        "INSERT INTO schema_migrations VALUES (99, 'v99', ?1)",
        rusqlite::params![chrono::Utc::now().to_rfc3339()],
    )
    .unwrap();
    drop(conn);

    let erreur = service::valider_import(&chemin).unwrap_err();
    assert!(erreur.contains("99"), "message attendu : {erreur}");
}

#[test]
fn un_fichier_non_sqlite_est_refuse() {
    let repertoire = tempfile::tempdir().unwrap();
    let chemin = repertoire.path().join("faux.sqlite");
    std::fs::write(&chemin, "pas une base du tout, juste du texte.").unwrap();
    assert!(service::valider_import(&chemin).is_err());
}

/// Un import réussi remplace la base ; un import échoué laisse la base
/// d'origine intacte (AB-003).
#[test]
fn un_import_reussi_remplace_et_un_import_echoue_preserve() {
    let repertoire = tempfile::tempdir().unwrap();

    // Base courante avec un paramètre.
    let cible = repertoire.path().join("cible.sqlite");
    {
        let pool = DatabasePool::open(&cible).unwrap();
        migrations::run_migrations(&pool.conn).unwrap();
        let parametres = crate::domaine::parametres::AppSettings {
            current_balance: crate::domaine::argent::Money::from_cents(11111),
            ..Default::default()
        };
        crate::modules::parametres::repository::insert_settings(&pool, &parametres).unwrap();
    }

    // Source valide avec une autre valeur.
    let source = repertoire.path().join("source.sqlite");
    {
        let pool = DatabasePool::open(&source).unwrap();
        migrations::run_migrations(&pool.conn).unwrap();
        let parametres = crate::domaine::parametres::AppSettings {
            current_balance: crate::domaine::argent::Money::from_cents(22222),
            ..Default::default()
        };
        crate::modules::parametres::repository::insert_settings(&pool, &parametres).unwrap();
    }

    service::importer_en_arriere_plan(cible.clone(), source.clone()).unwrap();
    let lue = crate::modules::parametres::repository::get_settings(
        &DatabasePool::open(&cible).unwrap(),
    )
    .unwrap()
    .unwrap();
    assert_eq!(lue.current_balance.cents, 22222, "les données importées doivent remplacer");

    // Échec : source corrompue (fichier tronqué). Le checksum de référence est
    // capturé APRÈS l'import réussi, sur l'état que l'échec ne doit pas toucher.
    let corrompue = repertoire.path().join("corrompue.sqlite");
    std::fs::write(&corrompue, b"SQLite format 3\0").unwrap();
    let avant = somme_checksum(&cible);
    assert!(service::importer_en_arriere_plan(cible.clone(), corrompue).is_err());
    let apres = somme_checksum(&cible);
    assert_eq!(avant, apres, "la base d'origine doit rester intacte");
}
