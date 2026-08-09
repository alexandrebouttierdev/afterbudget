//! Génère une base SQLite de démonstration, remplie de fausses données
//! réalistes, destinée aux captures d'écran du site et aux essais locaux.
//!
//! Aucune donnée réelle n'est utilisée : le fichier cible est recréé de zéro
//! à chaque exécution, dans le répertoire `demo/` (ignoré par git).
//!
//! Usage :
//! ```text
//! cargo run --bin seed_demo                # base « utilisée depuis des mois »
//! cargo run --bin seed_demo -- --onboarding   # base vierge, écran de bienvenue
//! cargo run --bin seed_demo -- <chemin> [--onboarding]
//! ```

use afterbudget::core::db::{migrations, pool::DatabasePool};
use rusqlite::params;
use std::path::{Path, PathBuf};

/// Chemin par défaut : respecte la disposition `XDG_DATA_HOME`, que
/// l'application suit aussi (`$XDG_DATA_HOME/afterbudget/afterbudget.sqlite`).
const CHEMIN_PAR_DEFAUT: &str = "demo/afterbudget/afterbudget.sqlite";
const CHEMIN_ONBOARDING: &str = "demo/onboarding/afterbudget/afterbudget.sqlite";

/// Une opération récurrente (règle mensuelle).
struct Regle<'a> {
    id: &'a str,
    genre: &'a str,
    libelle: &'a str,
    montant_centimes: i64,
    categorie: &'a str,
    jour: i64,
    debut: (i32, i64),
    fin: Option<(i32, i64)>,
    active: bool,
    note: Option<&'a str>,
}

/// Une transaction (réalisée ou en attente).
struct Operation<'a> {
    genre: &'a str,
    libelle: &'a str,
    montant_centimes: i64,
    date: &'a str,
    statut: &'a str,
    categorie: &'a str,
    regle: Option<&'a str>,
    note: Option<&'a str>,
}

fn main() {
    let mut arguments = std::env::args().skip(1);
    let onboarding = arguments.any(|a| a == "--onboarding");
    let chemin = arguments
        .find(|a| !a.starts_with("--"))
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(if onboarding { CHEMIN_ONBOARDING } else { CHEMIN_PAR_DEFAUT }));

    if let Err(e) = generer(&chemin, onboarding) {
        eprintln!("seed_demo : {e}");
        std::process::exit(1);
    }
    println!("Base de démonstration générée : {}", chemin.display());
}

fn generer(chemin: &Path, onboarding: bool) -> Result<(), String> {
    if let Some(parent) = chemin.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Création de {} impossible : {e}", parent.display()))?;
    }
    for suffixe in ["", "-wal", "-shm"] {
        let _ = std::fs::remove_file(format!("{}{suffixe}", chemin.display()));
    }

    let pool = DatabasePool::open(chemin)?;
    migrations::run_migrations(&pool.conn)?;

    inserer_parametres(&pool, onboarding)?;
    if onboarding {
        return Ok(());
    }

    for regle in regles() {
        inserer_regle(&pool, &regle)?;
    }
    for operation in operations() {
        inserer_operation(&pool, &operation)?;
    }
    Ok(())
}

fn horodatage(date: &str) -> String {
    format!("{date}T12:00:00Z")
}

