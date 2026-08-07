#[derive(Debug, Clone)]
pub struct FiltrerTransactionsDto {
    pub mois: String,
    pub recherche: Option<String>,
    pub type_transaction: Option<String>,
    pub statut: Option<String>,
    pub categorie_id: Option<String>,
}
