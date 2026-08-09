//! Tests de la matérialisation des règles récurrentes.

use crate::core::db::migrations;
use crate::core::db::pool::DatabasePool;
use crate::domaine::transaction::{TransactionKind, TransactionStatus};
use crate::modules::recurrences::commandes;
use crate::modules::recurrences::dtos::CreerRecurrenceDto;
use crate::modules::transactions::repository as transactions_repo;

fn base() -> DatabasePool {
    let pool = DatabasePool::open_in_memory().expect("base mémoire");
    migrations::run_migrations(&pool.conn).expect("migrations");
    pool
}

fn dto(libelle: &str, jour: u32, annee: i32, mois: u32) -> CreerRecurrenceDto {
    CreerRecurrenceDto {
        libelle: libelle.into(),
        montant: "890".into(),
        categorie_id: "logement".into(),
        type_transaction: "expense".into(),
        jour_du_mois: jour,
        debut_annee: annee,
        debut_mois: mois,
        note: None,
    }
}

fn transactions_du_mois(
    pool: &DatabasePool,
    annee: i32,
    mois: u32,
) -> Vec<crate::domaine::transaction::Transaction> {
    transactions_repo::find_by_month(pool, annee, mois, None, None, None, None).expect("lecture")
}

#[test]
fn une_regle_materialise_une_occurrence_dans_le_mois() {
    let pool = base();
    commandes::creer(&pool, &dto("Loyer", 3, 2026, 8)).expect("création");

    let creees = commandes::generer_pour_mois(&pool, 2026, 8).expect("génération");
    assert_eq!(creees, 1);

    let lignes = transactions_du_mois(&pool, 2026, 8);
    assert_eq!(lignes.len(), 1);
    assert_eq!(lignes[0].label, "Loyer");
    assert_eq!(lignes[0].kind, TransactionKind::Expense);
    assert_eq!(lignes[0].status, TransactionStatus::Pending);
    assert_eq!(lignes[0].transaction_date.to_string(), "2026-08-03");
}

/// La génération est rejouée à chaque ouverture du mois : elle doit rester
/// sans effet la seconde fois. C'est le garde-fou anti-doublon.
#[test]
fn rejouer_la_generation_ne_cree_pas_de_doublon() {
    let pool = base();
    commandes::creer(&pool, &dto("Loyer", 3, 2026, 8)).expect("création");

    assert_eq!(commandes::generer_pour_mois(&pool, 2026, 8).unwrap(), 1);
    assert_eq!(commandes::generer_pour_mois(&pool, 2026, 8).unwrap(), 0);
    assert_eq!(commandes::generer_pour_mois(&pool, 2026, 8).unwrap(), 0);

    assert_eq!(transactions_du_mois(&pool, 2026, 8).len(), 1);
}

/// Chaque mois couvert reçoit sa propre occurrence, et une seule.
#[test]
fn chaque_mois_couvert_recoit_son_occurrence() {
    let pool = base();
    commandes::creer(&pool, &dto("Loyer", 3, 2026, 8)).expect("création");

    for (annee, mois) in [(2026, 8), (2026, 9), (2026, 10), (2027, 1)] {
        assert_eq!(commandes::generer_pour_mois(&pool, annee, mois).unwrap(), 1);
        assert_eq!(transactions_du_mois(&pool, annee, mois).len(), 1);
    }
}

/// Aucune occurrence avant le mois de départ de la règle.
#[test]
fn aucune_occurrence_avant_le_debut_de_la_regle() {
    let pool = base();
    commandes::creer(&pool, &dto("Loyer", 3, 2026, 8)).expect("création");

    assert_eq!(commandes::generer_pour_mois(&pool, 2026, 7).unwrap(), 0);
    assert!(transactions_du_mois(&pool, 2026, 7).is_empty());
}

/// Une règle au 31 tombe le dernier jour des mois plus courts.
#[test]
fn le_jour_est_reporte_dans_les_mois_courts() {
    let pool = base();
    commandes::creer(&pool, &dto("Abonnement", 31, 2026, 1)).expect("création");

    commandes::generer_pour_mois(&pool, 2026, 2).expect("génération");
    let lignes = transactions_du_mois(&pool, 2026, 2);
    assert_eq!(lignes[0].transaction_date.to_string(), "2026-02-28");
}

/// Supprimer une règle ne doit pas effacer l'historique déjà produit : les
/// transactions restent, simplement détachées.
#[test]
fn supprimer_une_regle_conserve_les_occurrences_produites() {
    let pool = base();
    let regle = commandes::creer(&pool, &dto("Loyer", 3, 2026, 8)).expect("création");
    commandes::generer_pour_mois(&pool, 2026, 8).expect("génération");

    commandes::supprimer(&pool, &regle.id).expect("suppression");

    assert_eq!(transactions_du_mois(&pool, 2026, 8).len(), 1);
    assert!(commandes::lister(&pool).unwrap().is_empty());
    // La règle disparue, le mois ne régénère plus rien.
    assert_eq!(commandes::generer_pour_mois(&pool, 2026, 8).unwrap(), 0);
}

/// Plusieurs règles coexistent sans se gêner.
#[test]
fn plusieurs_regles_coexistent() {
    let pool = base();
    commandes::creer(&pool, &dto("Loyer", 3, 2026, 8)).expect("création");
    let mut salaire = dto("Salaire", 28, 2026, 8);
    salaire.type_transaction = "income".into();
    salaire.categorie_id = "salaire".into();
    salaire.montant = "2480".into();
    commandes::creer(&pool, &salaire).expect("création");

    assert_eq!(commandes::generer_pour_mois(&pool, 2026, 8).unwrap(), 2);

    let lignes = transactions_du_mois(&pool, 2026, 8);
    assert_eq!(lignes.len(), 2);
    assert_eq!(
        lignes
            .iter()
            .filter(|t| t.kind == TransactionKind::Income)
            .count(),
        1
    );
}

/// Une règle invalide est refusée avant d'atteindre la base.
#[test]
fn une_regle_invalide_est_refusee() {
    let pool = base();
    let mut invalide = dto("Loyer", 45, 2026, 8);
    invalide.montant = "0".into();

    assert!(commandes::creer(&pool, &invalide).is_err());
    assert!(commandes::lister(&pool).unwrap().is_empty());
}

/// Supprimer une règle inexistante doit échouer (AB-001, AB-009).
#[test]
fn supprimer_une_regle_absente_echoue() {
    let pool = base();
    assert!(commandes::supprimer(&pool, "id-inexistant").is_err());
}

/// Une règle aux horodatages invalides fait échouer le listage (AB-008).
#[test]
fn une_regle_corrompue_fait_echouer_le_listage() {
    let pool = base();
    pool.conn
        .execute(
            "INSERT INTO recurring_rules (id, kind, label, amount_cents, category_id,
                day_of_month, start_year, start_month, end_year, end_month, note,
                is_active, created_at, updated_at)
             VALUES ('regle-corrompue', 'expense', 'X', 1000, 'logement', 5, 2026, 8,
                NULL, NULL, NULL, 1, 'pas-un-horodatage', '2026-08-01T00:00:00Z')",
            [],
        )
        .unwrap();
    let erreur = commandes::lister(&pool).unwrap_err();
    assert!(
        erreur.contains("regle-corrompue"),
        "le message doit désigner la règle fautive : {erreur}"
    );
}
