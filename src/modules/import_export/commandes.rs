use crate::core::db::pool::DatabasePool;
use crate::modules::import_export::service;
use std::path::Path;

pub fn exporter(pool: &DatabasePool, dest: &Path) -> Result<(), String> {
    service::exporter(pool, dest)
}

pub fn valider_import(path: &Path) -> Result<(), String> {
    service::valider_import(path)
}

pub fn importer_en_arriere_plan(
    chemin_actuel: std::path::PathBuf,
    source: std::path::PathBuf,
) -> Result<(), String> {
    service::importer_en_arriere_plan(chemin_actuel, source)
}

pub fn nom_sauvegarde() -> String {
    service::nom_fichier_sauvegarde()
}
