use std::fmt;
use std::ops;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Money {
    pub cents: i64,
}

impl Money {
    pub const ZERO: Money = Money { cents: 0 };

    pub fn from_cents(cents: i64) -> Self {
        Self { cents }
    }

    pub fn from_euros(euros: f64) -> Self {
        Self {
            cents: (euros * 100.0).round() as i64,
        }
    }

    pub fn to_euros_f64(self) -> f64 {
        self.cents as f64 / 100.0
    }

    pub fn from_input(s: &str) -> Result<Self, String> {
        // Accepte les deux espaces insécables et l'espace ordinaire, afin de
        // pouvoir recoller un montant précédemment affiché.
        let cleaned = s
            .replace(['\u{00a0}', '\u{202f}', ' '], "")
            .replace(',', ".");

        let trimmed = cleaned.trim();
        if trimmed.is_empty() {
            return Err("Le montant est obligatoire.".into());
        }

        let value: f64 = trimmed
            .parse()
            .map_err(|_| format!("Montant invalide : « {} »", s))?;

        if value.is_nan() || value.is_infinite() {
            return Err(format!("Montant invalide : « {} »", s));
        }

        if !(-900_000_000_000_000.0..=900_000_000_000_000.0).contains(&value) {
            return Err("Le montant est trop grand.".into());
        }

        Ok(Self::from_euros(value))
    }

    /// Filtre une frappe destinée à un champ montant.
    ///
    /// Ne laisse passer que les chiffres et un unique séparateur décimal ; les
    /// lettres et les signes sont simplement ignorés, ce qui évite d'afficher
    /// une erreur pour une touche que l'utilisateur n'aurait pas dû pouvoir
    /// saisir. Le séparateur est normalisé en virgule, usage français.
    pub fn filtrer_saisie(saisie: &str) -> String {
        let mut resultat = String::with_capacity(saisie.len());
        let mut separateur_place = false;
        let mut decimales = 0;

        for caractere in saisie.chars() {
            match caractere {
                '0'..='9' => {
                    if separateur_place {
                        // Au-delà de deux décimales, la frappe est ignorée.
                        if decimales == 2 {
                            continue;
                        }
                        decimales += 1;
                    }
                    resultat.push(caractere);
                }
                ',' | '.' if !separateur_place && !resultat.is_empty() => {
                    separateur_place = true;
                    resultat.push(',');
                }
                _ => {}
            }
        }

        resultat
    }

    /// Comme `filtrer_saisie`, mais conserve un signe moins en tête : un solde
    /// de compte peut être négatif, contrairement au montant d'une transaction.
    pub fn filtrer_saisie_signee(saisie: &str) -> String {
        let negatif = saisie.trim_start().starts_with('-');
        let chiffres = Self::filtrer_saisie(saisie);
        if negatif && !chiffres.is_empty() {
            format!("-{chiffres}")
        } else if negatif {
            "-".to_string()
        } else {
            chiffres
        }
    }

    pub fn format_fr(&self) -> String {
        let abs_cents = self.cents.abs();
        let euros_part = abs_cents / 100;
        let cents_part = abs_cents % 100;

        let mut result = String::new();
        if self.cents < 0 {
            result.push('-');
        }

        let euros_str = format!("{}", euros_part);
        let n = euros_str.len();
        for (i, c) in euros_str.chars().enumerate() {
            if i > 0 && (n - i) % 3 == 0 {
                result.push('\u{00a0}');
            }
            result.push(c);
        }

        result.push(',');
        result.push_str(&format!("{:02}", cents_part));
        result.push('\u{00a0}');
        result.push('€');

        result
    }

    pub fn format_fr_signed(&self) -> String {
        if self.cents >= 0 {
            format!("+{}", self.format_fr())
        } else {
            self.format_fr()
        }
    }

    pub fn is_zero(&self) -> bool {
        self.cents == 0
    }

    pub fn is_positive(&self) -> bool {
        self.cents > 0
    }

    pub fn is_negative(&self) -> bool {
        self.cents < 0
    }

    pub fn abs(&self) -> Self {
        Self {
            cents: self.cents.abs(),
        }
    }
}

impl fmt::Display for Money {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.format_fr())
    }
}

impl ops::Add for Money {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self {
            cents: self.cents + other.cents,
        }
    }
}

impl ops::AddAssign for Money {
    fn add_assign(&mut self, other: Self) {
        self.cents += other.cents;
    }
}

impl ops::Sub for Money {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self {
            cents: self.cents - other.cents,
        }
    }
}

