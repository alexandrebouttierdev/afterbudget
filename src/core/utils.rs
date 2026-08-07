use chrono::Datelike;
use chrono::NaiveDate;

pub fn month_name(month: u32) -> &'static str {
    match month {
        1 => "Janvier",
        2 => "Février",
        3 => "Mars",
        4 => "Avril",
        5 => "Mai",
        6 => "Juin",
        7 => "Juillet",
        8 => "Août",
        9 => "Septembre",
        10 => "Octobre",
        11 => "Novembre",
        12 => "Décembre",
        _ => "Inconnu",
    }
}

pub fn month_year(year: i32, month: u32) -> String {
    format!("{} {}", month_name(month), year)
}

pub fn format_date_fr(date: &NaiveDate) -> String {
    date.format("%d/%m/%Y").to_string()
}

pub fn last_day_of_month(year: i32, month: u32) -> u32 {
    if month == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1)
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, 1)
    }
    .unwrap()
    .pred_opt()
    .unwrap()
    .day()
}

/// Formatte un pourcentage avec une décimale
pub fn format_percentage(pct: f64) -> String {
    format!("{:.1} %", pct)
}

/// Nom du jour de la semaine, en français.
pub fn nom_du_jour(date: &NaiveDate) -> &'static str {
    match date.weekday() {
        chrono::Weekday::Mon => "Lundi",
        chrono::Weekday::Tue => "Mardi",
        chrono::Weekday::Wed => "Mercredi",
        chrono::Weekday::Thu => "Jeudi",
        chrono::Weekday::Fri => "Vendredi",
        chrono::Weekday::Sat => "Samedi",
        chrono::Weekday::Sun => "Dimanche",
    }
}

/// Intitulé long d'un jour, utilisé comme séparateur de groupe dans la liste
/// des transactions : « Mercredi 12 août ».
pub fn jour_long(date: &NaiveDate) -> String {
    format!(
        "{} {} {}",
        nom_du_jour(date),
        date.day(),
        month_name(date.month()).to_lowercase()
    )
}

/// Formate une date pour la **saisie** : JJ/MM/AAAA, format français attendu
/// par l'utilisateur, distinct du format ISO stocké en base.
pub fn format_date_saisie(date: &NaiveDate) -> String {
    date.format("%d/%m/%Y").to_string()
}

/// Analyse une date saisie au format français. Tolère un jour ou un mois sur un
/// seul chiffre, refuse tout le reste.
pub fn analyser_date_saisie(saisie: &str) -> Option<NaiveDate> {
    let morceaux: Vec<&str> = saisie.trim().split('/').collect();
    if morceaux.len() != 3 {
        return None;
    }
    let jour: u32 = morceaux[0].parse().ok()?;
    let mois: u32 = morceaux[1].parse().ok()?;
    let annee: i32 = morceaux[2].parse().ok()?;
    if morceaux[2].len() != 4 {
        return None;
    }
    NaiveDate::from_ymd_opt(annee, mois, jour)
}

/// Filtre la frappe dans un champ date : chiffres uniquement, barres insérées
/// automatiquement, longueur bornée à JJ/MM/AAAA.
pub fn filtrer_saisie_date(saisie: &str) -> String {
    let chiffres: String = saisie
        .chars()
        .filter(|c| c.is_ascii_digit())
        .take(8)
        .collect();

    let mut resultat = String::with_capacity(10);
    for (index, caractere) in chiffres.chars().enumerate() {
        if index == 2 || index == 4 {
            resultat.push('/');
        }
        resultat.push(caractere);
    }
    resultat
}

