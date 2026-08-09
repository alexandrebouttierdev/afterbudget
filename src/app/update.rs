use iced::Task;

use crate::core::db::pool::DatabasePool;
use crate::domaine::argent::Money;
use crate::domaine::transaction::TransactionKind;
use crate::modules::import_export::commandes as io_cmd;
use crate::modules::onboarding::commandes as onboarding_cmd;
use crate::modules::onboarding::dtos::TerminerOnboardingDto;
use crate::modules::parametres::repository as params_repo;
use crate::modules::parametres::service as parametres_service;
use crate::modules::recurrences::commandes as recurrences_cmd;
use crate::modules::recurrences::dtos::CreerRecurrenceDto;
use crate::modules::transactions::commandes as tx_cmd;
use crate::modules::transactions::dtos::{CreerTransactionDto, ModifierTransactionDto};
use crate::ui::composants::champ;
use crate::ui::theme::mise_en_page::MiseEnPage;
use chrono::Datelike;

use super::message::{Message, Screen};
use super::state::{
    mois_precedent, mois_suivant, AppState, Notification, ThemeMode, TransactionFormState,
};

/// Fonction principale de mise à jour de l'état
pub fn update(state: &mut AppState, message: Message) -> Task<Message> {
    match message {
        // ---- Navigation ----
        Message::NavigateTo(screen) => {
            state.screen = screen;
            Task::none()
        }
        Message::PreviousMonth => {
            if state.current_month == 1 {
                state.current_month = 12;
                state.current_year -= 1;
            } else {
                state.current_month -= 1;
            }
            state.load_month_data().ok();
            Task::none()
        }
        Message::NextMonth => {
            if state.current_month == 12 {
                state.current_month = 1;
                state.current_year += 1;
            } else {
                state.current_month += 1;
            }
            state.load_month_data().ok();
            Task::none()
        }
        Message::GoToCurrentMonth => {
            let now = chrono::Utc::now();
            state.current_year = now.year();
            state.current_month = now.month();
            state.load_month_data().ok();
            Task::none()
        }

        // ---- Onboarding ----
        Message::SetOnboardingBalance(val) => {
            state.onboarding_balance_str = val;
            state.onboarding_error = None;
            Task::none()
        }
        Message::SetOnboardingOverdraft(val) => {
            state.onboarding_overdraft_str = val;
            state.onboarding_error = None;
            Task::none()
        }
        Message::SetOnboardingCurrency(val) => {
            state.onboarding_currency = val;
            Task::none()
        }
        Message::CompleteOnboarding => {
            let dto = TerminerOnboardingDto {
                solde_actuel: state.onboarding_balance_str.clone(),
                decouvert_autorise: state.onboarding_overdraft_str.clone(),
                devise: state.onboarding_currency.clone(),
            };

            let Some(db) = state.db.as_ref() else {
                return Task::none();
            };
            match onboarding_cmd::terminer(db, &dto) {
                Ok(()) => match state.load_data() {
                    Ok(()) => {
                        state.notification =
                            Some(Notification::succes("Bienvenue dans AfterBudget !"));
                    }
                    Err(e) => {
                        state.notification = Some(Notification::erreur(format!(
                            "Paramètres enregistrés, mais rechargement impossible : {e}"
                        )));
                    }
                },
                Err(e) => {
                    state.onboarding_error = Some(e);
                }
            }
            Task::none()
        }

        // ---- Paramètres ----
        Message::SetCurrentBalance(val) => {
            state.settings_balance_str = val;
            state.settings_error = None;
            Task::none()
        }
        Message::OpenBalanceEdit => {
            state.edition_solde_ouverte = true;
            state.settings_error = None;
            state.settings_balance_str = state
                .settings
                .as_ref()
                .map(|parametres| montant_editable(parametres.current_balance))
                .unwrap_or_default();
            Task::none()
        }
        Message::CancelBalanceEdit => {
            state.edition_solde_ouverte = false;
            state.settings_error = None;
            state.settings_balance_str.clear();
            Task::none()
        }
        Message::UpdateBalance => {
            // Un solde de compte peut être négatif : on peut déjà être à
            // découvert. Seule la lisibilité de la valeur est exigée.
            let balance = match Money::from_input(&state.settings_balance_str) {
                Ok(m) => m,
                Err(e) => {
                    state.settings_error = Some(e);
                    return Task::none();
                }
            };

            let Some(db) = state.db.as_ref() else {
                return Task::none();
            };
            if let Err(e) = parametres_service::mettre_a_jour_solde(db, balance) {
                state.notification =
                    Some(Notification::erreur(format!("Solde non enregistré : {e}")));
                return Task::none();
            }
            state.settings = parametres_service::obtenir_parametres(db).ok().flatten();

            if let Err(e) = state.load_month_data() {
                state.notification = Some(Notification::erreur(format!(
                    "Solde enregistré, mais rechargement impossible : {e}"
                )));
                return Task::none();
            }

            state.notification = Some(Notification::succes("Solde mis à jour."));
            state.settings_balance_str.clear();
            state.edition_solde_ouverte = false;
            Task::none()
        }
        Message::SetOverdraftLimit(val) => {
            state.settings_overdraft_str = val;
            state.settings_error = None;
            Task::none()
        }
        Message::UpdateOverdraft => {
            let overdraft = match Money::from_input(&state.settings_overdraft_str) {
                Ok(m) => {
                    if m.cents < 0 {
                        state.settings_error =
                            Some("Le découvert ne peut pas être négatif.".into());
                        return Task::none();
                    }
                    m
                }
                Err(e) => {
                    state.settings_error = Some(e);
                    return Task::none();
                }
            };

            let Some(db) = state.db.as_ref() else {
                return Task::none();
            };
            if let Err(e) = parametres_service::mettre_a_jour_decouvert(db, overdraft) {
                state.notification =
                    Some(Notification::erreur(format!("Découvert non enregistré : {e}")));
                return Task::none();
            }
            state.settings = parametres_service::obtenir_parametres(db).ok().flatten();

            if let Err(e) = state.load_month_data() {
                state.notification = Some(Notification::erreur(format!(
                    "Découvert enregistré, mais rechargement impossible : {e}"
                )));
                return Task::none();
            }

            state.notification = Some(Notification::succes("Découvert mis à jour."));
            state.settings_overdraft_str.clear();
            Task::none()
        }
        Message::SetTheme(theme) => appliquer_theme(state, ThemeMode::depuis_cle(&theme)),
        Message::ToggleTheme => appliquer_theme(state, state.theme_mode.inverse()),
        Message::SetCurrency(cur) => {
            let Some(db) = state.db.as_ref() else {
                return Task::none();
            };
            if let Err(e) = parametres_service::mettre_a_jour_devise(db, &cur) {
                state.notification =
                    Some(Notification::erreur(format!("Devise non enregistrée : {e}")));
                return Task::none();
            }
            state.settings = parametres_service::obtenir_parametres(db).ok().flatten();

            if let Err(e) = state.load_month_data() {
                state.notification = Some(Notification::erreur(format!(
                    "Devise enregistrée, mais rechargement impossible : {e}"
                )));
                return Task::none();
            }

            state.notification = Some(Notification::succes(format!("Devise changée pour {cur}.")));
            Task::none()
        }

        // ---- Transactions ----
        Message::OpenAddIncome => {
            let cat = state.default_category_for(TransactionKind::Income);
            let mut form = TransactionFormState {
                kind: TransactionKind::Income,
                ..Default::default()
            };
            if let Some(c) = cat {
                form.category_id = c.id.clone();
            }
            state.transaction_form = form;
            state.show_transaction_form = true;
            focaliser(champ::ID_MONTANT)
        }
        Message::OpenAddExpense => {
            let cat = state.default_category_for(TransactionKind::Expense);
            let mut form = TransactionFormState {
                kind: TransactionKind::Expense,
                ..Default::default()
            };
            if let Some(c) = cat {
                form.category_id = c.id.clone();
            }
            state.transaction_form = form;
            state.show_transaction_form = true;
            focaliser(champ::ID_MONTANT)
        }
        Message::OpenEditTransaction(id) => {
            if let Some(ref db) = state.db {
                if let Ok(Some(tx)) = tx_cmd::trouver(db, &id) {
                    state.transaction_form = TransactionFormState {
                        is_edit: true,
                        edit_id: Some(tx.id.clone()),
                        kind: tx.kind,
                        label: tx.label.clone(),
                        amount_str: montant_editable(tx.amount),
                        date_str: crate::core::utils::format_date_saisie(&tx.transaction_date),
                        calendrier_ouvert: false,
                        calendrier_annee: tx.transaction_date.year(),
                        calendrier_mois: tx.transaction_date.month(),
                        category_id: tx.category_id.clone(),
                        status: tx.status,
                        note: tx.note.clone().unwrap_or_default(),
                        // L'édition porte sur l'occurrence, jamais sur la règle.
                        recurrent: false,
                        label_error: None,
                        amount_error: None,
                        date_error: None,
                        category_error: None,
                    };
                    state.show_transaction_form = true;
                }
            }
            focaliser(champ::ID_MONTANT)
        }
        Message::CloseTransactionForm => {
            state.show_transaction_form = false;
            state.transaction_form = TransactionFormState::default();
            Task::none()
        }
        Message::SetFormKind(kind) => {
            if state.transaction_form.kind != kind {
                state.transaction_form.kind = kind;
                // La catégorie courante appartient à l'autre sens : on repart
                // sur la catégorie par défaut du nouveau sens.
                state.transaction_form.category_id = state
                    .default_category_for(kind)
                    .map(|c| c.id.clone())
                    .unwrap_or_default();
                state.transaction_form.category_error = None;
            }
            Task::none()
        }
        Message::SetFormLabel(val) => {
            state.transaction_form.label = val;
            state.transaction_form.label_error = None;
            Task::none()
        }
        Message::SetFormAmount(val) => {
            state.transaction_form.amount_str = val;
            state.transaction_form.amount_error = None;
            Task::none()
        }
        Message::SetFormDate(val) => {
            state.transaction_form.date_str = crate::core::utils::filtrer_saisie_date(&val);
            state.transaction_form.date_error = None;
            // Le calendrier suit la saisie dès qu'elle devient exploitable.
            if let Some(date) = state.transaction_form.date() {
                state.transaction_form.calendrier_annee = date.year();
                state.transaction_form.calendrier_mois = date.month();
            }
            Task::none()
        }
        Message::ToggleFormCalendar => {
            let formulaire = &mut state.transaction_form;
            formulaire.calendrier_ouvert = !formulaire.calendrier_ouvert;
            if formulaire.calendrier_ouvert {
                // À l'ouverture, le calendrier se cale sur la date saisie, ou
                // sur le mois courant si la saisie est incomplète.
                let repere = formulaire
                    .date()
                    .unwrap_or_else(|| chrono::Utc::now().date_naive());
                formulaire.calendrier_annee = repere.year();
                formulaire.calendrier_mois = repere.month();
            }
            Task::none()
        }
        Message::FormCalendarPreviousMonth => {
            let formulaire = &mut state.transaction_form;
            let (annee, mois) =
                mois_precedent(formulaire.calendrier_annee, formulaire.calendrier_mois);
            formulaire.calendrier_annee = annee;
            formulaire.calendrier_mois = mois;
            Task::none()
        }
        Message::FormCalendarNextMonth => {
            let formulaire = &mut state.transaction_form;
            let (annee, mois) =
                mois_suivant(formulaire.calendrier_annee, formulaire.calendrier_mois);
            formulaire.calendrier_annee = annee;
            formulaire.calendrier_mois = mois;
            Task::none()
        }
        Message::PickFormDate(date) => {
            let formulaire = &mut state.transaction_form;
            formulaire.date_str = crate::core::utils::format_date_saisie(&date);
            formulaire.date_error = None;
            formulaire.calendrier_ouvert = false;
            Task::none()
        }
        Message::SetFormCategory(val) => {
            state.transaction_form.category_id = val;
            state.transaction_form.category_error = None;
            Task::none()
        }
        Message::SetFormStatus(status) => {
            state.transaction_form.status = status;
            Task::none()
        }
        Message::SetFormNote(val) => {
            state.transaction_form.note = val;
            Task::none()
        }
        Message::SetFormRecurrent(actif) => {
            state.transaction_form.recurrent = actif;
            Task::none()
        }
        Message::DeleteRecurringRule(identifiant) => {
            let Some(db) = state.db.as_ref() else {
                return Task::none();
            };
            match recurrences_cmd::supprimer(db, &identifiant) {
                Ok(()) => {
                    if let Err(e) = state.load_data() {
                        state.notification = Some(Notification::erreur(format!(
                            "Récurrence supprimée, mais rechargement impossible : {e}"
                        )));
                        return Task::none();
                    }
                    state.notification = Some(Notification::succes(
                        "Récurrence supprimée. Les occurrences déjà créées sont conservées.",
                    ));
                }
                Err(e) => {
                    state.notification = Some(Notification::erreur(format!(
                        "Suppression impossible : {e}"
                    )));
                }
            }
            Task::none()
        }
        Message::SubmitTransactionForm => {
            if state.db.is_none() {
                return Task::none();
            }

            let db = state.db.as_ref().unwrap();
            let was_edit = state.transaction_form.is_edit;
            let form_kind = state.transaction_form.kind;
            let libelle = state.transaction_form.label.clone();
            let montant = state.transaction_form.amount_str.clone();
            let date = match state.transaction_form.date() {
                Some(date) => date.format("%Y-%m-%d").to_string(),
                None => {
                    state.transaction_form.date_error =
                        Some("Date incomplète ou inexistante (JJ/MM/AAAA).".into());
                    return Task::none();
                }
            };
            let categorie_id = state.transaction_form.category_id.clone();
            let type_transaction = state.transaction_form.kind.as_str().to_string();
            let statut = state.transaction_form.status.as_str().to_string();
            // Limite exprimée en caractères : la tronquage ne doit jamais
            // couper un caractère Unicode (AB-006).
            let note_raw = crate::core::utils::tronquer_texte(&state.transaction_form.note, 1000);
            let note = if note_raw.trim().is_empty() {
                None
            } else {
                Some(note_raw.trim().to_string())
            };

            if !was_edit && state.transaction_form.recurrent {
                let Some(date_choisie) = state.transaction_form.date() else {
                    state.transaction_form.date_error =
                        Some("Date incomplète ou inexistante (JJ/MM/AAAA).".into());
                    return Task::none();
                };

                let dto = CreerRecurrenceDto {
                    libelle: libelle.clone(),
                    montant: montant.clone(),
                    categorie_id: categorie_id.clone(),
                    type_transaction: type_transaction.clone(),
                    jour_du_mois: chrono::Datelike::day(&date_choisie),
                    debut_annee: chrono::Datelike::year(&date_choisie),
                    debut_mois: chrono::Datelike::month(&date_choisie),
                    note: note.clone(),
                };

                return match recurrences_cmd::creer(db, &dto) {
                    Ok(regle) => {
                        state.show_transaction_form = false;
                        state.transaction_form = TransactionFormState::default();
                        if let Err(e) = state.load_data() {
                            state.notification = Some(Notification::erreur(format!(
                                "Récurrence créée, mais rechargement impossible : {e}"
                            )));
                            return Task::none();
                        }
                        state.notification = Some(Notification::succes(format!(
                            "Récurrence créée : {}.",
                            regle.periodicite().to_lowercase()
                        )));
                        Task::none()
                    }
                    Err(e) => {
                        state.notification =
                            Some(Notification::erreur(format!("Récurrence refusée : {e}")));
                        Task::none()
                    }
                };
            }

            let result = if was_edit {
                let dto = ModifierTransactionDto {
                    id: state.transaction_form.edit_id.clone().unwrap_or_default(),
                    libelle,
                    montant,
                    date,
                    categorie_id,
                    type_transaction,
                    statut,
                    note,
                };
                tx_cmd::modifier(db, &dto)
            } else {
                let dto = CreerTransactionDto {
                    libelle,
                    montant,
                    date,
                    categorie_id,
                    type_transaction,
                    statut,
                    note,
                };
                tx_cmd::creer(db, &dto).map(|_| ())
            };

            match result {
                Ok(()) => {
                    state.show_transaction_form = false;
                    state.transaction_form = TransactionFormState::default();
                    if let Err(e) = state.load_month_data() {
                        state.notification = Some(Notification::erreur(format!(
                            "Transaction enregistrée, mais rechargement impossible : {e}"
                        )));
                        return Task::none();
                    }
                    let msg = if was_edit {
                        "Transaction modifiée."
                    } else {
                        match form_kind {
                            TransactionKind::Income => "Revenu ajouté.",
                            TransactionKind::Expense => "Dépense ajoutée.",
                        }
                    };
                    state.notification = Some(Notification::succes(msg));
                }
                Err(e) => {
                    state.notification = Some(Notification::erreur(format!(
                        "Erreur : {e}. Tes modifications sont conservées dans le formulaire."
                    )));
                }
            }
            Task::none()
        }
        Message::DeleteTransaction(id) => {
            if let Some(ref db) = state.db {
                if let Ok(Some(tx)) = tx_cmd::trouver(db, &id) {
                    state.delete_transaction = Some(tx);
                    state.show_delete_confirm = true;
                    // Une seule couche modale à la fois.
                    state.show_transaction_form = false;
                }
            }
            Task::none()
        }
        Message::ConfirmDeleteTransaction => {
            let Some(tx) = state.delete_transaction.clone() else {
                return Task::none();
            };
            let Some(db) = state.db.as_ref() else {
                return Task::none();
            };

            if let Err(e) = tx_cmd::supprimer(db, &tx.id) {
                state.notification =
                    Some(Notification::erreur(format!("Suppression impossible : {e}")));
                return Task::none();
            }
            state.delete_transaction = None;
            state.show_delete_confirm = false;

            if let Err(e) = state.load_month_data() {
                state.notification = Some(Notification::erreur(format!(
                    "Suppression enregistrée, mais rechargement impossible : {e}"
                )));
                return Task::none();
            }
            state.notification = Some(Notification::succes("Transaction supprimée."));
            Task::none()
        }
        Message::CancelDelete => {
            state.delete_transaction = None;
            state.show_delete_confirm = false;
            Task::none()
        }
        Message::ToggleTransactionStatus(id) => {
            let Some(db) = state.db.as_ref() else {
                return Task::none();
            };
            if let Err(e) = tx_cmd::changer_statut(db, &id) {
                state.notification =
                    Some(Notification::erreur(format!("Statut non modifié : {e}")));
                return Task::none();
            }
            if let Err(e) = state.load_month_data() {
                state.notification = Some(Notification::erreur(format!(
                    "Statut modifié, mais rechargement impossible : {e}"
                )));
                return Task::none();
            }
            state.notification = Some(Notification::succes("Statut modifié."));
            Task::none()
        }

        // ---- Filtres ----
        Message::SetFilterKind(kind) => {
            state.filter_kind = kind;
            state.load_month_data().ok();
            Task::none()
        }
        Message::SetFilterStatus(status) => {
            state.filter_status = status;
            state.load_month_data().ok();
            Task::none()
        }
        Message::SetFilterCategory(cat) => {
            state.filter_category = cat;
            state.load_month_data().ok();
            Task::none()
        }
        Message::SetSearchQuery(query) => {
            state.search_query = query;
            state.load_month_data().ok();
            Task::none()
        }
        Message::ClearFilters => {
            state.filter_kind = None;
            state.filter_status = None;
            state.filter_category = None;
            state.search_query.clear();
            state.load_month_data().ok();
            Task::none()
        }

        // ---- Import/Export ----
        Message::ExportDatabase => {
            if let Some(ref db) = state.db {
                let dest_path = {
                    let file = rfd::FileDialog::new()
                        .set_title("Exporter mes données")
                        .set_file_name(io_cmd::nom_sauvegarde())
                        .add_filter("Base SQLite", &["sqlite", "db"])
                        .save_file();

                    match file {
                        Some(path) => path,
                        None => return Task::none(),
                    }
                };

                match io_cmd::exporter(db, &dest_path) {
                    Ok(()) => {
                        let relu = match params_repo::get_settings(db) {
                            Ok(s) => s,
                            Err(e) => {
                                state.notification = Some(Notification::erreur(format!(
                                    "Relecture des paramètres impossible : {e}"
                                )));
                                return Task::none();
                            }
                        };
                        if let Some(mut s) = relu {
                            s.last_export_date = Some(chrono::Utc::now().to_rfc3339());
                            if let Err(e) = params_repo::update_settings(db, &s) {
                                state.notification = Some(Notification::erreur(format!(
                                    "Export réussi, mais mémorisation de la date impossible : {e}"
                                )));
                                return Task::none();
                            }
                            state.settings = Some(s);
                        }
                        state.notification = Some(Notification::succes(format!(
                            "Export réussi vers {}",
                            dest_path.display()
                        )));
                    }
                    Err(e) => {
                        state.notification =
                            Some(Notification::erreur(format!("Erreur d'export : {e}")));
                    }
                }
            }
            Task::none()
        }
        Message::InitiateImport => {
            let file = rfd::FileDialog::new()
                .set_title("Importer mes données")
                .add_filter("Base SQLite", &["sqlite", "db"])
                .pick_file();

            if let Some(path) = file {
                let path_str = path.to_string_lossy().to_string();

                match io_cmd::valider_import(&path) {
                    Ok(()) => {
                        state.import_file_path = Some(path_str);
                        state.show_import_confirm = true;
                        state.import_error = None;
                    }
                    Err(e) => {
                        state.import_error = Some(e);
                    }
                }
            }
            Task::none()
        }
        Message::ConfirmImport(path_str) => {
            let source = std::path::PathBuf::from(&path_str);
            let Some(chemin_actuel) = state.db.as_ref().map(|p| p.path.clone()) else {
                state.import_error = Some("Base de données non initialisée.".into());
                return Task::none();
            };

            Task::perform(
                async move { io_cmd::importer_en_arriere_plan(chemin_actuel, source) },
                Message::ImportResult,
            )
        }
        Message::ImportResult(resultat) => {
            state.show_import_confirm = false;
            state.import_file_path = None;

            // Réouverture systématique : la connexion vivante peut pointer
            // vers un inode remplacé par l'import ou la restauration.
            let reouverture = state
                .db
                .as_ref()
                .map(|p| p.path.clone())
                .ok_or_else(|| "Base de données non initialisée.".to_string())
                .and_then(|chemin| DatabasePool::open(&chemin));

            match (resultat, reouverture) {
                (Ok(()), Ok(pool)) => {
                    state.db = Some(pool);
                    match state.load_data() {
                        Ok(()) => state.notification = Some(Notification::succes("Import réussi.")),
                        Err(e) => state.notification = Some(Notification::erreur(format!(
                            "Import effectué, mais rechargement impossible : {e}"
                        ))),
                    }
                }
                (Ok(()), Err(e)) => {
                    state.notification = Some(Notification::erreur(format!(
                        "Import effectué, mais réouverture impossible : {e}"
                    )));
                }
                (Err(e), Ok(pool)) => {
                    state.db = Some(pool);
                    state.import_error = Some(format!("Erreur d'import : {e}"));
                    state.notification =
                        Some(Notification::erreur(format!("L'import a échoué : {e}")));
                }
                (Err(e), Err(e2)) => {
                    state.import_error = Some(format!("Erreur d'import : {e}"));
                    state.notification = Some(Notification::erreur(format!(
                        "L'import a échoué : {e} ; réouverture impossible : {e2}"
                    )));
                }
            }
            Task::none()
        }
        Message::CancelImport => {
            state.show_import_confirm = false;
            state.import_file_path = None;
            state.import_error = None;
            Task::none()
        }

        // ---- Réinitialisation ----
        Message::OpenResetConfirm => {
            state.show_reset_confirm = true;
            Task::none()
        }
        Message::ConfirmResetData => {
            state.show_reset_confirm = false;

            let efface = match state.db {
                Some(ref db) => params_repo::reset_all_data(db),
                None => Err("Base de données non initialisée.".to_string()),
            };

            match efface {
                Ok(()) => {
                    state.settings = None;
                    state.screen = Screen::Onboarding;
                    state.onboarding_balance_str.clear();
                    state.onboarding_overdraft_str.clear();
                    state.load_data().ok();
                    state.notification = Some(Notification::succes("Données réinitialisées."));
                }
                Err(e) => {
                    state.notification = Some(Notification::erreur(format!(
                        "Réinitialisation impossible : {e}"
                    )));
                }
            }
            Task::none()
        }
        Message::CancelReset => {
            state.show_reset_confirm = false;
            Task::none()
        }

        // ---- Raccourcis clavier ----
        Message::KeyboardEscape => {
            if state.show_transaction_form {
                state.show_transaction_form = false;
                state.transaction_form = TransactionFormState::default();
            } else if state.show_delete_confirm {
                state.show_delete_confirm = false;
                state.delete_transaction = None;
            } else if state.show_import_confirm {
                state.show_import_confirm = false;
            } else if state.show_reset_confirm {
                state.show_reset_confirm = false;
            }
            Task::none()
        }
        Message::KeyboardSave => {
            if state.show_transaction_form {
                return update(state, Message::SubmitTransactionForm);
            }
            Task::none()
        }
        Message::FocusSearch => {
            state.screen = Screen::Transactions;
            focaliser(champ::ID_RECHERCHE)
        }

        // ---- Divers ----
        Message::DismissNotification => {
            state.notification = None;
            Task::none()
        }
        Message::WindowResized(largeur, hauteur) => {
            state.mise_en_page = MiseEnPage::depuis_taille(largeur, hauteur);
            Task::none()
        }
        Message::Tick => {
            // Fait avancer l'animation de chargement et le compte à rebours des
            // notifications : c'est la seule horloge de l'application.
            state.phase_chargement = state.phase_chargement.wrapping_add(1);
            if let Some(notification) = state.notification.as_mut() {
                notification.restant = notification.restant.saturating_sub(1);
                if notification.restant == 0 {
                    state.notification = None;
                }
            }
            Task::none()
        }
        Message::OpenWebsite => {
            if let Err(e) = ouvrir_lien(SITE_WEB) {
                state.notification = Some(Notification::erreur(format!(
                    "Impossible d'ouvrir le navigateur : {e}"
                )));
            }
            Task::none()
        }
        Message::Ignore => Task::none(),
    }
}

