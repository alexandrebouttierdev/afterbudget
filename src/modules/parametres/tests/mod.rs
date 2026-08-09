use crate::domaine::argent::Money;
use crate::modules::parametres::service;

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
    assert_eq!(relu.current_balance.cents, 10000, "la valeur doit rester inchangée");
}

/// Mettre à jour les paramètres d'une base sans ligne id=1 échoue (AB-001).
#[test]
fn mettre_a_jour_sans_ligne_de_parametres_echoue() {
    let pool = base_temporaire::creer_base_test();
    let parametres = crate::domaine::parametres::AppSettings {
        current_balance: Money::from_cents(1),
        ..Default::default()
    };
    assert!(
        crate::modules::parametres::repository::update_settings(&pool, &parametres).is_err()
    );
}
