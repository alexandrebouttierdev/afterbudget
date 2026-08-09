use rusqlite::params;

use crate::core::db::pool::DatabasePool;
use crate::domaine::argent::Money;
use crate::domaine::recurrence::RecurringRule;
use crate::domaine::transaction::TransactionKind;

const COLONNES: &str = "id, kind, label, amount_cents, category_id, day_of_month,
     start_year, start_month, end_year, end_month, note, is_active, created_at, updated_at";

fn depuis_ligne(ligne: &rusqlite::Row<'_>) -> rusqlite::Result<RecurringRule> {
    let genre: String = ligne.get(1)?;
    let fin_annee: Option<i32> = ligne.get(8)?;
    let fin_mois: Option<u32> = ligne.get(9)?;
    let cree: String = ligne.get(12)?;
    let modifie: String = ligne.get(13)?;

    let horodatage = |brut: &str| {
        chrono::DateTime::parse_from_rfc3339(brut)
            .map(|d| d.with_timezone(&chrono::Utc))
            .unwrap_or_else(|_| chrono::Utc::now())
    };

    Ok(RecurringRule {
        id: ligne.get(0)?,
        kind: TransactionKind::from_str(&genre).unwrap_or(TransactionKind::Expense),
        label: ligne.get(2)?,
        amount: Money::from_cents(ligne.get(3)?),
        category_id: ligne.get(4)?,
        day_of_month: ligne.get(5)?,
        start: (ligne.get(6)?, ligne.get(7)?),
        end: match (fin_annee, fin_mois) {
            (Some(annee), Some(mois)) => Some((annee, mois)),
            _ => None,
        },
        note: ligne.get(10)?,
        is_active: ligne.get::<_, i64>(11)? != 0,
        created_at: horodatage(&cree),
        updated_at: horodatage(&modifie),
    })
}

pub fn lister(pool: &DatabasePool) -> Result<Vec<RecurringRule>, String> {
    let sql = format!(
        "SELECT {COLONNES} FROM recurring_rules ORDER BY is_active DESC, day_of_month, label"
    );
    let mut requete = pool
        .conn
        .prepare(&sql)
        .map_err(|e| format!("Préparation lecture récurrences : {e}"))?;

    let lignes = requete
        .query_map([], depuis_ligne)
        .map_err(|e| format!("Lecture récurrences : {e}"))?;

    lignes
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|e| format!("Lecture récurrences : {e}"))
}

pub fn creer(pool: &DatabasePool, regle: &RecurringRule) -> Result<(), String> {
    pool.conn
        .execute(
            "INSERT INTO recurring_rules (id, kind, label, amount_cents, category_id,
                day_of_month, start_year, start_month, end_year, end_month, note,
                is_active, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
            params![
                regle.id,
                regle.kind.as_str(),
                regle.label,
                regle.amount.cents,
                regle.category_id,
                regle.day_of_month,
                regle.start.0,
                regle.start.1,
                regle.end.map(|(annee, _)| annee),
                regle.end.map(|(_, mois)| mois),
                regle.note,
                i64::from(regle.is_active),
                regle.created_at.to_rfc3339(),
                regle.updated_at.to_rfc3339(),
            ],
        )
        .map_err(|e| format!("Création récurrence : {e}"))?;
    Ok(())
}

/// Supprime la règle et **détache** les transactions déjà produites, qui
/// restent dans les mois où elles ont été matérialisées.
pub fn supprimer(pool: &DatabasePool, identifiant: &str) -> Result<(), String> {
    pool.conn
        .execute(
            "UPDATE transactions SET recurring_rule_id = NULL WHERE recurring_rule_id = ?1",
            params![identifiant],
        )
        .map_err(|e| format!("Détachement des transactions : {e}"))?;
    let supprimees = pool
        .conn
        .execute(
            "DELETE FROM recurring_rules WHERE id = ?1",
            params![identifiant],
        )
        .map_err(|e| format!("Suppression récurrence : {e}"))?;

    if supprimees == 0 {
        return Err(format!("Récurrence introuvable : {identifiant}"));
    }
    Ok(())
}

/// Vrai si la règle a déjà matérialisé une occurrence dans ce mois.
///
/// C'est le garde-fou anti-doublon : la génération est rejouée à chaque
/// ouverture du mois, elle doit rester sans effet la seconde fois.
pub fn occurrence_existe(
    pool: &DatabasePool,
    identifiant: &str,
    annee: i32,
    mois: u32,
) -> Result<bool, String> {
    let debut = format!("{annee:04}-{mois:02}-01");
    let fin = format!(
        "{annee:04}-{mois:02}-{:02}",
        crate::core::utils::last_day_of_month(annee, mois)
    );

    pool.conn
        .query_row(
            "SELECT COUNT(*) FROM transactions
             WHERE recurring_rule_id = ?1 AND transaction_date >= ?2 AND transaction_date <= ?3",
            params![identifiant, debut, fin],
            |ligne| ligne.get::<_, i64>(0),
        )
        .map(|total| total > 0)
        .map_err(|e| format!("Vérification d'occurrence : {e}"))
}

/// Nombre de transactions déjà produites par une règle, tous mois confondus.
pub fn compter_occurrences(pool: &DatabasePool, identifiant: &str) -> Result<i64, String> {
    pool.conn
        .query_row(
            "SELECT COUNT(*) FROM transactions WHERE recurring_rule_id = ?1",
            params![identifiant],
            |ligne| ligne.get(0),
        )
        .map_err(|e| format!("Comptage d'occurrences : {e}"))
}