fn inserer_parametres(pool: &DatabasePool, onboarding: bool) -> Result<(), String> {
    let (solde, decouvert, solde_maj, onboarding_fait, dernier_export) = if onboarding {
        (
            0_i64,
            0_i64,
            "2026-08-09T08:00:00Z".to_string(),
            0_i64,
            None,
        )
    } else {
        // Solde négatif : le compte est déjà dans le découvert autorisé, et la
        // prévision de fin de mois reste dans le rouge (−250,00 € sur 500 €
        // autorisés) : état « Attention » avec jauge de découvert.
        (-26_990_i64, 50_000_i64, "2026-08-03T09:12:00Z".to_string(), 1_i64, Some("2026-07-30T20:05:00Z".to_string()))
    };
    pool.conn
        .execute(
            "INSERT INTO app_settings
                (id, current_balance_cents, overdraft_limit_cents, currency_code, locale,
                 theme, balance_updated_at, onboarding_completed, last_export_date,
                 created_at, updated_at)
             VALUES (1, ?1, ?2, 'EUR', 'fr-FR', 'light', ?3, ?4, ?5, ?6, ?6)",
            params![solde, decouvert, solde_maj, onboarding_fait, dernier_export, horodatage("2026-01-01")],
        )
        .map_err(|e| format!("Insertion des paramètres : {e}"))?;
    Ok(())
}

fn inserer_regle(pool: &DatabasePool, regle: &Regle<'_>) -> Result<(), String> {
    let (debut_annee, debut_mois) = regle.debut;
    let (fin_annee, fin_mois) = regle.fin.unwrap_or((0, 0));
    let fin_annee = if fin_annee == 0 { None } else { Some(fin_annee) };
    let fin_mois = if fin_mois == 0 { None } else { Some(fin_mois) };
    let debut_date = format!("{debut_annee:04}-{debut_mois:02}-01");
    pool.conn
        .execute(
            "INSERT INTO recurring_rules
                (id, kind, label, amount_cents, category_id, day_of_month,
                 start_year, start_month, end_year, end_month, note, is_active,
                 created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?13)",
            params![
                regle.id,
                regle.genre,
                regle.libelle,
                regle.montant_centimes,
                regle.categorie,
                regle.jour,
                debut_annee,
                debut_mois,
                fin_annee,
                fin_mois,
                regle.note,
                i64::from(regle.active),
                horodatage(&debut_date),
            ],
        )
        .map_err(|e| format!("Insertion de la règle {} : {e}", regle.id))?;
    Ok(())
}

fn inserer_operation(pool: &DatabasePool, operation: &Operation<'_>) -> Result<(), String> {
    pool.conn
        .execute(
            "INSERT INTO transactions
                (id, kind, label, amount_cents, transaction_date, status,
                 category_id, recurring_rule_id, note, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?10)",
            params![
                identifiant(operation.date, operation.libelle),
                operation.genre,
                operation.libelle,
                operation.montant_centimes,
                operation.date,
                operation.statut,
                operation.categorie,
                operation.regle,
                operation.note,
                horodatage(operation.date),
            ],
        )
        .map_err(|e| format!("Insertion de « {} » : {e}", operation.libelle))?;
    Ok(())
}

/// Identifiant stable et déterministe : la base est reproductible d'un run à
/// l'autre, sans dépendre du générateur aléatoire.
fn identifiant(date: &str, libelle: &str) -> String {
    let assaini: String = libelle
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .take(12)
        .collect();
    format!("demo-{date}-{}", assaini.to_lowercase())
}

/// Une dépense ordinaire (réalisée ou en attente).
fn operation(
    libelle: &'static str,
    montant_centimes: i64,
    date: &'static str,
    statut: &'static str,
    categorie: &'static str,
) -> Operation<'static> {
    Operation {
        genre: "expense",
        libelle,
        montant_centimes,
        date,
        statut,
        categorie,
        regle: None,
        note: None,
    }
}

/// Un revenu ordinaire (réalisé ou en attente).
fn operation_revenu(
    libelle: &'static str,
    montant_centimes: i64,
    date: &'static str,
    statut: &'static str,
    categorie: &'static str,
) -> Operation<'static> {
    Operation {
        genre: "income",
        libelle,
        montant_centimes,
        date,
        statut,
        categorie,
        regle: None,
        note: None,
    }
}

/// Une occurrence de règle mensuelle (dépense).
fn operation_regle(
    libelle: &'static str,
    montant_centimes: i64,
    date: &'static str,
    statut: &'static str,
    categorie: &'static str,
    regle: &'static str,
) -> Operation<'static> {
    let mut op = operation(libelle, montant_centimes, date, statut, categorie);
    op.regle = Some(regle);
    op
}

