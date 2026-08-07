use crate::core::db::pool::DatabasePool;
use crate::domaine::argent::Money;
use crate::domaine::parametres::AppSettings;
use crate::modules::commun;
use crate::modules::parametres::dtos::ModifierParametresDto;
use crate::modules::parametres::service;
use crate::modules::parametres::validateurs as valid;

pub fn obtenir_parametres(pool: &DatabasePool) -> Result<Option<AppSettings>, String> {
    service::obtenir_parametres(pool)
}

pub fn mettre_a_jour(pool: &DatabasePool, dto: &ModifierParametresDto) -> Result<(), String> {
    valid::valider_parametres(dto).map_err(|errs| {
        errs.iter()
            .map(|(f, e)| format!("{}: {}", f, e))
            .collect::<Vec<_>>()
            .join("\n")
    })?;

    let solde = Money::from_input(&dto.solde_actuel)?;
    let decouvert = Money::from_input(&dto.decouvert_autorise)?;
    commun::valider_decouvert(decouvert.cents)?;

    service::mettre_a_jour_solde(pool, solde)?;
    service::mettre_a_jour_decouvert(pool, decouvert)?;
    service::mettre_a_jour_devise(pool, &dto.devise)?;
    service::mettre_a_jour_theme(pool, &dto.theme)?;
    Ok(())
}
