use validator::Validate;

#[derive(Debug, Clone, Validate)]
pub struct CreerCategorieDto {
    #[validate(length(
        min = 1,
        max = 50,
        message = "Le nom est obligatoire (max 50 caractères)."
    ))]
    pub nom: String,
    #[validate(length(min = 1, message = "Le type est obligatoire."))]
    pub type_categorie: String,
    pub icone: String,
    #[validate(length(min = 1, message = "La couleur est obligatoire."))]
    pub couleur: String,
}
