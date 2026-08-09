use afterbudget::app::message::{Message, Screen};
use afterbudget::app::state::AppState;
use afterbudget::app::{update::update, view::view};
use afterbudget::core::config;
use afterbudget::core::db::migrations;
use afterbudget::core::db::pool::DatabasePool;
use afterbudget::ui::theme::mise_en_page::{HAUTEUR_MINIMALE, LARGEUR_MINIMALE};
use iced::keyboard::{key::Named, Key, Modifiers};
use iced::{window, Size, Subscription};

/// Taille d'ouverture : assez large pour la disposition en deux colonnes.
const TAILLE_INITIALE: Size = Size::new(1240.0, 780.0);

pub fn main() -> iced::Result {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let db_path = match config::database_path() {
        Ok(chemin) => chemin,
        Err(e) => {
            tracing::error!("{}", e);
            eprintln!("AfterBudget : {}", e);
            std::process::exit(1);
        }
    };
    let db = match DatabasePool::open(&db_path) {
        Ok(db) => db,
        Err(e) => {
            tracing::error!("{}", e);
            eprintln!("AfterBudget : {}", e);
            std::process::exit(1);
        }
    };

    if let Err(e) = migrations::run_migrations(&db.conn) {
        tracing::error!("{}", e);
        eprintln!("AfterBudget : {}", e);
        std::process::exit(1);
    }

    let mut state = AppState::new();
    state.db_path = db.path.to_string_lossy().to_string();
    state.db = Some(db);

    if let Err(e) = state.load_data() {
        tracing::error!("Chargement initial impossible : {}", e);
        eprintln!(
            "AfterBudget : le chargement initial a échoué ({}) — arrêt.",
            e
        );
        std::process::exit(1);
    }

    iced::application("AfterBudget", update, view)
        .window(window::Settings {
            size: TAILLE_INITIALE,
            min_size: Some(Size::new(LARGEUR_MINIMALE, HAUTEUR_MINIMALE)),
            ..Default::default()
        })
        // Le thème Iced reste neutre : toute la peinture passe par la palette
        // du design system, aucun widget ne retombe sur les couleurs par défaut.
        .theme(|_state: &AppState| iced::Theme::Light)
        .subscription(souscriptions)
        // La première tâche déclenche la vérification de mise à jour : elle ne
        // bloque ni le rendu ni la saisie, et son échec reste silencieux.
        .run_with(|| (state, iced::Task::done(Message::CheckForUpdates)))
}

fn souscriptions(_state: &AppState) -> Subscription<Message> {
    Subscription::batch([
        // Horloge unique : compte à rebours des notifications et animation de
        // l'indicateur de chargement.
        iced::time::every(std::time::Duration::from_secs(1)).map(|_| Message::Tick),
        window::resize_events()
            .map(|(_, taille)| Message::WindowResized(taille.width, taille.height)),
        iced::keyboard::on_key_press(raccourci),
    ])
}

/// Traduction d'une frappe en message.
///
/// Les raccourcis à modificateur ne perturbent pas la saisie de texte ; `Échap`
/// est capté seul, car il ne peut être confondu avec aucune saisie.
fn raccourci(touche: Key, modificateurs: Modifiers) -> Option<Message> {
    if let Key::Named(Named::Escape) = touche {
        return Some(Message::KeyboardEscape);
    }

    if !modificateurs.command() {
        return None;
    }

    match touche.as_ref() {
        Key::Character("n") => Some(if modificateurs.shift() {
            Message::OpenAddIncome
        } else {
            Message::OpenAddExpense
        }),
        Key::Character("f") => Some(Message::FocusSearch),
        Key::Character("s") => Some(Message::KeyboardSave),
        Key::Character("1") => Some(Message::NavigateTo(Screen::Dashboard)),
        Key::Character("2") => Some(Message::NavigateTo(Screen::Transactions)),
        Key::Character("3") => Some(Message::NavigateTo(Screen::Statistics)),
        Key::Character("4") => Some(Message::NavigateTo(Screen::Settings)),
        Key::Named(Named::ArrowLeft) => Some(Message::PreviousMonth),
        Key::Named(Named::ArrowRight) => Some(Message::NextMonth),
        _ => None,
    }
}