impl ops::SubAssign for Money {
    fn sub_assign(&mut self, other: Self) {
        self.cents -= other.cents;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_cents() {
        let m = Money::from_cents(1000);
        assert_eq!(m.cents, 1000);
    }

    #[test]
    fn test_from_euros() {
        let m = Money::from_euros(10.50);
        assert_eq!(m.cents, 1050);
    }

    #[test]
    fn test_from_input_entier() {
        let m = Money::from_input("10").unwrap();
        assert_eq!(m.cents, 1000);
    }

    #[test]
    fn test_from_input_virgule() {
        let m = Money::from_input("10,50").unwrap();
        assert_eq!(m.cents, 1050);
    }

    #[test]
    fn test_from_input_point() {
        let m = Money::from_input("10.50").unwrap();
        assert_eq!(m.cents, 1050);
    }

    #[test]
    fn test_from_input_negatif() {
        let m = Money::from_input("-478").unwrap();
        assert_eq!(m.cents, -47800);
    }

    /// Un montant affiché doit pouvoir être ressaisi tel quel, quel que soit
    /// l'espace insécable utilisé.
    #[test]
    fn un_montant_formate_est_relisible() {
        let montant = Money::from_cents(120750);
        assert_eq!(
            Money::from_input(montant.format_fr().trim_end_matches('€')).unwrap(),
            montant
        );
        assert_eq!(Money::from_input("1\u{202f}207,50").unwrap(), montant);
        assert_eq!(Money::from_input("1 207,50").unwrap(), montant);
    }

    #[test]
    fn test_from_input_invalide() {
        assert!(Money::from_input("abc").is_err());
        assert!(Money::from_input("").is_err());
    }

    #[test]
    fn test_format_fr_positif() {
        let m = Money::from_cents(120750);
        assert_eq!(m.format_fr(), "1\u{00a0}207,50\u{00a0}€");
    }

    #[test]
    fn test_format_fr_negatif() {
        let m = Money::from_cents(-47800);
        assert_eq!(m.format_fr(), "-478,00\u{00a0}€");
    }

    #[test]
    fn test_addition() {
        let a = Money::from_cents(1000);
        let b = Money::from_cents(500);
        assert_eq!((a + b).cents, 1500);
    }

    #[test]
    fn test_soustraction() {
        let a = Money::from_cents(1000);
        let b = Money::from_cents(500);
        assert_eq!((a - b).cents, 500);
    }

    #[test]
    fn test_montant_nul_refuse() {
        assert!(Money::from_input("0").unwrap().cents == 0);
    }

    /// Un champ montant ne doit jamais accepter de lettre ni de signe.
    #[test]
    fn la_saisie_dun_montant_ignore_les_lettres() {
        assert_eq!(Money::filtrer_saisie("12a3"), "123");
        assert_eq!(Money::filtrer_saisie("abc"), "");
        assert_eq!(Money::filtrer_saisie("-45"), "45");
        assert_eq!(Money::filtrer_saisie("12 €"), "12");
        assert_eq!(Money::filtrer_saisie("1e5"), "15");
    }

    /// Un seul séparateur décimal, normalisé en virgule.
    #[test]
    fn la_saisie_dun_montant_normalise_le_separateur() {
        assert_eq!(Money::filtrer_saisie("12.50"), "12,50");
        assert_eq!(Money::filtrer_saisie("12,50"), "12,50");
        assert_eq!(Money::filtrer_saisie("12,5,7"), "12,57");
        assert_eq!(Money::filtrer_saisie("12.5.7"), "12,57");
    }

    /// Le séparateur ne peut pas ouvrir la saisie, et les décimales sont
    /// limitées à deux — au-delà, la frappe est ignorée.
    #[test]
    fn la_saisie_dun_montant_borne_les_decimales() {
        assert_eq!(Money::filtrer_saisie(",50"), "50");
        assert_eq!(Money::filtrer_saisie("12,567"), "12,56");
        assert_eq!(Money::filtrer_saisie(""), "");
    }

    /// Ce qui sort du filtre doit toujours être relisible par `from_input`.
    #[test]
    fn la_saisie_filtree_est_toujours_relisible() {
        for brut in ["12a3", "12.5.7", "-45,99", "0", "1 000,25"] {
            let filtre = Money::filtrer_saisie(brut);
            if !filtre.is_empty() {
                assert!(
                    Money::from_input(&filtre).is_ok(),
                    "« {brut} » filtré en « {filtre} » reste illisible"
                );
            }
        }
    }

    /// Un solde peut porter un signe moins, et lui seul.
    #[test]
    fn la_saisie_dun_solde_conserve_le_signe() {
        assert_eq!(Money::filtrer_saisie_signee("-360"), "-360");
        assert_eq!(Money::filtrer_saisie_signee("360"), "360");
        assert_eq!(Money::filtrer_saisie_signee("-1 240,80"), "-1240,80");
        assert_eq!(Money::filtrer_saisie_signee("36-0"), "360");
        assert_eq!(Money::filtrer_saisie_signee("-"), "-");
        assert_eq!(Money::filtrer_saisie_signee("-abc"), "-");
    }

    /// Ce qui sort du filtre signé doit rester relisible.
    #[test]
    fn la_saisie_signee_filtree_est_relisible() {
        for brut in ["-360", "-1 240,80", "0", "12a3"] {
            let filtre = Money::filtrer_saisie_signee(brut);
            if filtre != "-" && !filtre.is_empty() {
                assert!(
                    Money::from_input(&filtre).is_ok(),
                    "« {brut} » → « {filtre} »"
                );
            }
        }
    }

    #[test]
    fn test_comparaison() {
        let a = Money::from_cents(1000);
        let b = Money::from_cents(500);
        assert!(a > b);
        assert!(b < a);
        assert_eq!(a, Money::from_cents(1000));
    }
}
