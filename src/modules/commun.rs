use crate::domaine::argent::Money;

pub fn valider_montant(montant: &str) -> Result<Money, String> {
    let m = Money::from_input(montant)?;
    if m.cents <= 0 {
        return Err("Le montant doit être supérieur à zéro.".into());
    }
    Ok(m)
}

/// Valide un **solde de compte**, par opposition à un montant de transaction.
///
/// Un solde peut être nul, et surtout **négatif** : on peut parfaitement être
/// déjà à découvert au moment où l'on renseigne son budget. Seule la lisibilité
/// de la valeur est vérifiée.
pub fn valider_solde(solde: &str) -> Result<Money, String> {
    Money::from_input(solde)
}

pub fn valider_date(date: &str) -> Result<chrono::NaiveDate, String> {
    chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d")
        .map_err(|_| "Date invalide (format attendu : AAAA-MM-JJ).".into())
}

pub fn valider_couleur_hex(couleur: &str) -> Result<(), String> {
    if couleur.len() != 7 || !couleur.starts_with('#') {
        return Err("Format attendu : #RRGGBB.".into());
    }
    u32::from_str_radix(&couleur[1..], 16)
        .map_err(|_| "Code couleur hexadécimal invalide.".to_string())?;
    Ok(())
}

pub fn valider_type_transaction(t: &str) -> Result<(), String> {
    match t {
        "income" | "expense" => Ok(()),
        _ => Err("Type de transaction invalide (attendu : income ou expense).".into()),
    }
}

pub fn valider_statut(s: &str) -> Result<(), String> {
    match s {
        "pending" | "completed" => Ok(()),
        _ => Err("Statut invalide (attendu : pending ou completed).".into()),
    }
}

pub fn valider_decouvert(cents: i64) -> Result<(), String> {
    if cents < 0 {
        return Err("Le découvert autorisé ne peut pas être négatif.".into());
    }
    Ok(())
}

pub fn valider_devise(d: &str) -> Result<(), String> {
    match d {
        "EUR" | "USD" | "GBP" => Ok(()),
        _ => Err("Devise non reconnue.".into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valider_montant_valide() {
        assert!(valider_montant("10,50").is_ok());
        assert!(valider_montant("100").is_ok());
    }

    #[test]
    fn test_valider_montant_zero() {
        assert!(valider_montant("0").is_err());
    }

    #[test]
    fn test_valider_montant_negatif() {
        assert!(valider_montant("-5").is_err());
    }

    /// Un solde peut être négatif : on peut déjà être à découvert.
    #[test]
    fn un_solde_negatif_est_accepte() {
        assert_eq!(valider_solde("-360").unwrap().cents, -36000);
        assert_eq!(valider_solde("-1 240,80").unwrap().cents, -124080);
        assert_eq!(valider_solde("0").unwrap().cents, 0);
        assert_eq!(valider_solde("1250,50").unwrap().cents, 125050);
    }

    /// Un solde illisible reste refusé.
    #[test]
    fn un_solde_illisible_est_refuse() {
        assert!(valider_solde("abc").is_err());
        assert!(valider_solde("").is_err());
    }

    /// Le montant d'une transaction, lui, reste strictement positif : le sens
    /// est porté par le type de la transaction, pas par le signe du montant.
    #[test]
    fn un_montant_de_transaction_reste_strictement_positif() {
        assert!(valider_montant("-5").is_err());
        assert!(valider_montant("0").is_err());
        assert!(valider_montant("5").is_ok());
    }

    #[test]
    fn test_valider_date_valide() {
        assert!(valider_date("2026-08-03").is_ok());
    }

    #[test]
    fn test_valider_date_invalide() {
        assert!(valider_date("03/08/2026").is_err());
    }

    #[test]
    fn test_valider_couleur_hex() {
        assert!(valider_couleur_hex("#3B82F6").is_ok());
        assert!(valider_couleur_hex("invalid").is_err());
        assert!(valider_couleur_hex("#ZZZZZZ").is_err());
    }

    #[test]
    fn test_valider_type_transaction() {
        assert!(valider_type_transaction("income").is_ok());
        assert!(valider_type_transaction("expense").is_ok());
        assert!(valider_type_transaction("invalid").is_err());
    }

    #[test]
    fn test_valider_statut() {
        assert!(valider_statut("pending").is_ok());
        assert!(valider_statut("completed").is_ok());
        assert!(valider_statut("invalid").is_err());
    }

    #[test]
    fn test_valider_decouvert() {
        assert!(valider_decouvert(50000).is_ok());
        assert!(valider_decouvert(0).is_ok());
        assert!(valider_decouvert(-1).is_err());
    }

    #[test]
    fn test_valider_devise() {
        assert!(valider_devise("EUR").is_ok());
        assert!(valider_devise("USD").is_ok());
        assert!(valider_devise("GBP").is_ok());
        assert!(valider_devise("JPY").is_err());
    }
}
