use crate::core::db::pool::DatabasePool;
use crate::domaine::argent::Money;
use crate::modules::commun;
use crate::modules::onboarding::dtos::TerminerOnboardingDto;
use crate::modules::parametres::service as parametres_service;
use crate::modules::parametres::validateurs as valid;

pub fn terminer(pool: &DatabasePool, dto: &TerminerOnboardingDto) -> Result<(), String> {
    valid::valider_onboarding(dto).map_err(|errs| {
        errs.iter()
            .map(|(f, e)| format!("{}: {}", f, e))
            .collect::<Vec<_>>()
            .join("\n")
    })?;

    let solde = Money::from_input(&dto.solde_actuel)?;
    let decouvert = Money::from_input(&dto.decouvert_autorise)?;
    commun::valider_decouvert(decouvert.cents)?;

    parametres_service::terminer_onboarding(pool, solde, decouvert, &dto.devise)
}
