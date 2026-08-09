use crate::core::db::pool::DatabasePool;
use crate::domaine::argent::Money;
use crate::domaine::recurrence::RecurringRule;
use crate::domaine::transaction::{TransactionKind, TransactionStatus};
use crate::modules::recurrences::dtos::CreerRecurrenceDto;
use crate::modules::recurrences::repository as repo;
use crate::modules::transactions::repository as transactions_repo;

/// Crée une règle à partir d'un formulaire déjà validé.
pub fn creer(pool: &DatabasePool, dto: &CreerRecurrenceDto) -> Result<RecurringRule, String> {
    let montant = crate::modules::commun::valider_montant(&dto.montant)?;
    let genre =
        TransactionKind::from_str(&dto.type_transaction).ok_or("Type de transaction invalide.")?;

    let maintenant = chrono::Utc::now();
    let regle = RecurringRule {
        id: uuid::Uuid::new_v4().to_string(),
        kind: genre,
        label: dto.libelle.trim().to_string(),
        amount: montant,
        category_id: dto.categorie_id.clone(),
        day_of_month: dto.jour_du_mois,
        start: (dto.debut_annee, dto.debut_mois),
        end: None,
        note: dto.note.clone(),
        is_active: true,
        created_at: maintenant,
        updated_at: maintenant,
    };

    repo::creer(pool, &regle)?;
    Ok(regle)
}

pub fn lister(pool: &DatabasePool) -> Result<Vec<RecurringRule>, String> {
    repo::lister(pool)
}

pub fn supprimer(pool: &DatabasePool, identifiant: &str) -> Result<(), String> {
    repo::supprimer(pool, identifiant)
}

/// Matérialise dans un mois toutes les règles qui le couvrent.
///
/// Appelée à chaque ouverture d'un mois, donc **idempotente** : une règle qui a
/// déjà produit son occurrence dans ce mois est ignorée. Supprimer la
/// transaction générée ne la fait pas revenir tant que le mois reste ouvert —
/// elle réapparaîtra à la prochaine ouverture, ce qui est le comportement
/// attendu d'une règle toujours active.
///
/// Renvoie le nombre d'occurrences réellement créées.
pub fn generer_pour_mois(pool: &DatabasePool, annee: i32, mois: u32) -> Result<usize, String> {
    let regles = repo::lister(pool)?;
    let mut creees = 0;

    for regle in regles.iter().filter(|r| r.couvre(annee, mois)) {
        if repo::occurrence_existe(pool, &regle.id, annee, mois)? {
            continue;
        }
        let Some(date) = regle.date_pour(annee, mois) else {
            continue;
        };

        transactions_repo::insert_from_rule(
            pool,
            &regle.id,
            regle.kind,
            &regle.label,
            regle.amount,
            date,
            &regle.category_id,
            // Une occurrence naît toujours « en attente » : c'est ce qui la
            // fait entrer dans la prévision de fin de mois.
            TransactionStatus::Pending,
            regle.note.clone(),
        )?;
        creees += 1;
    }

    Ok(creees)
}

/// Somme mensuelle des règles actives d'un sens donné, pour information.
pub fn total_mensuel(regles: &[RecurringRule], genre: TransactionKind) -> Money {
    regles
        .iter()
        .filter(|r| r.is_active && r.kind == genre)
        .fold(Money::ZERO, |total, r| total.saturating_add(r.amount))
}
