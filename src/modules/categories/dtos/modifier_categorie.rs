use validator::Validate;

#[derive(Debug, Clone, Validate)]
pub struct ModifierCategorieDto {
    #[validate(length(min = 1))]
    pub id: String,
    #[validate(length(min = 1, max = 50))]
    pub nom: String,
    pub icone: String,
    #[validate(length(min = 1))]
    pub couleur: String,
    pub active: bool,
}
