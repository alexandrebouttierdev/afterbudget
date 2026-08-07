use crate::modules::commun;
use crate::modules::onboarding::dtos::TerminerOnboardingDto;
use crate::modules::parametres::dtos::ModifierParametresDto;

pub fn valider_parametres(dto: &ModifierParametresDto) -> Result<(), Vec<(String, String)>> {
    let mut erreurs = Vec::new();
    // Un solde de compte peut être négatif : on peut déjà être à découvert.
    if let Err(e) = commun::valider_solde(&dto.solde_actuel) {
        erreurs.push(("solde_actuel".into(), e));
    }
    if let Err(e) = crate::domaine::argent::Money::from_input(&dto.decouvert_autorise) {
        erreurs.push(("decouvert".into(), e.to_string()));
    } else if let Ok(m) = crate::domaine::argent::Money::from_input(&dto.decouvert_autorise) {
        if let Err(e) = commun::valider_decouvert(m.cents) {
            erreurs.push(("decouvert".into(), e));
        }
    }
    if let Err(e) = commun::valider_devise(&dto.devise) {
        erreurs.push(("devise".into(), e));
    }
    if erreurs.is_empty() {
        Ok(())
    } else {
        Err(erreurs)
    }
}

pub fn valider_onboarding(dto: &TerminerOnboardingDto) -> Result<(), Vec<(String, String)>> {
    valider_parametres(&ModifierParametresDto {
        solde_actuel: dto.solde_actuel.clone(),
        decouvert_autorise: dto.decouvert_autorise.clone(),
        devise: dto.devise.clone(),
        theme: "system".into(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dto(solde: &str, decouvert: &str) -> ModifierParametresDto {
        ModifierParametresDto {
            solde_actuel: solde.into(),
            decouvert_autorise: decouvert.into(),
            devise: "EUR".into(),
            theme: "light".into(),
        }
    }

    /// Terminer l'onboarding avec un compte déjà à découvert doit fonctionner.
    #[test]
    fn un_solde_negatif_passe_la_validation() {
        assert!(valider_parametres(&dto("-360", "500")).is_ok());
        assert!(valider_parametres(&dto("0", "0")).is_ok());
        assert!(valider_onboarding(&TerminerOnboardingDto {
            solde_actuel: "-360".into(),
            decouvert_autorise: "500".into(),
            devise: "EUR".into(),
        })
        .is_ok());
    }

    /// Le découvert autorisé, lui, reste positif ou nul : c'est un plafond.
    #[test]
    fn un_decouvert_negatif_reste_refuse() {
        let erreurs = valider_parametres(&dto("100", "-500")).unwrap_err();
        assert!(erreurs.iter().any(|(champ, _)| champ == "decouvert"));
    }

    #[test]
    fn un_solde_illisible_est_signale_sur_le_bon_champ() {
        let erreurs = valider_parametres(&dto("abc", "500")).unwrap_err();
        assert!(erreurs.iter().any(|(champ, _)| champ == "solde_actuel"));
    }
}
