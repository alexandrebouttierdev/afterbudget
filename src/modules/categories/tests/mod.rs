use crate::modules::categories::repository as repo;

mod commun;
use commun::base_temporaire;

#[test]
fn test_trouver_categories_par_defaut() {
    let pool = base_temporaire::creer_base_test();
    let cats = repo::find_all_active(&pool).unwrap();
    assert!(cats.iter().any(|c| c.id == "logement"));
    assert!(cats.iter().any(|c| c.id == "alimentation"));
    assert!(cats.iter().any(|c| c.id == "autre"));
}

#[test]
fn test_identifiants_categories_uniques() {
    let pool = base_temporaire::creer_base_test();
    let cats = repo::find_all_active(&pool).unwrap();
    let ids: Vec<&str> = cats.iter().map(|c| c.id.as_str()).collect();
    let mut unique = ids.clone();
    unique.sort();
    unique.dedup();
    assert_eq!(unique.len(), ids.len());
}

#[test]
fn test_couleurs_categories_valides() {
    let pool = base_temporaire::creer_base_test();
    let cats = repo::find_all_active(&pool).unwrap();
    for cat in &cats {
        assert!(cat.color.starts_with('#'));
        assert_eq!(cat.color.len(), 7);
    }
}

/// Une catégorie aux horodatages invalides fait échouer le chargement
/// (AB-008). Le kind invalide est bloqué par le CHECK du schéma.
#[test]
fn une_categorie_invalide_fait_echouer_le_chargement() {
    let pool = base_temporaire::creer_base_test();
    pool.conn
        .execute(
            "INSERT INTO categories (id, kind, name, icon, color, sort_order, is_default, is_active, created_at, updated_at)
             VALUES ('cat-corrompue', 'expense', 'X', 'x', '#3B82F6', 99, 0, 1, 'pas-un-horodatage', '2026-08-01T00:00:00Z')",
            [],
        )
        .unwrap();
    assert!(repo::find_all_active(&pool).is_err());
    assert!(repo::find_by_id(&pool, "cat-corrompue").is_err());
}
