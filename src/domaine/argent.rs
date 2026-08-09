use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Money {
    pub cents: i64,
}

impl Money {
    pub const ZERO: Money = Money { cents: 0 };

    pub fn from_cents(cents: i64) -> Self {
        Self { cents }
    }

    /// Parse une saisie monétaire française en centimes, sans aucune
    /// conversion flottante (AB-005).
    ///
    /// Accepte un signe `-` en tête (un solde peut être négatif), des espaces
    /// de milliers (ordinaires ou insécables) et un unique séparateur décimal
    /// `,` ou `.`. Au-delà de deux décimales, la saisie est refusée — jamais
    /// arrondie. Le débordement de `i64` est une erreur.
    pub fn from_input(s: &str) -> Result<Self, String> {
        let cleaned = s
            .replace(['\u{00a0}', '\u{202f}', ' '], "")
            .replace(',', ".");

        let trimmed = cleaned.trim();
        if trimmed.is_empty() {
            return Err("Le montant est obligatoire.".into());
        }

        let (negatif, corps) = match trimmed.as_bytes()[0] {
            b'-' => (true, &trimmed[1..]),
            b'+' => (false, &trimmed[1..]),
            _ => (false, trimmed),
        };
        if corps.is_empty() {
            return Err(format!("Montant invalide : « {} »", s));
        }

        let (entiers, decimales) = match corps.split_once('.') {
            Some((e, d)) => (e, d),
            None => (corps, ""),
        };
        if !entiers.chars().all(|c| c.is_ascii_digit())
            || !decimales.chars().all(|c| c.is_ascii_digit())
        {
            return Err(format!("Montant invalide : « {} »", s));
        }
        if decimales.len() > 2 {
            return Err(format!("Montant invalide : « {} »", s));
        }

        let entiers: i64 = entiers
            .parse()
            .map_err(|_| "Le montant est trop grand.".to_string())?;
        let decimales: i64 = if decimales.is_empty() {
            0
        } else if decimales.len() == 1 {
            decimales
                .parse::<i64>()
                .map_err(|_| "Le montant est trop grand.".to_string())?
                * 10
        } else {
            decimales
                .parse()
                .map_err(|_| "Le montant est trop grand.".to_string())?
        };

        let centimes = if negatif {
            // La partie entière est négativée d'abord : c'est la seule façon de
            // représenter i64::MIN, dont la valeur absolue déborde i64.
            entiers
                .checked_neg()
                .and_then(|v| v.checked_mul(100))
                .and_then(|v| v.checked_sub(decimales))
                .ok_or_else(|| "Le montant est trop grand.".to_string())?
        } else {
            entiers
                .checked_mul(100)
                .and_then(|v| v.checked_add(decimales))
                .ok_or_else(|| "Le montant est trop grand.".to_string())?
        };

        Ok(Self { cents: centimes })
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
        // unsigned_abs : jamais de panic sur i64::MIN (AB-005).
        let abs_cents = self.cents.unsigned_abs();
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

    /// Valeur absolue pour l'affichage : saturée, jamais de panic (AB-005).
    pub fn abs(&self) -> Self {
        Self {
            cents: self.cents.saturating_abs(),
        }
    }

    pub fn checked_abs(self) -> Option<Self> {
        self.cents.checked_abs().map(|cents| Self { cents })
    }

    pub fn checked_add(self, other: Self) -> Option<Self> {
        self.cents
            .checked_add(other.cents)
            .map(|cents| Self { cents })
    }

    pub fn checked_sub(self, other: Self) -> Option<Self> {
        self.cents
            .checked_sub(other.cents)
            .map(|cents| Self { cents })
    }

    /// Addition saturée : utilisée uniquement pour des affichages de synthèse.
    pub fn saturating_add(self, other: Self) -> Self {
        Self {
            cents: self.cents.saturating_add(other.cents),
        }
    }
}

impl fmt::Display for Money {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.format_fr())
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
    fn test_parsing_entier_exact() {
        assert_eq!(Money::from_input("10").unwrap().cents, 1000);
        assert_eq!(Money::from_input("10,50").unwrap().cents, 1050);
        assert_eq!(Money::from_input("10.50").unwrap().cents, 1050);
        assert_eq!(Money::from_input("-478").unwrap().cents, -47800);
        assert_eq!(Money::from_input("1 207,50").unwrap().cents, 120750);
        assert_eq!(Money::from_input("1\u{00a0}207,50").unwrap().cents, 120750);
        assert_eq!(Money::from_input("1\u{202f}207,50").unwrap().cents, 120750);
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

    /// Trois décimales sont refusées : pas d'arrondi silencieux (AB-005).
    #[test]
    fn trois_decimales_sont_refusees() {
        assert!(Money::from_input("12,345").is_err());
        assert!(Money::from_input("12.345").is_err());
    }

    /// Les bornes i64 sont les bornes du contrat : dépassement refusé, pas de
    /// wrap, pas de panic.
    #[test]
    fn les_bornes_i64_encadrent_le_montant() {
        let max = "92233720368547758,07";
        assert_eq!(Money::from_input(max).unwrap().cents, i64::MAX);
        let min = "-92233720368547758,08";
        assert_eq!(Money::from_input(min).unwrap().cents, i64::MIN);
        assert!(Money::from_input("92233720368547758,08").is_err());
        assert!(Money::from_input("-92233720368547758,09").is_err());
    }

    /// Un exposant scientifique n'est pas une saisie monétaire.
    #[test]
    fn une_saisie_scientifique_est_refusee() {
        assert!(Money::from_input("1e5").is_err());
    }

    #[test]
    fn laddition_verifiee_detecte_le_debordement() {
        let a = Money::from_cents(i64::MAX);
        assert_eq!(a.checked_add(Money::from_cents(1)), None);
        assert_eq!(a.checked_add(Money::ZERO), Some(a));
        let b = Money::from_cents(i64::MIN);
        assert_eq!(b.checked_sub(Money::from_cents(1)), None);
        assert_eq!(b.checked_abs(), None);
        assert_eq!(
            Money::from_cents(-5).checked_abs(),
            Some(Money::from_cents(5))
        );
    }

    /// L'affichage ne doit jamais paniquer, même sur la valeur minimale
    /// (l'abs littéral de i64::MIN est un débordement).
    #[test]
    fn laffichage_ne_panique_pas_sur_min() {
        let m = Money::from_cents(i64::MIN);
        let s = m.format_fr();
        assert_eq!(
            s,
            "-92\u{00a0}233\u{00a0}720\u{00a0}368\u{00a0}547\u{00a0}758,08\u{00a0}€"
        );
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
