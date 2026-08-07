use crate::core::db::pool::DatabasePool;
use crate::domaine::recurrence::RecurringRule;
use crate::modules::recurrences::dtos::CreerRecurrenceDto;
use crate::modules::recurrences::service;
use crate::modules::recurrences::validateurs as valid;

pub fn creer(pool: &DatabasePool, dto: &CreerRecurrenceDto) -> Result<RecurringRule, String> {
    valid::valider_creation(dto).map_err(|erreurs| {
        erreurs
            .iter()
            .map(|(champ, message)| format!("{champ}: {message}"))
            .collect::<Vec<_>>()
            .join("\n")
    })?;

    service::creer(pool, dto)
}

pub fn lister(pool: &DatabasePool) -> Result<Vec<RecurringRule>, String> {
    service::lister(pool)
}

pub fn supprimer(pool: &DatabasePool, identifiant: &str) -> Result<(), String> {
    service::supprimer(pool, identifiant)
}

pub fn generer_pour_mois(pool: &DatabasePool, annee: i32, mois: u32) -> Result<usize, String> {
    service::generer_pour_mois(pool, annee, mois)
}
