use validator::Validate;

#[derive(Debug, Clone, Validate)]
pub struct TerminerOnboardingDto {
    #[validate(length(min = 1, message = "Le solde est obligatoire."))]
    pub solde_actuel: String,
    #[validate(length(min = 1, message = "Le découvert est obligatoire."))]
    pub decouvert_autorise: String,
    pub devise: String,
}
