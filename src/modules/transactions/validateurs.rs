use crate::modules::commun;
use crate::modules::transactions::dtos::{CreerTransactionDto, ModifierTransactionDto};

#[allow(clippy::type_complexity)]
pub fn valider_creation(
    dto: &CreerTransactionDto,
) -> Result<Vec<(String, String)>, Vec<(String, String)>> {
    let mut erreurs = Vec::new();

    if dto.libelle.trim().is_empty() || dto.libelle.len() > 120 {
        erreurs.push((
            "libelle".into(),
            "Le libellé est obligatoire (max 120 caractères).".into(),
        ));
    }
    if let Err(e) = commun::valider_montant(&dto.montant) {
        erreurs.push(("montant".into(), e));
    }
    if let Err(e) = commun::valider_date(&dto.date) {
        erreurs.push(("date".into(), e));
    }
    if dto.categorie_id.trim().is_empty() {
        erreurs.push((
            "categorie_id".into(),
            "La catégorie est obligatoire.".into(),
        ));
    }
    if let Err(e) = commun::valider_type_transaction(&dto.type_transaction) {
        erreurs.push(("type".into(), e));
    }
    if let Err(e) = commun::valider_statut(&dto.statut) {
        erreurs.push(("statut".into(), e));
    }
    if let Some(ref note) = dto.note {
        if note.chars().count() > 1000 {
            erreurs.push((
                "note".into(),
                "La note ne doit pas dépasser 1 000 caractères.".into(),
            ));
        }
    }

    if erreurs.is_empty() {
        Ok(erreurs)
    } else {
        Err(erreurs)
    }
}

#[allow(clippy::type_complexity)]
pub fn valider_modification(
    dto: &ModifierTransactionDto,
) -> Result<Vec<(String, String)>, Vec<(String, String)>> {
    let tmp = CreerTransactionDto {
        libelle: dto.libelle.clone(),
        montant: dto.montant.clone(),
        date: dto.date.clone(),
        categorie_id: dto.categorie_id.clone(),
        type_transaction: dto.type_transaction.clone(),
        statut: dto.statut.clone(),
        note: dto.note.clone(),
    };
    valider_creation(&tmp)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dto_valide() -> CreerTransactionDto {
        CreerTransactionDto {
            libelle: "Courses".into(),
            montant: "50,00".into(),
            date: "2026-08-01".into(),
            categorie_id: "alimentation".into(),
            type_transaction: "expense".into(),
            statut: "pending".into(),
            note: None,
        }
    }

    #[test]
    fn test_libelle_vide() {
        let mut dto = dto_valide();
        dto.libelle = "".into();
        assert!(valider_creation(&dto).is_err());
    }

    #[test]
    fn test_libelle_trop_long() {
        let mut dto = dto_valide();
        dto.libelle = "a".repeat(121);
        assert!(valider_creation(&dto).is_err());
    }

    #[test]
    fn test_montant_invalide() {
        let mut dto = dto_valide();
        dto.montant = "abc".into();
        assert!(valider_creation(&dto).is_err());
    }

    #[test]
    fn test_montant_zero() {
        let mut dto = dto_valide();
        dto.montant = "0".into();
        assert!(valider_creation(&dto).is_err());
    }

    #[test]
    fn test_montant_negatif() {
        let mut dto = dto_valide();
        dto.montant = "-10".into();
        assert!(valider_creation(&dto).is_err());
    }

    #[test]
    fn test_date_invalide() {
        let mut dto = dto_valide();
        dto.date = "pas-une-date".into();
        assert!(valider_creation(&dto).is_err());
    }

    #[test]
    fn test_categorie_vide() {
        let mut dto = dto_valide();
        dto.categorie_id = "".into();
        assert!(valider_creation(&dto).is_err());
    }

    #[test]
    fn test_type_invalide() {
        let mut dto = dto_valide();
        dto.type_transaction = "inconnu".into();
        assert!(valider_creation(&dto).is_err());
    }

    #[test]
    fn test_statut_invalide() {
        let mut dto = dto_valide();
        dto.statut = "inconnu".into();
        assert!(valider_creation(&dto).is_err());
    }

    #[test]
    fn test_note_trop_longue() {
        let mut dto = dto_valide();
        dto.note = Some("a".repeat(1001));
        assert!(valider_creation(&dto).is_err());
    }

    #[test]
    fn test_valide() {
        let dto = dto_valide();
        assert!(valider_creation(&dto).is_ok());
    }

    /// Une note de 1000 caractères multioctets passe, 1001 est refusée :
    /// la limite compte des caractères, pas des octets (AB-006).
    #[test]
    fn la_limite_de_note_compte_les_caracteres() {
        let ok = {
            let mut d = dto_valide();
            d.note = Some("😀".repeat(1000));
            d
        };
        assert!(valider_creation(&ok).is_ok());

        let trop = {
            let mut d = dto_valide();
            d.note = Some("😀".repeat(1001));
            d
        };
        let erreurs = valider_creation(&trop).unwrap_err();
        assert!(erreurs.iter().any(|(champ, _)| champ == "note"));
    }
}
