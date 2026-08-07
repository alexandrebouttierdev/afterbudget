use validator::Validate;

#[derive(Debug, Clone, Validate)]
pub struct CreerRecurrenceDto {
    #[validate(length(
        min = 1,
        max = 120,
        message = "Le libellé est obligatoire (max 120 caractères)."
    ))]
    pub libelle: String,
    #[validate(length(min = 1, message = "Le montant est obligatoire."))]
    pub montant: String,
    #[validate(length(min = 1, message = "La catégorie est obligatoire."))]
    pub categorie_id: String,
    pub type_transaction: String,
    /// Jour du mois visé, de 1 à 31.
    pub jour_du_mois: u32,
    /// Premier mois couvert.
    pub debut_annee: i32,
    pub debut_mois: u32,
    pub note: Option<String>,
}