/// Une occurrence de règle mensuelle (revenu).
fn operation_revenu_regle(
    libelle: &'static str,
    montant_centimes: i64,
    date: &'static str,
    statut: &'static str,
    categorie: &'static str,
    regle: &'static str,
) -> Operation<'static> {
    let mut op = operation_revenu(libelle, montant_centimes, date, statut, categorie);
    op.regle = Some(regle);
    op
}

fn operation_avec_note(
    genre: &'static str,
    libelle: &'static str,
    montant_centimes: i64,
    date: &'static str,
    statut: &'static str,
    categorie: &'static str,
    note: &'static str,
) -> Operation<'static> {
    let mut op = operation(libelle, montant_centimes, date, statut, categorie);
    op.genre = genre;
    op.note = Some(note);
    op
}

/// Règles mensuelles du « carnet » fictif.
fn regles() -> Vec<Regle<'static>> {
    vec![
        Regle {
            id: "r-salaire",
            genre: "income",
            libelle: "Salaire",
            montant_centimes: 238_000,
            categorie: "salaire",
            jour: 28,
            debut: (2026, 1),
            fin: None,
            active: true,
            note: Some("Virement employeur"),
        },
        Regle {
            id: "r-loyer",
            genre: "expense",
            libelle: "Loyer",
            montant_centimes: 85_000,
            categorie: "logement",
            jour: 3,
            debut: (2026, 1),
            fin: None,
            active: true,
            note: Some("Résidence Les Tilleuls"),
        },
        Regle {
            id: "r-epargne",
            genre: "expense",
            libelle: "Épargne mensuelle",
            montant_centimes: 15_000,
            categorie: "epargne",
            jour: 2,
            debut: (2026, 1),
            fin: None,
            active: true,
            note: None,
        },
        Regle {
            id: "r-telephone",
            genre: "expense",
            libelle: "Forfait téléphone",
            montant_centimes: 2_499,
            categorie: "telephone",
            jour: 12,
            debut: (2026, 1),
            fin: None,
            active: true,
            note: None,
        },
        Regle {
            id: "r-energie",
            genre: "expense",
            libelle: "Électricité",
            montant_centimes: 6_823,
            categorie: "energie",
            jour: 11,
            debut: (2026, 1),
            fin: None,
            active: true,
            note: None,
        },
        Regle {
            id: "r-assurance",
            genre: "expense",
            libelle: "Assurance habitation",
            montant_centimes: 1_850,
            categorie: "assurance",
            jour: 8,
            debut: (2026, 1),
            fin: None,
            active: true,
            note: None,
        },
        Regle {
            id: "r-sport",
            genre: "expense",
            libelle: "Salle de sport",
            montant_centimes: 2_990,
            categorie: "sport",
            jour: 20,
            debut: (2026, 1),
            fin: None,
            active: true,
            note: None,
        },
        Regle {
            id: "r-musique",
            genre: "expense",
            libelle: "Musique en streaming",
            montant_centimes: 1_099,
            categorie: "abonnements",
            jour: 7,
            debut: (2026, 1),
            fin: None,
            active: true,
            note: None,
        },
        Regle {
            id: "r-internet",
            genre: "expense",
            libelle: "Fibre internet",
            montant_centimes: 3_999,
            categorie: "internet",
            jour: 15,
            debut: (2026, 2),
            fin: None,
            active: true,
            note: None,
        },
        Regle {
            id: "r-video",
            genre: "expense",
            libelle: "Plateforme vidéo",
            montant_centimes: 1_299,
            categorie: "abonnements",
            jour: 5,
            debut: (2026, 3),
            fin: None,
            active: true,
            note: None,
        },
        Regle {
            id: "r-cafe",
            genre: "expense",
            libelle: "Café en livraison",
            montant_centimes: 1_500,
            categorie: "restaurants",
            jour: 4,
            debut: (2026, 2),
            fin: None,
            active: false,
            note: None,
        },
    ]
}

