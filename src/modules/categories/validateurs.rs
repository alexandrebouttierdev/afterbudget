use crate::modules::categories::dtos::{CreerCategorieDto, ModifierCategorieDto};
use crate::modules::commun;

pub fn valider_creation(dto: &CreerCategorieDto) -> Result<(), Vec<(String, String)>> {
    let mut erreurs = Vec::new();
    if dto.nom.trim().is_empty() || dto.nom.len() > 50 {
        erreurs.push((
            "nom".into(),
            "Le nom est obligatoire (max 50 caractères).".into(),
        ));
    }
    if let Err(e) = commun::valider_couleur_hex(&dto.couleur) {
        erreurs.push(("couleur".into(), e));
    }
    if let Err(e) = commun::valider_type_transaction(&dto.type_categorie) {
        erreurs.push(("type".into(), e));
    }
    if erreurs.is_empty() {
        Ok(())
    } else {
        Err(erreurs)
    }
}

pub fn valider_modification(dto: &ModifierCategorieDto) -> Result<(), Vec<(String, String)>> {
    let tmp = CreerCategorieDto {
        nom: dto.nom.clone(),
        type_categorie: String::new(),
        icone: dto.icone.clone(),
        couleur: dto.couleur.clone(),
    };
    valider_creation(&tmp)
}
