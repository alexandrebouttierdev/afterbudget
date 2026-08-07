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
