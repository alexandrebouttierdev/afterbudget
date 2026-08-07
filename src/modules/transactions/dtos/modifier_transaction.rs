use validator::Validate;

#[derive(Debug, Clone, Validate)]
pub struct ModifierTransactionDto {
    #[validate(length(min = 1, message = "L'identifiant est obligatoire."))]
    pub id: String,
    #[validate(length(
        min = 1,
        max = 120,
        message = "Le libellé est obligatoire (max 120 caractères)."
    ))]
    pub libelle: String,
    #[validate(length(min = 1, message = "Le montant est obligatoire."))]
    pub montant: String,
    #[validate(length(min = 1, message = "La date est obligatoire."))]
    pub date: String,
    #[validate(length(min = 1, message = "La catégorie est obligatoire."))]
    pub categorie_id: String,
    pub type_transaction: String,
    pub statut: String,
    pub note: Option<String>,
}
