use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct ImporterSauvegardeDto {
    pub chemin: PathBuf,
    pub confirmer_remplacement: bool,
}
