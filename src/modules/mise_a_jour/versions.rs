//! Versions sémantiques de l'application.
//!
//! Le distante vient du tag d'une release GitHub (`v0.2.3`), la locale de
//! `Cargo.toml` (`0.1.0`). La comparaison se fait champ par champ, sans
//! dépendance externe.

use std::fmt;

/// Version sémantique : majeur.mineur.correctif.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Version {
    pub majeur: u32,
    pub mineur: u32,
    pub correctif: u32,
}

impl Version {
    /// Parse `"0.2.3"` ou `"v0.2.3"`. Retourne `None` pour tout autre format.
    pub fn parse(s: &str) -> Option<Self> {
        let corps = s.strip_prefix('v').unwrap_or(s);
        let mut parties = corps.split('.');
        let majeur = parties.next()?.parse().ok()?;
        let mineur = parties.next()?.parse().ok()?;
        let correctif = parties.next()?.parse().ok()?;
        if parties.next().is_some() {
            return None;
        }
        Some(Self {
            majeur,
            mineur,
            correctif,
        })
    }

    /// Version de la compilation courante, tirée de `Cargo.toml`.
    pub fn from_cargo() -> Self {
        // La version Cargo est toujours valide : `expect` est sûr.
        Self::parse(env!("CARGO_PKG_VERSION")).expect("version Cargo invalide")
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.majeur, self.mineur, self.correctif)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn une_version_se_parse_avec_ou_sans_prefixe() {
        assert_eq!(
            Version::parse("0.2.3"),
            Some(Version {
                majeur: 0,
                mineur: 2,
                correctif: 3
            })
        );
        assert_eq!(
            Version::parse("v1.10.0"),
            Some(Version {
                majeur: 1,
                mineur: 10,
                correctif: 0
            })
        );
    }

    #[test]
    fn un_format_invalide_est_refuse() {
        for invalide in ["", "v", "abc", "1.2", "1.2.3.4", "1.x.3", "1.2.3-rc1"] {
            assert_eq!(Version::parse(invalide), None, "{invalide:?} doit être refusé");
        }
    }

    /// La comparaison est numérique, jamais lexicographique : 0.10.0 est plus
    /// récent que 0.9.9.
    #[test]
    fn la_comparaison_est_numerique() {
        let v0910 = Version::parse("0.10.0").unwrap();
        let v099 = Version::parse("0.9.9").unwrap();
        assert!(v0910 > v099);
        assert!(v099 < v0910);

        let v123 = Version::parse("1.2.3").unwrap();
        assert!(v123 > v0910);
        assert_eq!(Version::parse("1.2.3"), Some(v123));
    }

    #[test]
    fn la_version_cargo_est_lue_et_reexprimee() {
        let version = Version::from_cargo();
        assert_eq!(version.to_string(), env!("CARGO_PKG_VERSION"));
    }
}
