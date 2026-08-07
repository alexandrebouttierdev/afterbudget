use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Erreur de base de données : {0}")]
    BaseDeDonnees(String),
    #[error("Erreur de migration : {0}")]
    Migration(String),
    #[error("Erreur de validation : {0}")]
    Validation(String),
    #[error("Erreur métier : {0}")]
    Metier(String),
    #[error("Erreur d'import/export : {0}")]
    ImportExport(String),
    #[error("Erreur de fichier : {0}")]
    Fichier(String),
    #[error("Erreur d'initialisation : {0}")]
    Initialisation(String),
}

pub type AppResult<T> = Result<T, AppError>;

impl From<rusqlite::Error> for AppError {
    fn from(e: rusqlite::Error) -> Self {
        AppError::BaseDeDonnees(e.to_string())
    }
}
