use crate::modules::commun;
use crate::modules::recurrences::dtos::CreerRecurrenceDto;

pub fn valider_creation(dto: &CreerRecurrenceDto) -> Result<(), Vec<(String, String)>> {
    let mut erreurs = Vec::new();

    if dto.libelle.trim().is_empty() || dto.libelle.chars().count() > 120 {
        erreurs.push((
            "libelle".into(),
            "Le libellé est obligatoire (max 120 caractères).".into(),
        ));
    }
    if let Err(e) = commun::valider_montant(&dto.montant) {
        erreurs.push(("montant".into(), e));
    }
    if dto.categorie_id.trim().is_empty() {
        erreurs.push(("categorie".into(), "La catégorie est obligatoire.".into()));
    }
    if let Err(e) = commun::valider_type_transaction(&dto.type_transaction) {
        erreurs.push(("type".into(), e));
    }
    if !(1..=31).contains(&dto.jour_du_mois) {
        erreurs.push((
            "jour".into(),
            "Le jour du mois doit être compris entre 1 et 31.".into(),
        ));
    }
    if !(1..=12).contains(&dto.debut_mois) {
        erreurs.push(("debut".into(), "Mois de départ invalide.".into()));
    }

    if erreurs.is_empty() {
        Ok(())
    } else {
        Err(erreurs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dto() -> CreerRecurrenceDto {
        CreerRecurrenceDto {
            libelle: "Loyer".into(),
            montant: "890".into(),
            categorie_id: "logement".into(),
            type_transaction: "expense".into(),
            jour_du_mois: 3,
            debut_annee: 2026,
            debut_mois: 8,
            note: None,
        }
    }

    #[test]
    fn une_regle_complete_est_valide() {
        assert!(valider_creation(&dto()).is_ok());
    }

    /// Le jour du mois est borné à l'intervalle réel d'un calendrier.
    #[test]
    fn un_jour_hors_intervalle_est_refuse() {
        for jour in [0, 32, 99] {
            let mut d = dto();
            d.jour_du_mois = jour;
            let erreurs = valider_creation(&d).unwrap_err();
            assert!(erreurs.iter().any(|(champ, _)| champ == "jour"));
        }
    }

    /// Le montant d'une récurrence suit la même règle qu'une transaction :
    /// strictement positif, le sens étant porté par le type.
    #[test]
    fn un_montant_non_positif_est_refuse() {
        for montant in ["0", "-10", "abc", ""] {
            let mut d = dto();
            d.montant = montant.into();
            assert!(valider_creation(&d).is_err(), "montant « {montant} »");
        }
    }

    #[test]
    fn un_libelle_vide_est_refuse() {
        let mut d = dto();
        d.libelle = "   ".into();
        let erreurs = valider_creation(&d).unwrap_err();
        assert!(erreurs.iter().any(|(champ, _)| champ == "libelle"));
    }

    #[test]
    fn un_type_inconnu_est_refuse() {
        let mut d = dto();
        d.type_transaction = "transfert".into();
        assert!(valider_creation(&d).is_err());
    }
}