/// Ramène un jour dans les bornes d'un mois donné : le 31 d'un mois de 30 jours
/// devient le 30. Sert aux règles récurrentes comme au sélecteur de date.
pub fn jour_borne(annee: i32, mois: u32, jour: u32) -> u32 {
    jour.clamp(1, last_day_of_month(annee, mois))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn les_mois_sont_nommes_en_francais() {
        assert_eq!(month_name(1), "Janvier");
        assert_eq!(month_name(8), "Août");
        assert_eq!(month_name(12), "Décembre");
        assert_eq!(month_name(0), "Inconnu");
        assert_eq!(month_name(13), "Inconnu");
    }

    #[test]
    fn les_jours_sont_nommes_en_francais() {
        // 12 août 2026 est un mercredi.
        let date = NaiveDate::from_ymd_opt(2026, 8, 12).unwrap();
        assert_eq!(nom_du_jour(&date), "Mercredi");
        assert_eq!(jour_long(&date), "Mercredi 12 août");
    }

    /// Le mois est en minuscules dans un intitulé de jour, en capitale en
    /// en-tête d'écran : les deux formes doivent rester cohérentes.
    #[test]
    fn lintitule_de_jour_ne_capitalise_que_le_jour() {
        let date = NaiveDate::from_ymd_opt(2026, 1, 4).unwrap();
        assert_eq!(jour_long(&date), "Dimanche 4 janvier");
        assert_eq!(month_year(2026, 1), "Janvier 2026");
    }

    #[test]
    fn le_dernier_jour_du_mois_gere_les_annees_bissextiles() {
        assert_eq!(last_day_of_month(2024, 2), 29);
        assert_eq!(last_day_of_month(2026, 2), 28);
        assert_eq!(last_day_of_month(2026, 12), 31);
        assert_eq!(last_day_of_month(2026, 4), 30);
    }

    #[test]
    fn les_dates_sont_formatees_a_la_francaise() {
        let date = NaiveDate::from_ymd_opt(2026, 8, 3).unwrap();
        assert_eq!(format_date_fr(&date), "03/08/2026");
    }

    #[test]
    fn une_date_de_saisie_fait_laller_retour() {
        let date = NaiveDate::from_ymd_opt(2026, 8, 3).unwrap();
        assert_eq!(format_date_saisie(&date), "03/08/2026");
        assert_eq!(analyser_date_saisie("03/08/2026"), Some(date));
        assert_eq!(analyser_date_saisie("3/8/2026"), Some(date));
    }

    /// Une saisie incomplète ou aberrante ne doit jamais produire de date.
    #[test]
    fn une_date_de_saisie_invalide_est_refusee() {
        for saisie in [
            "",
            "03/08",
            "2026-08-03",
            "32/08/2026",
            "03/13/2026",
            "31/02/2026",
            "03/08/26",
            "aa/bb/cccc",
        ] {
            assert!(
                analyser_date_saisie(saisie).is_none(),
                "« {saisie} » ne devrait pas être acceptée"
            );
        }
    }

    /// Le masque insère les barres au fil de la frappe et borne la longueur.
    #[test]
    fn le_masque_de_date_insere_les_barres() {
        assert_eq!(filtrer_saisie_date("0"), "0");
        assert_eq!(filtrer_saisie_date("03"), "03");
        assert_eq!(filtrer_saisie_date("038"), "03/8");
        assert_eq!(filtrer_saisie_date("03082026"), "03/08/2026");
        assert_eq!(filtrer_saisie_date("03/08/2026"), "03/08/2026");
        assert_eq!(filtrer_saisie_date("030820269999"), "03/08/2026");
        assert_eq!(filtrer_saisie_date("aa03bb08"), "03/08");
    }

    /// Un jour hors bornes est ramené dans le mois : le 31 d'un mois de 30
    /// jours devient le 30, et février est géré, année bissextile comprise.
    #[test]
    fn un_jour_hors_bornes_est_ramene_dans_le_mois() {
        assert_eq!(jour_borne(2026, 4, 31), 30);
        assert_eq!(jour_borne(2026, 2, 31), 28);
        assert_eq!(jour_borne(2024, 2, 31), 29);
        assert_eq!(jour_borne(2026, 1, 31), 31);
        assert_eq!(jour_borne(2026, 1, 0), 1);
    }

    #[test]
    fn les_pourcentages_gardent_une_decimale() {
        assert_eq!(format_percentage(12.34), "12.3 %");
        assert_eq!(format_percentage(0.0), "0.0 %");
    }
}