/// Toutes les opérations du carnet fictif, mois par mois, de janvier à août
/// 2026 (mois courant au moment de la génération des captures).
///
/// Les occurrences issues d'une règle portent l'identifiant de la règle :
/// l'application les reconnaît et ne les matérialise pas une seconde fois.
fn operations() -> Vec<Operation<'static>> {
    let mut toutes = Vec::new();

    // ── Janvier 2026 ───────────────────────────────────────────────────────
    toutes.extend([
        operation_avec_note("income", "Salaire", 238_000, "2026-01-28", "completed", "salaire", "Virement employeur"),
        operation_revenu("Remboursement CPAM", 2_240, "2026-01-15", "completed", "remboursement"),
        operation_regle("Épargne mensuelle", 15_000, "2026-01-02", "completed", "epargne", "r-epargne"),
        operation_regle("Loyer", 85_000, "2026-01-03", "completed", "logement", "r-loyer"),
        operation_regle("Musique en streaming", 1_099, "2026-01-07", "completed", "abonnements", "r-musique"),
        operation_regle("Assurance habitation", 1_850, "2026-01-08", "completed", "assurance", "r-assurance"),
        operation_regle("Électricité", 6_823, "2026-01-11", "completed", "energie", "r-energie"),
        operation_regle("Forfait téléphone", 2_499, "2026-01-12", "completed", "telephone", "r-telephone"),
        operation_regle("Salle de sport", 2_990, "2026-01-20", "completed", "sport", "r-sport"),
        operation("Supermarché", 9_650, "2026-01-05", "completed", "alimentation"),
        operation("Supermarché", 6_120, "2026-01-09", "completed", "alimentation"),
        operation("Marché", 1_260, "2026-01-15", "completed", "alimentation"),
        operation("Supermarché", 8_475, "2026-01-22", "completed", "alimentation"),
        operation("Restaurant", 3_240, "2026-01-10", "completed", "restaurants"),
        operation("Essence", 5_130, "2026-01-17", "completed", "transport"),
        operation("Cinéma", 2_350, "2026-01-24", "completed", "loisirs"),
    ]);

    // ── Février 2026 ───────────────────────────────────────────────────────
    toutes.extend([
        operation_revenu("Salaire", 238_000, "2026-02-28", "completed", "salaire"),
        operation_revenu("Vente d'occasion", 2_000, "2026-02-10", "completed", "vente"),
        operation_regle("Épargne mensuelle", 15_000, "2026-02-02", "completed", "epargne", "r-epargne"),
        operation_regle("Loyer", 85_000, "2026-02-03", "completed", "logement", "r-loyer"),
        operation_regle("Musique en streaming", 1_099, "2026-02-07", "completed", "abonnements", "r-musique"),
        operation_regle("Assurance habitation", 1_850, "2026-02-08", "completed", "assurance", "r-assurance"),
        operation_regle("Électricité", 6_823, "2026-02-11", "completed", "energie", "r-energie"),
        operation_regle("Forfait téléphone", 2_499, "2026-02-12", "completed", "telephone", "r-telephone"),
        operation_regle("Fibre internet", 3_999, "2026-02-15", "completed", "internet", "r-internet"),
        operation_regle("Salle de sport", 2_990, "2026-02-20", "completed", "sport", "r-sport"),
        operation("Supermarché", 7_480, "2026-02-04", "completed", "alimentation"),
        operation("Supermarché", 8_835, "2026-02-11", "completed", "alimentation"),
        operation("Supermarché", 5_260, "2026-02-18", "completed", "alimentation"),
        operation("Supermarché", 9_105, "2026-02-25", "completed", "alimentation"),
        operation("Saint-Valentin", 4_500, "2026-02-14", "completed", "restaurants"),
        operation("Essence", 4_790, "2026-02-07", "completed", "transport"),
        operation("Pharmacie", 1_520, "2026-02-20", "completed", "sante"),
        operation("Librairie", 1_890, "2026-02-16", "completed", "livres"),
    ]);

    // ── Mars 2026 ──────────────────────────────────────────────────────────
    toutes.extend([
        operation_revenu("Salaire", 238_000, "2026-03-28", "completed", "salaire"),
        operation_revenu("Remboursement CPAM", 3_570, "2026-03-12", "completed", "remboursement"),
        operation_regle("Épargne mensuelle", 15_000, "2026-03-02", "completed", "epargne", "r-epargne"),
        operation_regle("Loyer", 85_000, "2026-03-03", "completed", "logement", "r-loyer"),
        operation_regle("Plateforme vidéo", 1_299, "2026-03-05", "completed", "abonnements", "r-video"),
        operation_regle("Musique en streaming", 1_099, "2026-03-07", "completed", "abonnements", "r-musique"),
        operation_regle("Assurance habitation", 1_850, "2026-03-08", "completed", "assurance", "r-assurance"),
        operation_regle("Électricité", 6_823, "2026-03-11", "completed", "energie", "r-energie"),
        operation_regle("Forfait téléphone", 2_499, "2026-03-12", "completed", "telephone", "r-telephone"),
        operation_regle("Fibre internet", 3_999, "2026-03-15", "completed", "internet", "r-internet"),
        operation_regle("Salle de sport", 2_990, "2026-03-20", "completed", "sport", "r-sport"),
        operation("Supermarché", 10_540, "2026-03-06", "completed", "alimentation"),
        operation("Supermarché", 6_285, "2026-03-13", "completed", "alimentation"),
        operation("Supermarché", 7_730, "2026-03-20", "completed", "alimentation"),
        operation("Supermarché", 8_890, "2026-03-27", "completed", "alimentation"),
        operation("Restaurant", 3_980, "2026-03-14", "completed", "restaurants"),
        operation("Essence", 5_420, "2026-03-21", "completed", "transport"),
        operation("Veste", 4_999, "2026-03-18", "completed", "vetements"),
        operation("Pharmacie", 860, "2026-03-08", "completed", "sante"),
        operation("Livres techniques", 2_750, "2026-03-10", "completed", "education"),
    ]);

    // ── Avril 2026 ─────────────────────────────────────────────────────────
    toutes.extend([
        operation_revenu("Salaire", 238_000, "2026-04-28", "completed", "salaire"),
        operation_revenu("Vente d'occasion", 1_500, "2026-04-05", "completed", "vente"),
        operation_revenu("Remboursement CPAM", 1_820, "2026-04-20", "completed", "remboursement"),
        operation_regle("Épargne mensuelle", 15_000, "2026-04-02", "completed", "epargne", "r-epargne"),
        operation_regle("Loyer", 85_000, "2026-04-03", "completed", "logement", "r-loyer"),
        operation_regle("Plateforme vidéo", 1_299, "2026-04-05", "completed", "abonnements", "r-video"),
        operation_regle("Musique en streaming", 1_099, "2026-04-07", "completed", "abonnements", "r-musique"),
        operation_regle("Assurance habitation", 1_850, "2026-04-08", "completed", "assurance", "r-assurance"),
        operation_regle("Électricité", 6_823, "2026-04-11", "completed", "energie", "r-energie"),
        operation_regle("Forfait téléphone", 2_499, "2026-04-12", "completed", "telephone", "r-telephone"),
        operation_regle("Fibre internet", 3_999, "2026-04-15", "completed", "internet", "r-internet"),
        operation_regle("Salle de sport", 2_990, "2026-04-20", "completed", "sport", "r-sport"),
        operation("Supermarché", 9_215, "2026-04-04", "completed", "alimentation"),
        operation("Supermarché", 5_830, "2026-04-11", "completed", "alimentation"),
        operation("Supermarché", 10_175, "2026-04-18", "completed", "alimentation"),
        operation("Restaurant", 5_260, "2026-04-06", "completed", "restaurants"),
        operation("Essence", 4_490, "2026-04-09", "completed", "transport"),
        operation("Pharmacie", 2_340, "2026-04-15", "completed", "sante"),
        operation_avec_note("expense", "Cadeau anniversaire", 3_000, "2026-04-12", "completed", "cadeaux", "Pour Léa"),
        operation("Billet de train", 4_280, "2026-04-20", "completed", "voyages"),
    ]);

    // ── Mai 2026 ───────────────────────────────────────────────────────────
    toutes.extend([
        operation_revenu("Salaire", 238_000, "2026-05-28", "completed", "salaire"),
        operation_revenu("Prime de mai", 20_000, "2026-05-10", "completed", "prime"),
        operation_revenu("Remboursement CPAM", 1_650, "2026-05-22", "completed", "remboursement"),
        operation_regle("Épargne mensuelle", 15_000, "2026-05-02", "completed", "epargne", "r-epargne"),
        operation_regle("Loyer", 85_000, "2026-05-03", "completed", "logement", "r-loyer"),
        operation_regle("Plateforme vidéo", 1_299, "2026-05-05", "completed", "abonnements", "r-video"),
        operation_regle("Musique en streaming", 1_099, "2026-05-07", "completed", "abonnements", "r-musique"),
        operation_regle("Assurance habitation", 1_850, "2026-05-08", "completed", "assurance", "r-assurance"),
        operation_regle("Électricité", 6_823, "2026-05-11", "completed", "energie", "r-energie"),
        operation_regle("Forfait téléphone", 2_499, "2026-05-12", "completed", "telephone", "r-telephone"),
        operation_regle("Fibre internet", 3_999, "2026-05-15", "completed", "internet", "r-internet"),
        operation_regle("Salle de sport", 2_990, "2026-05-20", "completed", "sport", "r-sport"),
        operation("Supermarché", 8_620, "2026-05-02", "completed", "alimentation"),
        operation("Supermarché", 4_755, "2026-05-09", "completed", "alimentation"),
        operation("Supermarché", 11_080, "2026-05-16", "completed", "alimentation"),
        operation("Restaurant", 6_840, "2026-05-17", "completed", "restaurants"),
        operation("Essence", 4_930, "2026-05-08", "completed", "transport"),
        operation("Concert", 3_500, "2026-05-23", "completed", "loisirs"),
        operation("Vétérinaire", 3_290, "2026-05-18", "completed", "animaux"),
        operation("Sneakers", 3_999, "2026-05-25", "completed", "vetements"),
        operation("Quincaillerie", 2_130, "2026-05-24", "completed", "bricolage"),
    ]);

    // ── Juin 2026 ──────────────────────────────────────────────────────────
    toutes.extend([
        operation_revenu("Salaire", 238_000, "2026-06-28", "completed", "salaire"),
        operation_revenu("Remboursement CPAM", 2_980, "2026-06-15", "completed", "remboursement"),
        operation_revenu("Vente d'occasion", 1_200, "2026-06-25", "completed", "vente"),
        operation_regle("Épargne mensuelle", 15_000, "2026-06-02", "completed", "epargne", "r-epargne"),
        operation_regle("Loyer", 85_000, "2026-06-03", "completed", "logement", "r-loyer"),
        operation_regle("Plateforme vidéo", 1_299, "2026-06-05", "completed", "abonnements", "r-video"),
        operation_regle("Musique en streaming", 1_099, "2026-06-07", "completed", "abonnements", "r-musique"),
        operation_regle("Assurance habitation", 1_850, "2026-06-08", "completed", "assurance", "r-assurance"),
        operation_regle("Électricité", 6_823, "2026-06-11", "completed", "energie", "r-energie"),
        operation_regle("Forfait téléphone", 2_499, "2026-06-12", "completed", "telephone", "r-telephone"),
        operation_regle("Fibre internet", 3_999, "2026-06-15", "completed", "internet", "r-internet"),
        operation_regle("Salle de sport", 2_990, "2026-06-20", "completed", "sport", "r-sport"),
        operation("Supermarché", 9_870, "2026-06-03", "completed", "alimentation"),
        operation("Supermarché", 6_425, "2026-06-10", "completed", "alimentation"),
        operation("Supermarché", 8_390, "2026-06-17", "completed", "alimentation"),
        operation("Restaurant", 4_730, "2026-06-12", "completed", "restaurants"),
        operation("Essence", 5_780, "2026-06-06", "completed", "transport"),
        operation("Sortie scolaire", 2_490, "2026-06-20", "completed", "enfants"),
        operation("Réservation week-end", 15_000, "2026-06-26", "completed", "voyages"),
        operation("Pharmacie", 940, "2026-06-19", "completed", "sante"),
    ]);

    // ── Juillet 2026 (mois précédent : sert aux comparaisons) ─────────────
    toutes.extend([
        operation_revenu("Salaire", 238_000, "2026-07-28", "completed", "salaire"),
        operation_revenu("Remboursement CPAM", 1_820, "2026-07-01", "completed", "remboursement"),
        operation_revenu("Remboursement colocation", 1_500, "2026-07-15", "completed", "remboursement"),
        operation_revenu("Vente d'occasion", 2_500, "2026-07-10", "completed", "vente"),
        operation_regle("Épargne mensuelle", 15_000, "2026-07-02", "completed", "epargne", "r-epargne"),
        operation_regle("Loyer", 85_000, "2026-07-03", "completed", "logement", "r-loyer"),
        operation_regle("Plateforme vidéo", 1_299, "2026-07-05", "completed", "abonnements", "r-video"),
        operation_regle("Musique en streaming", 1_099, "2026-07-07", "completed", "abonnements", "r-musique"),
        operation_regle("Assurance habitation", 1_850, "2026-07-08", "completed", "assurance", "r-assurance"),
        operation_regle("Électricité", 6_823, "2026-07-11", "completed", "energie", "r-energie"),
        operation_regle("Forfait téléphone", 2_499, "2026-07-12", "completed", "telephone", "r-telephone"),
        operation_regle("Fibre internet", 3_999, "2026-07-15", "completed", "internet", "r-internet"),
        operation_regle("Salle de sport", 2_990, "2026-07-20", "completed", "sport", "r-sport"),
        operation("Supermarché", 8_520, "2026-07-04", "completed", "alimentation"),
        operation("Supermarché", 11_260, "2026-07-11", "completed", "alimentation"),
        operation("Supermarché", 7_635, "2026-07-18", "completed", "alimentation"),
        operation("Supermarché", 9_810, "2026-07-25", "completed", "alimentation"),
        operation("Restaurant", 3_850, "2026-07-09", "completed", "restaurants"),
        operation("Restaurant", 6_280, "2026-07-14", "completed", "restaurants"),
        operation("Essence", 4_870, "2026-07-18", "completed", "transport"),
        operation("Pharmacie", 960, "2026-07-10", "completed", "sante"),
        operation("Librairie", 2_490, "2026-07-16", "completed", "livres"),
        operation("Concert en plein air", 4_500, "2026-07-22", "completed", "loisirs"),
        operation("Cadeau", 3_000, "2026-07-24", "completed", "cadeaux"),
        operation("Robe", 5_999, "2026-07-25", "completed", "vetements"),
        operation("Contrôle technique", 12_000, "2026-07-27", "completed", "transport"),
        operation("Location week-end Airbnb", 38_000, "2026-07-20", "completed", "voyages"),
        operation("Nouveau téléphone", 59_900, "2026-07-11", "completed", "telephone"),
    ]);

    // ── Août 2026 (mois courant des captures) ─────────────────────────────
    toutes.extend([
        operation_revenu("Remboursement CPAM", 4_560, "2026-08-01", "completed", "remboursement"),
        operation_revenu("Vente d'occasion", 3_000, "2026-08-05", "completed", "vente"),
        operation_revenu("Remboursement CPAM", 1_290, "2026-08-08", "completed", "remboursement"),
        operation_revenu_regle("Salaire", 238_000, "2026-08-28", "pending", "salaire", "r-salaire"),
        operation_revenu("Prime de vacances", 15_000, "2026-08-20", "pending", "prime"),
        operation_regle("Épargne mensuelle", 15_000, "2026-08-02", "completed", "epargne", "r-epargne"),
        operation_regle("Loyer", 85_000, "2026-08-03", "completed", "logement", "r-loyer"),
        operation("Supermarché", 7_842, "2026-08-04", "completed", "alimentation"),
        operation_regle("Plateforme vidéo", 1_299, "2026-08-05", "completed", "abonnements", "r-video"),
        operation("Marché", 890, "2026-08-06", "completed", "alimentation"),
        operation("Pharmacie", 1_235, "2026-08-06", "completed", "sante"),
        operation("Supermarché", 9_315, "2026-08-07", "completed", "alimentation"),
        operation_regle("Musique en streaming", 1_099, "2026-08-07", "completed", "abonnements", "r-musique"),
        operation_regle("Assurance habitation", 1_850, "2026-08-08", "completed", "assurance", "r-assurance"),
        operation("Cinéma", 2_400, "2026-08-08", "completed", "loisirs"),
        operation("Déjeuner entre amis", 2_480, "2026-08-09", "completed", "restaurants"),
        operation_regle("Électricité", 6_823, "2026-08-11", "pending", "energie", "r-energie"),
        operation_regle("Forfait téléphone", 2_499, "2026-08-12", "pending", "telephone", "r-telephone"),
        operation_regle("Fibre internet", 3_999, "2026-08-15", "pending", "internet", "r-internet"),
        operation("Courses de mi-mois", 8_500, "2026-08-16", "pending", "alimentation"),
        operation("Essence", 6_200, "2026-08-18", "pending", "transport"),
        operation("Fournitures scolaires", 9_500, "2026-08-19", "pending", "enfants"),
        operation_regle("Salle de sport", 2_990, "2026-08-20", "pending", "sport", "r-sport"),
        operation("Cadeau anniversaire", 4_000, "2026-08-20", "pending", "cadeaux"),
        operation("Réparation de la voiture", 32_000, "2026-08-21", "pending", "transport"),
        operation("Concert", 5_500, "2026-08-22", "pending", "loisirs"),
        operation("Veste", 7_999, "2026-08-23", "pending", "vetements"),
        operation("Dîner d'anniversaire", 6_000, "2026-08-24", "pending", "restaurants"),
        operation("Location villa en Bretagne", 59_000, "2026-08-25", "pending", "voyages"),
        operation("Billets d'avion", 32_000, "2026-08-26", "pending", "voyages"),
        operation("Chaussures de marche", 9_500, "2026-08-26", "pending", "vetements"),
        operation("Nouvelles lunettes", 15_000, "2026-08-27", "pending", "sante"),
        operation("Gros plein d'essence", 7_000, "2026-08-27", "pending", "transport"),
        operation("Courses de fin de mois", 11_000, "2026-08-28", "pending", "alimentation"),
        operation("Sortie au parc d'attractions", 9_000, "2026-08-29", "pending", "loisirs"),
        operation("Restaurant en famille", 9_000, "2026-08-29", "pending", "restaurants"),
        operation("Coiffeur", 3_500, "2026-08-30", "pending", "bien-etre"),
    ]);

    toutes
}