/// Adresse du site public d'AfterBudget.
pub const SITE_WEB: &str = "https://afterbudget.vercel.app/";

/// Ouvre une adresse dans le navigateur par défaut du système.
///
/// Chaque plateforme a son propre lanceur ; aucune dépendance supplémentaire
/// n'est nécessaire pour les trois cibles visées.
fn ouvrir_lien(url: &str) -> Result<(), String> {
    #[cfg(target_os = "linux")]
    let (programme, arguments): (&str, Vec<&str>) = ("xdg-open", vec![url]);
    #[cfg(target_os = "macos")]
    let (programme, arguments): (&str, Vec<&str>) = ("open", vec![url]);
    #[cfg(target_os = "windows")]
    let (programme, arguments): (&str, Vec<&str>) = ("cmd", vec!["/C", "start", "", url]);

    std::process::Command::new(programme)
        .args(arguments)
        .spawn()
        .map(|_| ())
        .map_err(|e| e.to_string())
}

/// Met un montant sous une forme directement réutilisable dans un champ de
/// saisie : sans séparateur de milliers ni symbole, virgule décimale.
pub fn montant_editable(montant: Money) -> String {
    let signe = if montant.cents < 0 { "-" } else { "" };
    let abs = montant.cents.unsigned_abs();
    format!("{}{},{:02}", signe, abs / 100, abs % 100)
}

/// Applique et persiste un mode de thème.
fn appliquer_theme(state: &mut AppState, mode: ThemeMode) -> Task<Message> {
    state.theme_mode = mode;
    let Some(db) = state.db.as_ref() else {
        return Task::none();
    };
    if let Err(e) = parametres_service::mettre_a_jour_theme(db, mode.cle()) {
        state.notification =
            Some(Notification::erreur(format!("Thème non enregistré : {e}")));
    } else {
        state.settings = parametres_service::obtenir_parametres(db).ok().flatten();
    }
    Task::none()
}

/// Place le focus clavier sur un champ identifié.
fn focaliser(identifiant: &'static str) -> Task<Message> {
    iced::widget::text_input::focus(iced::widget::text_input::Id::new(identifiant))
}
